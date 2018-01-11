extern crate curl;
extern crate futures;
extern crate getopts;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate tokio_core;
extern crate tokio_curl;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

use std::env;
use std::fs;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::rc::Rc;
use std::vec;

use failure::Error;
use futures::prelude::*;
use futures::stream;
use tokio_core::reactor::Core;
use tokio_curl::Session;

pub type MyFuture<T> = Box<Future<Item = T, Error = Error>>;

#[macro_export]
macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => {
            let e: ::failure::Error = e.into();
            println!("{} failed with {}", stringify!($e), e);
            for e in e.causes().skip(1) {
                println!("caused by: {}", e);
            }
            panic!("wut");
        }
    })
}

#[derive(Deserialize, Debug)]
pub struct History {
    pub project: Project,
    pub builds: Vec<Build>,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    #[serde(rename = "projectId")]
    pub project_id: u32,
    #[serde(rename = "accountId")]
    pub account_id: u32,
    #[serde(rename = "accountName")]
    pub account_name: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "repositoryName")]
    pub repository_name: String,
    #[serde(rename = "repositoryType")]
    pub repository_type: String,
}

#[derive(Deserialize)]
pub struct GetBuild {
    pub build: Build,
}

#[derive(Deserialize, Debug)]
pub struct Build {
    #[serde(rename = "buildId")]
    pub build_id: u32,
    pub jobs: Vec<Job>,
    #[serde(rename = "buildNumber")]
    pub build_number: u32,
    pub version: String,
    pub message: String,
    pub branch: String,
    #[serde(rename = "commitId")]
    pub commit_id: String,
    pub status: String,
    pub started: Option<String>,
    pub finished: Option<String>,
    pub created: String,
    pub updated: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    #[serde(rename = "jobId")]
    pub job_id: String,
    pub status: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct LastBuild {
    pub build: Build,
}

pub struct AppVeyorBuilds {
    session: Session,
    pending: vec::IntoIter<Build>,
    next_start: Option<u32>,
    fetching: Option<MyFuture<History>>,
    token: String,
    branch: Option<String>,
}

impl AppVeyorBuilds {
    pub fn new(session: Session,
               token: String,
               branch: Option<String>) -> AppVeyorBuilds {
        AppVeyorBuilds {
            session: session,
            pending: Vec::new().into_iter(),
            fetching: None,
            next_start: None,
            token: token,
            branch: branch,
        }
    }
}

impl Stream for AppVeyorBuilds {
    type Item = Build;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Error> {
        const PER_PAGE: u32 = 100;
        loop {
            if let Some(item) = self.pending.next() {
                return Ok(Some(item).into())
            }

            match self.fetching.poll()? {
                Async::Ready(None) => {}
                Async::Ready(Some(builds)) => {
                    self.fetching = None;
                    let min = builds.builds.iter().map(|b| b.build_id).min();
                    self.next_start = min;
                    self.pending = builds.builds.into_iter();
                    continue
                }
                Async::NotReady => return Ok(Async::NotReady),
            }

            let mut url = "/projects/rust-lang/rust/history?recordsNumber=".to_string();
            url.push_str(&PER_PAGE.to_string());
            if let Some(ref branch) = self.branch {
                url.push_str("&branch=");
                url.push_str(branch);
            }
            if let Some(s) = self.next_start {
                url.push_str("&startBuildId=");
                url.push_str(&s.to_string());
            }
            self.fetching = Some(http::appveyor_get(&self.session,
                                                    &url,
                                                    &self.token));
        }
    }
}

mod http {
    use std::sync::{Arc, Mutex};
    use std::str;

    use curl::easy::{Easy, List};
    use failure::ResultExt;
    use futures::Future;
    use serde::Deserialize;
    use serde_json;
    use tokio_curl::Session;

    use MyFuture;

    #[allow(dead_code)]
    pub struct Response {
        easy: Easy,
        headers: Arc<Mutex<Vec<Vec<u8>>>>,
        pub body: Arc<Mutex<Vec<u8>>>,
    }

    pub fn appveyor_get<T>(sess: &Session,
                           url: &str,
                           token: &str) -> MyFuture<T>
        where T: Deserialize + 'static,
    {
        let headers = vec![
            format!("Authorization: Bearer {}", token),
            format!("Accept: application/json"),
        ];

        get_json(sess,
                 &format!("https://ci.appveyor.com/api{}", url),
                 None,
                 None,
                 &headers)
    }

