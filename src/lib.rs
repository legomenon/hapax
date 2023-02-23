use serde::Serialize;
use serde_json::to_string;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

// use rayon::prelude::*;

#[derive(Serialize, Debug)]
pub struct Stats {
    pub file_name: String,
    length: usize,

    term_frequency: HashMap<String, TF>,
}
#[derive(Debug, Serialize)]
struct TF {
    freq: usize,
    per: f64,
}

impl Stats {
    pub fn new(words: &Vec<String>, file: &Path) -> Self {
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

        Self {
            term_frequency: stats,
            file_name,
            length: len,
        }
    }
    pub fn new_total() -> Self {
        Self {
            file_name: "total".to_owned(),
            length: 0,
            term_frequency: HashMap::new(),
        }
    }

    pub fn extend(&mut self, words: &[String]) {
        self.length += words.len();

        words.iter().for_each(|w| {
            self.term_frequency
                .entry(w.clone())
                .and_modify(|counter| counter.freq += 1)
                .or_insert(TF { freq: 1, per: 0.0 });
        });

        self.term_frequency.iter_mut().for_each(|(_, v)| {
            let per = v.freq as f64 * 100.0 / self.length as f64;
            v.per = per;
        });
    }

    pub fn exclude_junk<P: AsRef<Path>>(&mut self, s: P) {
        let j_words = fs::read_to_string(s).expect("Couldn't read junk words");
        let j_set: HashSet<String> = j_words.split('\n').map(|w| w.to_lowercase()).collect();
        j_set.iter().for_each(|w| {
            if self.term_frequency.contains_key(w.as_str()) {
                self.term_frequency.remove(w.as_str());
            }
        });

        // self.term_frequency.iter_mut().for_each(|w| {
        //     if j_set.contains(w.0.as_str()) {
        //         self.term_frequency.remove(w.0.as_str());
        //     }
        // });
        self.length = self.term_frequency.len();
    }

    pub fn write(&self, o: &str, p: &str) -> io::Result<()> {
        let dir = p.to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir)?;
        }
        let file_name = dir + "/" + &self.file_name;

        match o {
            "Json" => self.write_json(file_name)?,
            "Text" => self.write_text(file_name)?,
            "Csv" => self.write_csv(file_name)?,
            _ => unreachable!(),
        }
        Ok(())
    }

    fn write_text(&self, file_name: String) -> io::Result<()> {
        let file = File::create(file_name + ".stats.txt")?;
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
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn write_csv(&self, file_name: String) -> io::Result<()> {
        let file = File::create(file_name + ".stats.csv")?;
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
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn write_json(&self, file_name: String) -> io::Result<()> {
        let file = File::create(file_name + ".stats.json")?;
        let mut file = BufWriter::new(file);

        let s = to_string(&self)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn find_files_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?;
    Ok(entries
        .flatten()
        .filter(|file| file.file_type().map_or(false, |t| t.is_file()))
        .map(|file| file.path())
        .collect())
}

pub fn find_words_in_file(file: &str) -> io::Result<Vec<String>> {
    let re = regex::Regex::new(r#"\b[\p{L}’'-]+\b"#).expect("Failed to parse regex");
    let lines = read_lines(file)?;

    Ok(lines
        .filter_map(|l| match l {
            Ok(l) => Some(l),
            Err(_) => None,
        })
        .flat_map(|l| {
            re.find_iter(&l)
                .map(|w| w.as_str().to_owned().to_lowercase())
                .collect::<Vec<String>>()
        })
        .collect())
}
