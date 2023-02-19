use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

#[derive(Debug)]
struct Stats {
    term_frequency: HashMap<String, usize>,
    file: String,
    length: usize,
}

fn main() {
    let file = "ga.srt";
    let words = find_words(file);
    let st = get_stats(&words, file);
    write_stats(&st);
    // dbg!(st);
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
    let mut stats: HashMap<String, usize> = HashMap::new();
    let len = words.len();
    words.into_iter().for_each(|w| {
        stats
            .entry(w.clone())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
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
    let mut vec: Vec<(&String, &usize)> = s.term_frequency.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1));

    let mut file = BufWriter::new(file);
    let head = format!(
        "{:<22} {:<22}{}\n{}\n",
        "WORD:",
        "FREQUENCY:",
        "PERCENT:",
        "-".repeat(53)
    );
    file.write(head.as_bytes()).unwrap();

    let s = vec
        .iter()
        .map(|(key, value)| {
            let per = **value as f64 * 100.0 / s.length as f64;
            format!("{:<22} {:<22} {:.2}%\n", key, *value, per)
        })
        .collect::<String>();
    file.write_all(s.as_bytes()).unwrap();
}
