use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let words = find_words("ga.srt");
    let stats = words_stats(&words);
    dbg!(stats);
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn find_words(file: &str) -> Vec<String> {
    let re = regex::Regex::new("(?:[A-Za-z]+’?-?'?[A-Za-z]+)|(?:[A-Za-z]+)").unwrap();
    let lines = read_lines(file).unwrap();

    lines
        .filter_map(|l| match l {
            Ok(l) => Some(l),
            Err(_) => None,
        })
        .map(|l| {
            re.find_iter(&l)
                .map(|w| w.as_str().to_owned())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        .flatten()
        .collect()
}

fn words_stats(words: &Vec<String>) -> HashMap<String, usize> {
    let mut stats: HashMap<String, usize> = HashMap::new();
    words.into_iter().for_each(|w| {
        stats
            .entry(w.clone())
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    });
    stats
}
