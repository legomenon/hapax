use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

#[derive(Debug)]
struct Stats {
    term_frequency: HashMap<String, (usize, f64)>,
    file: String,
    length: usize,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "./")]
    output: Option<String>,
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::TF { dir, file } => match (dir, file) {
            (_, Some(f)) => {
                let words = find_words_in_file(&f);
                let st = get_stats(&words, &f);
                write_stats_in_file(&st);
            }
            (Some(d), _) => find_words_in_dir(&d),
            (None, None) => eprintln!("Provide file path or directory"),
        },
    }
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn find_words_in_dir<P: AsRef<Path>>(dir: P) {
    let entries = fs::read_dir(dir).unwrap();
    entries.for_each(|entry| match entry {
        Ok(file) => match file.file_name().to_str() {
            Some(f_name) => {
                if file.file_type().map_or(false, |t| t.is_file()) {
                    println!("{}", f_name);
                    let path = file.path().display().to_string();
                    let words = find_words_in_file(&path);
                    let st = get_stats(&words, f_name);
                    write_stats_in_dir(&st, &path);
                }
            }
            None => eprintln!("error in file {:?}", file),
        },
        Err(e) => eprintln!("error with filename  {:?}", e),
    });
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

fn write_stats_in_dir(s: &Stats, path: &str) {
    let file_name = &s.file;
    let dir = path.replace(file_name, "") + "result";
    let path = std::path::Path::new(&dir);

    if !path.exists() {
        fs::create_dir(&dir).unwrap();
    }
    let file_name = dir + "/" + &s.file;

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

fn write_stats_in_file(s: &Stats) {
    let file_name = s.file.clone() + "stats";
    println!("{}", &file_name);

    let file = File::create(&file_name).unwrap();
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
