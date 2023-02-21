use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::to_string;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rayon::prelude::*;

#[derive(Serialize, Debug)]
struct Stats {
    file_name: String,
    // file_path: PathBuf,
    length: usize,
    term_frequency: HashMap<String, (usize, f64)>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// type of the output file: json/csv/text
    #[clap(default_value = "json")]
    #[arg(short, long)]
    output: String,
    /// path to the output folder
    #[clap(default_value = "./")]
    #[arg(short, long)]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// provides term frequency
    TF {
        /// file for parsing
        #[arg(short, long)]
        file: Option<String>,
        /// dir for parsing
        #[arg(short, long, conflicts_with("file"))]
        dir: Option<String>,
    },
}

enum Output {
    Json,
    Text,
    Csv,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::TF { dir, file } => match (dir, file) {
            (_, Some(f)) => {
                let words = find_words_in_file(&f);
                let f = PathBuf::from(f);
                let st = Stats::build(&words, &f);
                let o = cli.output.parse::<Output>().unwrap();
                st.write(&o, &cli.path);
            }
            (Some(d), _) => {
                let files = find_files_in_dir(&d);
                let o = cli.output.parse::<Output>().unwrap();
                let path = &cli.path;

                files.par_iter().for_each(|f| {
                    let words = find_words_in_file(&f.display().to_string());
                    let st = Stats::build(&words, &f);

                    st.write(&o, &path);
                });
            }
            (None, None) => eprintln!("Provide file path or directory"),
        },
    }
}

impl Stats {
    fn build(words: &Vec<String>, file: &PathBuf) -> Self {
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
        let mut f = file.clone();
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

    fn write(&self, o: &Output, p: &str) {
        match o {
            Output::Json => self.write_json(p),
            Output::Text => self.write_text(p),
            Output::Csv => self.write_csv(p),
        }
    }

    fn write_text(&self, p: &str) {
        let dir = p.to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir).unwrap();
        }
        let file_name = dir + "/" + &self.file_name + ".stats.txt";

        let file = File::create(&file_name).unwrap();
        let mut file = BufWriter::new(file);

        let mut v: Vec<(String, (usize, f64))> = self.term_frequency.clone().into_iter().collect();
        v.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

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
            .map(|(key, val)| format!("{:<22} {:<22} {:.2}%\n", key, val.0, val.1))
            .collect::<String>();

        let data = data + &s;
        file.write_all(data.as_bytes()).unwrap();
    }

    fn write_csv(&self, p: &str) {
        let dir = p.to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir).unwrap();
        }
        let file_name = dir + "/" + &self.file_name + ".stats.csv";

        let file = File::create(&file_name).unwrap();
        let mut file = BufWriter::new(file);

        let mut v: Vec<(String, (usize, f64))> = self.term_frequency.clone().into_iter().collect();
        v.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        let data = format!(
            "FILE,LENGTH\n{},{}\n\nWORD,FREQUENCY,PERCENT\n",
            self.file_name, self.length
        );

        let s = v
            .iter()
            .map(|(key, val)| format!("{},{},{}\n", key, val.0, val.1))
            .collect::<String>();

        let data = data + &s;
        file.write_all(data.as_bytes()).unwrap();
    }

    fn write_json(&self, p: &str) {
        let dir = p.to_string() + "/result/";
        let path = std::path::Path::new(&dir);

        if !path.exists() {
            fs::create_dir(&dir).unwrap();
        }
        let file_name = dir + "/" + &self.file_name + ".stats.json";

        let file = File::create(&file_name).unwrap();
        let mut file = BufWriter::new(file);

        let s = to_string(&self).unwrap();

        file.write_all(s.as_bytes()).unwrap();
    }
}
fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn find_files_in_dir<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let entries = fs::read_dir(dir).unwrap();
    entries
        .filter_map(|entry| match entry {
            Ok(file) => match file.file_name().to_str() {
                Some(_) => {
                    if file.file_type().map_or(false, |t| t.is_file()) {
                        Some(file.path())
                    } else {
                        None
                    }
                }
                None => {
                    eprintln!("error in file {:?}", file);
                    None
                }
            },
            Err(e) => {
                eprintln!("error with filename  {:?}", e);
                None
            }
        })
        .collect()
}

fn find_words_in_file(file: &str) -> Vec<String> {
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
        .flatten()
        .collect()
}

impl FromStr for Output {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Output::Json),
            "text" => Ok(Output::Text),
            "csv" => Ok(Output::Csv),
            _ => Err("invalid output format".to_owned()),
        }
    }
}