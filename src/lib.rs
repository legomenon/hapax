use serde::Serialize;
use serde_json::to_string;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

#[derive(Serialize, Debug)]
pub struct Stats<'a> {
    pub file_name: Cow<'a, str>,
    unique: usize,
    total: usize,
    term_frequency: HashMap<Cow<'a, str>, (usize, f64)>,
}

#[derive(Clone, Copy)]
pub enum Output {
    Json,
    Txt,
    Csv,
}

pub struct Options {
    pub output_type: Output,
    pub output_path: PathBuf,
    pub skip_junk_words: bool,
    pub skip_lemmanization: bool,
}

impl<'a> Stats<'a> {
    pub fn new(words: &'a [String], file: &'a Path) -> Self {
        let mut term_frequency: HashMap<Cow<'a, str>, (usize, f64)> = HashMap::new();
        let len = words.len();

        let mut total: usize = 0;

        words.iter().for_each(|w| {
            term_frequency
                .entry(Cow::Borrowed(w))
                .and_modify(|counter| counter.0 += 1)
                .or_insert((1, 0.0));
        });

        term_frequency.iter_mut().for_each(|(_, v)| {
            total += v.0;
            let per = v.0 as f64 * 100.0 / len as f64;
            v.1 = per;
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
            file_name: Cow::Owned(file_name),
            unique,
            total,
        }
    }
    pub fn new_total() -> Self {
        Self {
            file_name: Cow::Borrowed("total"),
            unique: 0,
            term_frequency: HashMap::new(),
            total: 0,
        }
    }

    pub fn extend(&mut self, words: &'a [String]) {
        words.iter().for_each(|w| {
            self.term_frequency
                .entry(Cow::Borrowed(w))
                .and_modify(|counter| counter.0 += 1)
                .or_insert((1, 0.0));
        });
        self.recalculate();
    }

    fn recalculate(&mut self) {
        self.total = 1;
        self.unique = self.term_frequency.len();
        self.term_frequency.iter().for_each(|(_, v)| {
            self.total += v.0;
        });
        self.term_frequency.iter_mut().for_each(|(_, v)| {
            let per = v.0 as f64 * 100.0 / self.total as f64;
            v.1 = per;
        });
    }

    pub fn write(&self, o: Output, p: &Path) -> io::Result<()> {
        let dir = p.display().to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir)?;
        }
        let file_name = dir + "/" + &self.file_name;

        match o {
            Output::Json => self.write_json(file_name)?,
            Output::Txt => self.write_txt(file_name)?,
            Output::Csv => self.write_csv(file_name)?,
        }
        Ok(())
    }

    fn write_txt(&self, file_name: String) -> io::Result<()> {
        let file = File::create(file_name + ".stats.txt")?;
        let mut file = BufWriter::new(file);

        let mut v: Vec<(&Cow<str>, &(usize, f64))> = self.term_frequency.iter().collect();
        v.sort_unstable_by(|a, b| b.1 .0.cmp(&a.1 .0));

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
            .map(|(key, val)| format!("{:<22} {:<22} {:.3}%\n", key, val.0, val.1))
            .collect::<String>();

        let data = data + &s;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn write_csv(&self, file_name: String) -> io::Result<()> {
        let file = File::create(file_name + ".stats.csv")?;
        let mut file = BufWriter::new(file);

        let mut v: Vec<(&Cow<str>, &(usize, f64))> = self.term_frequency.iter().collect();
        v.sort_unstable_by(|a, b| b.1 .0.cmp(&a.1 .0));

        let data = format!(
            "FILE,UNIQUE,TOTAL\n{},{},{}\n\nWORD,FREQUENCY,PERCENT\n",
            self.file_name, self.unique, self.total
        );

        let s = v
            .iter()
            .map(|(key, val)| format!("{},{},{}\n", key, val.0, val.1))
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
                .map(|w| w.as_str().to_lowercase())
                .collect::<Vec<String>>()
        })
        .collect())
}

