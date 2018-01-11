use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

const MIN: usize = 0;
const MAX: usize = 10_000;

const SERIES: &[(&str, &str)] = &[
    (
        "Building stage0 compiler artifacts",
        "Assembling stage2 compiler",
    ),
    (
        "Check compiletest suite=run-pass mode=run-pass",
        "test result: ok",
    ),
    (
        "Building LLVM",
        "Building stage0 compiler artifacts",
    ),
    (
        "Testing libstd stage1",
        "Testing libtest stage1",
    ),
    (
        "Building stage0 compiler artifacts",
        "Copying stage0 rustc",
    ),
    (
        "Building stage1 compiler artifacts",
        "Copying stage1 rustc",
    ),
];

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
    let zero = Duration::from_secs(0);
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

        let mut durs = SERIES.iter()
            .map(|_| (zero, zero))
            .collect::<Vec<_>>();

        let mut lines = contents.lines();

        for line in lines {
            for (series, durs) in SERIES.iter().zip(&mut durs) {
                if durs.0 == zero {
                    if line.contains(series.0) {
                        durs.0 = parse(line).1;
                    }
                } else if durs.1 == zero {
                    if line.contains(series.1) {
                        durs.1 = parse(line).1;
                    }
                }
            }
        }

        let secs = durs.iter()
            .map(|&(a, b)| {
                if a != zero && b != zero {
                    (b - a).as_secs()
                } else {
                    0
                }
            })
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if secs.iter().any(|s| *s == "0") {
            continue
        }
        data.push_str(&format!("{} {}\n",
                               file.file_name().unwrap().to_str().unwrap(),
                               secs.join(" ")));
    }

    fs::File::create("two-log-statements.dat").unwrap()
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
