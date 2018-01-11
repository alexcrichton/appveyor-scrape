use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

fn main() {
    let mut a = String::new();
    let mut b = String::new();

    let mut args = env::args().skip(1);
    File::open(args.next().unwrap()).unwrap().read_to_string(&mut a).unwrap();
    File::open(args.next().unwrap()).unwrap().read_to_string(&mut b).unwrap();

    let mut alines = HashMap::new();
    for line in a.lines() {
        let (l, dur) = parse(line);
        if l.trim().len() == 0 {
            continue
        }
        alines.entry(l).or_insert(Vec::new()).push(dur);
    }

    let mut diff = Vec::new();

    for line in b.lines() {
        let (l, dur) = parse(line);
        let times = match alines.get_mut(l) {
            Some(l) => l,
            None => continue,
        };
        if times.len() == 0 {
            continue
        }
        let time = times.remove(0);
        if dur > time && dur - time > Duration::from_secs(10) {
            diff.push((dur - time, l));
        }
    }
    diff.sort();
    for &(t, l) in diff.iter() {
        println!("{} {:?}", l, t);
    }
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