pub fn lemmanization(v: Vec<String>, lemma: &HashMap<String, String>) -> io::Result<Vec<String>> {
    let new_v: Vec<String> = v
        .iter()
        .map(|w| lemma.get(w).unwrap_or(w).to_owned())
        .collect();

    Ok(new_v)
}

pub fn exclude_junk(v: &[String], h: &HashSet<String>) -> io::Result<Vec<String>> {
    let new_v: Vec<String> = v
        .iter()
        .filter(|&w| !h.contains(&w.to_lowercase()))
        .cloned()
        .collect();

    Ok(new_v)
}

pub fn preload_lemma() -> io::Result<HashMap<String, String>> {
    let file = read_lines("./lemmas.txt")?;

    let l: Vec<(String, Vec<String>)> = file
        .filter_map(|line| line.ok())
        .map(|l| {
            let parts: Vec<String> = l.split_whitespace().map(|w| w.to_owned()).collect();
            let word = parts[0].clone();
            let lemmas = parts[1..].to_vec();
            (word, lemmas)
        })
        .collect();

    let mut lemma: HashMap<String, String> = HashMap::new();

    l.into_iter().for_each(|(word, vec)| {
        vec.into_iter().for_each(|l| {
            let _ = lemma.insert(l, word.clone());
        });
    });

    Ok(lemma)
}

pub fn preload_junk() -> io::Result<HashSet<String>> {
    let j_words = read_lines("./junk_words.txt")?;

    let mut j_set: HashSet<String> = HashSet::new();
    j_set.reserve(1000);
    j_words.filter_map(|line| line.ok()).for_each(|w| {
        let _ = j_set.insert(w);
    });
    Ok(j_set)
}

pub fn process_files(files: &Vec<PathBuf>, ops: Arc<Options>) {
    println!("PARSING {} FILES:\n", files.len());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !ops.skip_lemmanization {
            words = lemmanization(words, &lemma).unwrap_or(Vec::new());
        }

        if !ops.skip_junk_words {
            words = exclude_junk(&words, &junk).unwrap_or(Vec::new());
        }
        if words.is_empty() {
            println!(
                "{:<15}{} not exist | could not be read",
                "WARNING",
                f.file_name()
                    .expect("file name is invalid")
                    .to_string_lossy()
            );
            return;
        }

        println!(
            "{:<14} {}",
            "PARSING",
            f.file_name()
                .expect("file name is invalid")
                .to_string_lossy()
        );

        let st = Stats::new(&words, f);

        match st.write(ops.output_type, &ops.output_path) {
            Ok(_) => println!("{:<15}{}", "OK", st.file_name),
            Err(e) => println!("{:<15}{}:{}", "ERROR", st.file_name, e),
        }
    });
}

pub fn process_files_total(files: &Vec<PathBuf>, ops: Arc<Options>) {
    println!("PARSING {} FILES:\n", files.len());

    let words: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let st = Mutex::new(Stats::new_total());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut w = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !ops.skip_lemmanization {
            w = lemmanization(w, &lemma).unwrap_or(Vec::new());
        }

        if !ops.skip_junk_words {
            w = exclude_junk(&w, &junk).unwrap_or(Vec::new());
        }

        if w.is_empty() {
            println!(
                "{:<16}{} is empty | could not be read",
                "WARNING",
                f.file_name()
                    .expect("file name is invalid")
                    .to_string_lossy()
            );
            return;
        }

        println!(
            "{:<15} {}",
            "PARSING",
            f.file_name()
                .expect("file name is invalid")
                .to_string_lossy()
        );
        words.lock().unwrap().append(&mut w);
    });

    let words = words.lock().unwrap();
    let mut st = st.lock().unwrap();

    st.extend(words.as_slice());

    match st.write(ops.output_type, &ops.output_path) {
        Ok(_) => println!("\n\n{:<15}{}", "OK", st.file_name),
        Err(e) => println!("\n\n{:<15}{}:{}", "ERROR", st.file_name, e),
    }
}

impl FromStr for Output {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Output::Json),
            "txt" => Ok(Output::Txt),
            "csv" => Ok(Output::Csv),
            _ => Err("invalid output format"),
        }
    }
}
