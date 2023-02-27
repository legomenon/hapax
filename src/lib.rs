use serde::Serialize;
use serde_json::to_string;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub mod cli;
pub mod helpers;
use crate::helpers::Output;

#[derive(Serialize, Debug)]
pub struct Stats<'a> {
    pub file_name: Cow<'a, str>,
    unique: usize,
    total: usize,
    term_frequency: HashMap<Cow<'a, str>, (usize, f64)>,
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