    pub fn get_json<T>(sess: &Session,
                       url: &str,
                       user: Option<&str>,
                       pass: Option<&str>,
                       headers: &[String]) -> MyFuture<T>
        where T: Deserialize + 'static
    {
        let response = get(sess, url, user, pass, headers);
        let ret = response.and_then(|response| {
            let body = response.body.lock().unwrap();
            let json = str::from_utf8(&body)?;
            let ret = serde_json::from_str(json).with_context(|_| {
                format!("failed to decode: {:#?}", json)
            })?;
            Ok(ret)
        });
        Box::new(ret)
    }

    pub fn get(sess: &Session,
               url: &str,
               user: Option<&str>,
               pass: Option<&str>,
               headers: &[String]) -> MyFuture<Response> {
        let mut handle = Easy::new();
        let mut list = List::new();
        t!(list.append("User-Agent: hello/1.2"));
        for header in headers {
            t!(list.append(header));
        }

        if let Some(user) = user {
            t!(handle.username(user));
        }
        if let Some(pass) = pass {
            t!(handle.password(pass));
        }

        t!(handle.http_headers(list));
        t!(handle.get(true));
        t!(handle.url(url));

        perform(sess, handle, url)
    }

    pub fn perform(sess: &Session, mut easy: Easy, url: &str) -> MyFuture<Response> {
        debug!("fetching: {}", url);
        let headers = Arc::new(Mutex::new(Vec::new()));
        let data = Arc::new(Mutex::new(Vec::new()));

        let (data2, headers2) = (data.clone(), headers.clone());
        t!(easy.header_function(move |data| {
            headers2.lock().unwrap().push(data.to_owned());
            true
        }));
        t!(easy.write_function(move |buf| {
            data2.lock().unwrap().extend_from_slice(&buf);
            Ok(buf.len())
        }));

        let response = sess.perform(easy);
        let url = url.to_string();
        let checked_response = response
            .map_err(|e| e.into_error().into())
            .and_then(move |mut easy| {
                debug!("finished: {}", url);
                match t!(easy.response_code()) {
                    200 | 204 => {
                        Ok(Response {
                            easy: easy,
                            headers: headers,
                            body: data,
                        })
                    }
                    code => {
                        Err(format_err!("not a 200 code: {}\n\n{}\n",
                                    code,
                                    String::from_utf8_lossy(&data.lock().unwrap())))
                    }
                }
            });

        Box::new(checked_response)
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let sess = Session::new(core.handle());
    let token = env::args().nth(1).unwrap();
    let branch = "auto".to_string();
    let builds = AppVeyorBuilds::new(sess.clone(), token.clone(), Some(branch));

    // we're only interested in successful builds right now
    let builds = builds.filter(|b| b.status == "success");

    // fetch the `Build` again so we can include all the jobs
    let builds = builds.map(|b| {
        let url = format!("/projects/rust-lang/rust/build/{}", b.version);
        http::appveyor_get::<GetBuild>(&sess, &url, &token)
            .map(|b| b.build)
    }).buffer_unordered(20);

    // Flatten our list of builds into a list of (build, job) pairs. At the same
    // time also filter the jobs to the only ones we're interested in.
    let builds = builds.map(|mut b| {
        let jobs = mem::replace(&mut b.jobs, Vec::new());
        let b = Rc::new(b);
        stream::iter_ok::<_, Error>(
            jobs.into_iter()
                .filter(|j| j.name.contains("build=i686"))
                .filter(|j| j.name.contains("x.py test"))
                .map(move |j| (b.clone(), j))
        )
    }).flatten();

    // Fetch the build logs for a job
    let builds = builds.map(|(b, j)| {
        let url = format!("https://ci.appveyor.com/api/buildjobs/{}/log", j.job_id);
        http::get(&sess, &url, None, None, &[])
            .map(move |response| {
                let mut body = response.body.lock().unwrap();
                let text = String::from_utf8(mem::replace(&mut *body, Vec::new()))
                    .unwrap();
                (b, j, text)
            })
    }).buffer_unordered(20);

    let dst = Path::new("logs");
    t!(fs::create_dir_all(&dst));
    t!(fs::create_dir_all(&dst.join("msvc")));
    t!(fs::create_dir_all(&dst.join("gnu")));

    // Write out all logs to the logs directory with indexed logs
    let c = builds.for_each(|(build, job, log)| {
        let msvc = job.name.contains("pc-windows-msvc");
        let dst = dst.join(if msvc { "msvc" } else { "gnu" });
        let dst = dst.join(build.build_number.to_string());

        t!(t!(fs::File::create(&dst)).write_all(log.as_bytes()));
        println!("{}", dst.display());
        Ok(())
    });

    t!(core.run(c));
}
