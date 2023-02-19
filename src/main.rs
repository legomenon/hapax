use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

#[derive(Debug)]
struct Stats {
    term_frequency: HashMap<String, (usize, f64)>,
    file: String,
    length: usize,
}

fn main() {
    let file = "ga.sr";
    let words = find_words(file);
    let st = get_stats(&words, file);
    write_stats(&st);
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn find_words(file: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"\b[\p{L}’'-]+\b"#).unwrap();
    let lines = read_lines(file).unwrap();

    lines
        .filter_map(|l| match l {
            Ok(l) => Some(l),
            Err(_) => None,
        })
        .map(|l| {
            re.find_iter(&l)
                .map(|w| w.as_str().to_owned().to_lowercase())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        .flatten()
        .collect()
}

fn get_stats(words: &Vec<String>, file: &str) -> Stats {
    let mut stats: HashMap<String, (usize, f64)> = HashMap::new();
    let len = words.len();

    words.into_iter().for_each(|w| {
        stats
            .entry(w.clone())
            .and_modify(|counter| counter.0 += 1)
            .or_insert((1, 0.0));
    });

    stats.iter_mut().for_each(|(_, v)| {
        let per = v.0 as f64 * 100.0 / len as f64;
        v.1 = per;
    });

    Stats {
        term_frequency: stats,
        file: file.to_owned(),
        length: len,
    }
}

fn write_stats(s: &Stats) {
    let file_name = s.file.to_owned() + ".stats";
    let file = File::create(file_name).unwrap();
    let mut file = BufWriter::new(file);

    let mut v: Vec<(&String, &(usize, f64))> = s.term_frequency.iter().collect();
    v.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    let head = format!(
        "FILE: {:<16} LENGTH: {}\n\n{:<22} {:<22}{}\n{}\n",
        s.file,
        s.length,
        "WORD:",
        "FREQUENCY:",
        "PERCENT:",
        "-".repeat(53)
    );

    file.write(head.as_bytes()).unwrap();

    let s = v
        .into_iter()
        .map(|(key, val)| format!("{:<22} {:<22} {:.2}%\n", key, val.0, val.1))
        .collect::<String>();

    file.write_all(s.as_bytes()).unwrap();
}
