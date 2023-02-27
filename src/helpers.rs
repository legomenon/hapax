use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::Stats;

use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
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

pub fn lemmatization(v: Vec<String>, lemma: &HashMap<String, String>) -> io::Result<Vec<String>> {
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
    let file = read_lines("./lemmatization")?;

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
    let j_words = read_lines("./junk_words")?;

    let mut j_set: HashSet<String> = HashSet::new();
    j_set.reserve(1000);
    j_words.filter_map(|line| line.ok()).for_each(|w| {
        let _ = j_set.insert(w);
    });
    Ok(j_set)
}

pub fn process_files(files: &Vec<PathBuf>, ops: Arc<Options>) {
    info!("parsing {} files", files.len());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !ops.skip_lemmanization {
            words = lemmatization(words, &lemma).unwrap_or(Vec::new());
        }

        if !ops.skip_junk_words {
            words = exclude_junk(&words, &junk).unwrap_or(Vec::new());
        }
        if words.is_empty() {
            warn!(
                "{}: is empty or could not be read",
                f.file_name()
                    .expect("file name is invalid")
                    .to_string_lossy()
            );
            return;
        }

        info!(
            "{}",
            f.file_name()
                .expect("file name is invalid")
                .to_string_lossy()
        );

        let st = Stats::new(&words, f);

        if let Err(e) = st.write(ops.output_type, &ops.output_path) {
            error!("{}: {}", st.file_name, e);
        }
    });
}

pub fn process_files_total(files: &Vec<PathBuf>, ops: Arc<Options>) {
    info!("parsing {} files:", files.len());
    let words: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let st = Mutex::new(Stats::new_total());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut w = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !ops.skip_lemmanization {
            w = lemmatization(w, &lemma).unwrap_or(Vec::new());
        }

        if !ops.skip_junk_words {
            w = exclude_junk(&w, &junk).unwrap_or(Vec::new());
        }

        if w.is_empty() {
            warn!(
                "{}: is empty or could not be read",
                f.file_name()
                    .expect("file name is invalid")
                    .to_string_lossy()
            );
            return;
        }

        info!(
            "{}",
            f.file_name()
                .expect("file name is invalid")
                .to_string_lossy()
        );

        words.lock().unwrap().append(&mut w);
    });

    let words = words.lock().unwrap();
    let mut st = st.lock().unwrap();

    st.extend(words.as_slice());

    if let Err(e) = st.write(ops.output_type, &ops.output_path) {
        error!("{}: {}", st.file_name, e);
    }
}

impl FromStr for Output {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Output::Json),
            "txt" => Ok(Output::Txt),
            "csv" => Ok(Output::Csv),
            _ => Err(Error::new(ErrorKind::Other, "invalid output option")),
        }
    }
}
