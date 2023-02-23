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
    unique: usize,
    total: usize,
    term_frequency: HashMap<String, TF>,
}
#[derive(Debug, Serialize)]
struct TF {
    freq: usize,
    per: f64,
}

impl Stats {
    pub fn new(words: &Vec<String>, file: &Path) -> Self {
        let mut term_frequency: HashMap<String, TF> = HashMap::new();
        let len = words.len();

        let mut total: usize = 0;

        words.iter().for_each(|w| {
            term_frequency
                .entry(w.clone())
                .and_modify(|counter| counter.freq += 1)
                .or_insert(TF { freq: 1, per: 0.0 });
        });

        term_frequency.iter_mut().for_each(|(_, v)| {
            total += v.freq;
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

        let unique = term_frequency.len();

        Self {
            term_frequency,
            file_name,
            unique,
            total,
        }
    }
    pub fn new_total() -> Self {
        Self {
            file_name: "total".to_owned(),
            unique: 0,
            term_frequency: HashMap::new(),
            total: 0,
        }
    }

    pub fn extend(&mut self, words: &[String]) {
        words.iter().for_each(|w| {
            self.term_frequency
                .entry(w.clone())
                .and_modify(|counter| counter.freq += 1)
                .or_insert(TF { freq: 1, per: 0.0 });
        });
        self.recalculate();
    }

    pub fn exclude_junk<P: AsRef<Path>>(&mut self, s: P) {
        let j_words = fs::read_to_string(s).expect("Couldn't read junk words");
        let j_set: HashSet<String> = j_words.split('\n').map(|w| w.to_lowercase()).collect();
        j_set.iter().for_each(|w| {
            if self.term_frequency.contains_key(w.as_str()) {
                self.term_frequency.remove(w.as_str());
            }
        });
        self.recalculate();
    }

    fn recalculate(&mut self) {
        self.total = 1;
        self.unique = self.term_frequency.len();
        self.term_frequency.iter().for_each(|(_, v)| {
            self.total += v.freq;
        });
        self.term_frequency.iter_mut().for_each(|(_, v)| {
            let per = v.freq as f64 * 100.0 / self.total as f64;
            v.per = per;
        });
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
            "FILE: {:<16} UNIQUE: {:<13} TOTAL:{}\n\n{:<22} {:<22}{}\n{}\n",
            self.file_name,
            self.unique,
            self.total,
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
            "FILE,UNIQUE,TOTAL\n{},{},{}\n\nWORD,FREQUENCY,PERCENT\n",
            self.file_name, self.unique, self.total
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
    let re = regex::Regex::new(r#"\b[A-Za-z]+\b"#).expect("Failed to parse regex");
    let br = regex::Regex::new(r#"<[^>]+>"#).expect("Failed to parse regex");
    let lines = read_lines(file)?;

    Ok(lines
        .filter_map(|l| match l {
            Ok(l) => Some(l),
            Err(_) => None,
        })
        .flat_map(|l| {
            let r: String = br.replace_all(&l, "").into();
            re.find_iter(&r)
                .map(|w| w.as_str().to_owned().to_lowercase())
                .collect::<Vec<String>>()
        })
        .collect())
}

pub fn lemmanization(v: Vec<String>) -> io::Result<Vec<String>> {
    let file = fs::read_to_string("./lemmas.txt")?;
    let l = file
        .split('\n')
        .map(|l| l.split(' ').map(|w| w.to_owned()).collect::<Vec<_>>())
        .map(|w: Vec<String>| {
            let word = w[0].clone();
            let lemmas: Vec<String> = w[1..].to_vec();
            (word, lemmas)
        })
        .collect::<Vec<_>>();

    let mut lemma: HashMap<String, String> = HashMap::new();
    l.into_iter().for_each(|w| {
        w.1.into_iter().for_each(|l| {
            lemma.insert(l, w.0.clone());
        })
    });

    let mut new_v: Vec<String> = Vec::new();
    v.into_iter().for_each(|w| match lemma.get(&w) {
        Some(l) => new_v.push(l.to_owned()),
        None => new_v.push(w),
    });
    Ok(new_v)
}
