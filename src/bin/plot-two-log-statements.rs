use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

const MIN: usize = 0;
const MAX: usize = 10_000;

const SERIES: &[(&str, &str)] = &[
    // (
    //     "Building stage0 compiler artifacts",
    //     "Assembling stage2 compiler",
    // ),
    (
        "Building stage0 compiler artifacts",
        "Copying stage0 rustc",
    ),
    (
        "Building stage1 compiler artifacts",
        "Copying stage1 rustc",
    ),
    (
        "Check compiletest suite=run-pass mode=run-pass",
        "test result: ok",
    ),
    (
        "Testing libstd stage1",
        "Testing libtest stage1",
    ),
    (
        "Building LLVM",
        "Building stage0 compiler artifacts",
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
    let mut data2 = String::new();
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

        let mut last = zero;
        for line in lines {
            last = parse(line).1;
            for (series, durs) in SERIES.iter().zip(&mut durs) {
                if durs.0 == zero {
                    if line.contains(series.0) {
                        durs.0 = last;
                    }
                } else if durs.1 == zero {
                    if line.contains(series.1) {
                        durs.1 = last;
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
            .collect::<Vec<_>>();
        if secs.iter().any(|s| *s == 0) {
            continue
        }
        let remaining = durs.iter().fold(last, |c, &(a, b)| c - (b - a));
        data.push_str(file.file_name().unwrap().to_str().unwrap());
        data.push_str(" ");
        data.push_str(&(remaining.as_secs() + secs.iter().sum::<u64>()).to_string());
        data.push_str(" ");
        for (i, s) in secs.iter().enumerate() {
            data.push_str(&(s + secs[i+1..].iter().sum::<u64>()).to_string());
            data.push_str(" ");
        }
        data.push_str("\n");

        data2.push_str(file.file_name().unwrap().to_str().unwrap());
        data2.push_str(" ");
        data2.push_str(&remaining.as_secs().to_string());
        data2.push_str(" ");
        for s in secs.iter() {
            data2.push_str(&s.to_string());
            data2.push_str(" ");
        }
        data2.push_str("\n");
    }

    fs::File::create("two-log-statements.dat").unwrap()
        .write_all(data.as_bytes()).unwrap();
    fs::File::create("two-log-statements2.dat").unwrap()
        .write_all(data2.as_bytes()).unwrap();
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
