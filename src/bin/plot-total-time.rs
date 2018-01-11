use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

const MIN: usize = 0;
const MAX: usize = 10_000;

fn main() {
    let path = env::args().nth(1).unwrap();
    let path = Path::new(&path);

    let mut files = path.read_dir()
        .unwrap()
        .map(|e| e.unwrap())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    files.sort();

    let mut data = String::new();
    let mut contents = String::new();
    for file in files {
		let num = file.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        if num < MIN || num > MAX {
            continue
        }

        contents.truncate(0);
        fs::File::open(&file).unwrap()
            .read_to_string(&mut contents).unwrap();

        let last = contents.lines().last().unwrap();
        let (_, dur) = parse(last);
        data.push_str(&format!("{} {}\n",
                               file.file_name().unwrap().to_str().unwrap(),
                               dur.as_secs()));
    }

    fs::File::create("total-time.dat").unwrap()
        .write_all(data.as_bytes()).unwrap();
}

fn parse(s: &str) -> (&str, Duration) {
    assert!(s.starts_with('['));
    let i = s.find(']').unwrap();
    let mut dur = s[1..i].split(':');
    let line = &s[i + 1..];
    let h = dur.next().unwrap().parse::<u64>().unwrap();
    let m = dur.next().unwrap().parse::<u64>().unwrap();
    let s = dur.next().unwrap().parse::<u64>().unwrap();
    (line, Duration::from_secs(h * 3600 + m * 60 + s))
}
