use clap::{Parser, Subcommand};
use hapax::{
    exclude_junk, find_files_in_dir, find_words_in_file, lemmanization, preload_junk,
    preload_lemma, Output, Stats,
};
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// type of the output file: json/csv/txt
    #[clap(default_value = "json")]
    #[arg(short, long)]
    output: String,
    /// path to the output folder
    #[clap(default_value = "./")]
    #[arg(short, long)]
    path: String,
    /// exclude junk words [default: false]
    #[arg(short, long)]
    junk: bool,
    /// words lemmanization [default: false]
    #[arg(short, long)]
    lemma: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// provides term frequency TF
    Tf {
        /// files for parsing
        #[arg(short, long,value_parser, num_args = 1.., value_delimiter = ' ')]
        file: Option<Vec<String>>,
        /// dir for parsing
        #[arg(short, long, conflicts_with("file"))]
        dir: Option<String>,
    },
    /// provides term frequency for all documents words comb
    Tft {
        /// dir for parsing
        #[arg(short, long)]
        dir: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Arc::new(Cli::parse());

    match &cli.command {
        Commands::Tf { dir, file } => match (dir, file) {
            (_, Some(f)) => {
                let files: Vec<PathBuf> = f.iter().map(PathBuf::from).collect();
                process_files(&files, cli)
            }
            (Some(d), _) => {
                let files = find_files_in_dir(d)?;
                process_files(&files, cli)
            }
            (None, None) => eprintln!("Provide file path or directory"),
        },
        Commands::Tft { dir: tdir } => {
            let files = find_files_in_dir(tdir)?;
            process_files_total(&files, cli);
        }
    }
    Ok(())
}

fn process_files(files: &Vec<PathBuf>, cli: Arc<Cli>) {
    let cli = cli;
    let o = cli
        .output
        .parse::<Output>()
        .expect("can not parse cli command");

    println!("PARSING {} FILES:\n", files.len());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !cli.lemma {
            words = lemmanization(words, &lemma).unwrap_or(Vec::new());
        }

        if !cli.junk {
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

        match st.write(o, &cli.path) {
            Ok(_) => println!("{:<15}{}", "OK", st.file_name),
            Err(e) => println!("{:<15}{}:{}", "ERROR", st.file_name, e),
        }
    });
}

fn process_files_total(files: &Vec<PathBuf>, cli: Arc<Cli>) {
    let cli = cli;
    let o = cli
        .output
        .parse::<Output>()
        .expect("can not parse cli command");

    println!("PARSING {} FILES:\n", files.len());

    let words: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let st = Mutex::new(Stats::new_total());
    let junk = Arc::new(preload_junk().unwrap());
    let lemma = Arc::new(preload_lemma().unwrap());

    files.par_iter().for_each(|f| {
        let mut w = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !cli.lemma {
            w = lemmanization(w, &lemma).unwrap_or(Vec::new());
        }

        if !cli.junk {
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

    match st.write(o, &cli.path) {
        Ok(_) => println!("\n\n{:<15}{}", "OK", st.file_name),
        Err(e) => println!("\n\n{:<15}{}:{}", "ERROR", st.file_name, e),
    }
    drop(st);
}
