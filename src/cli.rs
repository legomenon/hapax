use clap::{Parser, Subcommand};
use hapax::{find_files_in_dir, find_words_in_file, lemmanization, Stats};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

#[derive(Parser, Clone)]
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
#[derive(Debug)]
enum Output {
    Json,
    Txt,
    Csv,
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
    println!("PARSING {} FILES:\n", files.len());
    files.par_iter().for_each(|f| {
        let mut words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !cli.lemma {
            words = lemmanization(words).unwrap_or(Vec::new());
        }
        if words.is_empty() {
            println!(
                "{:<15}{} not exist | can not be read",
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
        let mut st = Stats::new(&words, f);
        if !cli.junk {
            st.exclude_junk("./junk_words.txt");
        }
        let o = cli
            .output
            .parse::<Output>()
            .expect("can not parse cli command");

        match st.write(&format!("{o:?}"), &cli.path) {
            Ok(_) => println!("{:<15}{}", "OK", st.file_name),
            Err(e) => println!("{:<15}{}:{}", "ERROR", st.file_name, e),
        }
    });
}

fn process_files_total(files: &Vec<PathBuf>, cli: Arc<Cli>) {
    let cli = cli;

    println!("PARSING {} FILES:\n", files.len());
    let st = Mutex::new(Stats::new_total());
    files.par_iter().for_each(|f| {
        let mut words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());

        if !cli.lemma {
            words = lemmanization(words).unwrap_or(Vec::new());
        }

        if words.is_empty() {
            println!(
                "{:<16}{} is empty | can not be read",
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
        st.lock().unwrap().extend(&words);
    });

    let mut st = st.lock().unwrap();
    if !cli.junk {
        st.exclude_junk("./junk_words.txt");
    }
    let o = cli
        .output
        .parse::<Output>()
        .expect("can not parse cli command");
    match st.write(&format!("{o:?}"), &cli.path) {
        Ok(_) => println!("\n\n{:<15}{}", "OK", st.file_name),
        Err(e) => println!("\n\n{:<15}{}:{}", "ERROR", st.file_name, e),
    }
}

impl FromStr for Output {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Output::Json),
            "text" => Ok(Output::Txt),
            "csv" => Ok(Output::Csv),
            _ => Err("invalid output format".to_owned()),
        }
    }
}
