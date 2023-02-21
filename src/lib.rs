use serde::Serialize;
use serde_json::to_string;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

// use rayon::prelude::*;

#[derive(Serialize, Debug)]
pub struct Stats {
    file_name: String,
    length: usize,
    term_frequency: HashMap<String, TF>,
}
#[derive(Debug, Serialize)]
struct TF {
    freq: usize,
    per: f64,
}

impl Stats {
    pub fn build(words: &Vec<String>, file: &Path) -> Self {
        let mut stats: HashMap<String, TF> = HashMap::new();
        let len = words.len();

        words.iter().for_each(|w| {
            stats
                .entry(w.clone())
                .and_modify(|counter| counter.freq += 1)
                .or_insert(TF { freq: 1, per: 0.0 });
        });

        stats.iter_mut().for_each(|(_, v)| {
            let per = v.freq as f64 * 100.0 / len as f64;
            v.per = per;
        });
        let mut f = file.to_path_buf();
        f.pop();
        let file_name = file
            .display()
            .to_string()
            .replace(&f.display().to_string(), "")
            .replace('/', "");

        println!("{}", file_name);

        Self {
            term_frequency: stats,
            file_name,
            length: len,
        }
    }

    pub fn write(&self, o: &str, p: &str) {
        let dir = p.to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir).unwrap();
        }
        let file_name = dir + "/" + &self.file_name;

        match o {
            "Json" => self.write_json(file_name),
            "Text" => self.write_text(file_name),
            "Csv" => self.write_csv(file_name),
            _ => unreachable!(),
        }
    }

    fn write_text(&self, file_name: String) {
        let file = File::create(file_name + ".stats.txt").unwrap();
        let mut file = BufWriter::new(file);

        let mut v: Vec<(&String, &TF)> = self.term_frequency.iter().collect();
        v.sort_by(|a, b| b.1.freq.cmp(&a.1.freq));

        let data = format!(
            "FILE: {:<16} LENGTH: {}\n\n{:<22} {:<22}{}\n{}\n",
            self.file_name,
            self.length,
            "WORD:",
            "FREQUENCY:",
            "PERCENT:",
            "-".repeat(53)
        );

        let s = v
            .iter()
            .map(|(key, val)| format!("{:<22} {:<22} {:.2}%\n", key, val.freq, val.per))
            .collect::<String>();

        let data = data + &s;
        file.write_all(data.as_bytes()).unwrap();
    }

    fn write_csv(&self, file_name: String) {
        let file = File::create(file_name + ".stats.csv").unwrap();
        let mut file = BufWriter::new(file);

        let mut v: Vec<(&String, &TF)> = self.term_frequency.iter().collect();
        v.sort_by(|a, b| b.1.freq.cmp(&a.1.freq));

        let data = format!(
            "FILE,LENGTH\n{},{}\n\nWORD,FREQUENCY,PERCENT\n",
            self.file_name, self.length
        );

        let s = v
            .iter()
            .map(|(key, val)| format!("{},{},{}\n", key, val.freq, val.per))
            .collect::<String>();

        let data = data + &s;
        file.write_all(data.as_bytes()).unwrap();
    }

    fn write_json(&self, file_name: String) {
        let file = File::create(file_name + ".stats.json").unwrap();
        let mut file = BufWriter::new(file);

        let s = to_string(&self).unwrap();
        file.write_all(s.as_bytes()).unwrap();
    }
}

pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn find_files_in_dir<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let entries = fs::read_dir(dir).unwrap();
    entries
        .flatten()
        .filter(|file| file.file_type().map_or(false, |t| t.is_file()))
        .map(|file| file.path())
        .collect()
}

pub fn find_words_in_file(file: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"\b[\p{L}’'-]+\b"#).unwrap();
    let lines = read_lines(file).unwrap();

    lines
        .filter_map(|l| match l {
            Ok(l) => Some(l),
            Err(_) => None,
        })
        .flat_map(|l| {
            re.find_iter(&l)
                .map(|w| w.as_str().to_owned().to_lowercase())
                .collect::<Vec<String>>()
        })
        .collect()
}
