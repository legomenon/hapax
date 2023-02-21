use clap::{Parser, Subcommand};
use hapax::{find_files_in_dir, find_words_in_file, Stats};
use std::path::PathBuf;
use std::str::FromStr;

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
#[derive(Debug)]
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

                st.write(&format!("{o:?}"), &cli.path);
            }
            (Some(d), _) => {
                let files = find_files_in_dir(&d);
                let o = cli.output.parse::<Output>().unwrap();
                let path = &cli.path;

                files.par_iter().for_each(|f| {
                    let words = find_words_in_file(&f.display().to_string());
                    let st = Stats::build(&words, &f);

                    st.write(&format!("{o:?}"), &path);
                });
            }
            (None, None) => eprintln!("Provide file path or directory"),
        },
    }
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
