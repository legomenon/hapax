use clap::{Parser, Subcommand};
use hapax::{find_files_in_dir, find_words_in_file, Stats};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use rayon::prelude::*;

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
    /// exclude junk words
    #[arg(short, long)]
    junk: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// provides term frequency TF
    Tf {
        /// file for parsing
        #[arg(short, long)]
        file: Option<String>,
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
    Text,
    Csv,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tf { dir, file } => match (dir, file) {
            (_, Some(f)) => {
                let words = find_words_in_file(&f)?;
                let f = PathBuf::from(f);
                if words.is_empty() {
                    println!(
                        "{:<15}{:?} is empty ",
                        "WARNING",
                        f.file_name().expect("file name is invalid")
                    );
                    std::process::exit(0);
                }
                println!(
                    "{:<14} {}",
                    "PARSING",
                    f.file_name()
                        .expect("file name is invalid")
                        .to_string_lossy()
                );
                let mut st = Stats::new(&words, &f);

                let o = cli
                    .output
                    .parse::<Output>()
                    .expect("can not parse cli command");
                if cli.junk {
                    st.exclude_junk("./junk_words.txt");
                }

                match st.write(&format!("{o:?}"), &cli.path) {
                    Ok(_) => println!("{:<15}{}", "OK", st.file_name),
                    Err(e) => println!("{:<15}{}:{}", "ERROR", st.file_name, e),
                }
            }
            (Some(d), _) => {
                let files = find_files_in_dir(d)?;
                let o = cli
                    .output
                    .parse::<Output>()
                    .expect("can not parse cli command");

                println!("Parsing {} files:\n\n", files.len());

                files.par_iter().for_each(|f| {
                    let words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());
                    if words.is_empty() {
                        println!(
                            "{:<15}{} is empty | can not read a file",
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

                    match st.write(&format!("{o:?}"), &cli.path) {
                        Ok(_) => println!("{:<15}{}", "OK", st.file_name),
                        Err(e) => println!("{:<15}{}:{}", "ERROR", st.file_name, e),
                    }
                });
            }
            (None, None) => eprintln!("Provide file path or directory"),
        },
        Commands::Tft { dir: tdir } => {
            let files = find_files_in_dir(tdir)?;
            let o = cli
                .output
                .parse::<Output>()
                .expect("can not parse cli command");

            println!("PARSING {} FILES:\n", files.len());
            let st = Mutex::new(Stats::new_total());
            files.par_iter().for_each(|f| {
                let words = find_words_in_file(&f.display().to_string()).unwrap_or(Vec::new());
                if words.is_empty() {
                    println!(
                        "{:<15}{:?} is empty | can not read a file",
                        "WARNING",
                        f.file_name()
                            .expect("file name is invalid")
                            .to_string_lossy()
                    );
                    return;
                }
                println!(
                    "{:<15} {:?}",
                    "PARSING",
                    f.file_name()
                        .expect("file name is invalid")
                        .to_string_lossy()
                );
                st.lock().unwrap().extend(&words);
            });
            let st = st.lock().unwrap();
            match st.write(&format!("{o:?}"), &cli.path) {
                Ok(_) => println!("\n\n{:<15}{}", "OK", st.file_name),
                Err(e) => println!("\n\n{:<15}{}:{}", "ERROR", st.file_name, e),
            }
        }
    }
    Ok(())
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
