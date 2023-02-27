use crate::helpers::{find_files_in_dir, process_files, process_files_total, Options, Output};
use clap::{Parser, Subcommand};
use env_logger::fmt::Color;
use env_logger::Env;
use log::error;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

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
    /// skip junk words
    #[arg(short, long)]
    junk: bool,
    /// skip lemmatization
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

pub fn run() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let mut style = buf.style();
            let color = match record.level() {
                log::Level::Error => Color::Red,
                log::Level::Warn => Color::Yellow,
                log::Level::Info => Color::Green,
                log::Level::Debug => Color::Blue,
                log::Level::Trace => Color::Magenta,
            };
            style.set_color(color).set_bold(true);
            writeln!(buf, "[{}] {}", style.value(record.level()), record.args())
        })
        .init();

    let cli = Arc::new(Cli::parse());
    let o = cli
        .output
        .parse::<Output>()
        .expect("can not parse cli command");

    let mut path = PathBuf::new();
    path.push(&cli.path);
    if !path.exists() {
        error!("path is not exist");
        std::process::exit(2);
    }

    let ops = Options {
        output_type: o,
        output_path: path,
        skip_junk_words: cli.junk,
        skip_lemmanization: cli.lemma,
    };
    let ops = Arc::new(ops);

    match &cli.command {
        Commands::Tf { dir, file } => match (dir, file) {
            (_, Some(f)) => {
                let files: Vec<PathBuf> = f.iter().map(PathBuf::from).collect();
                process_files(&files, ops)
            }
            (Some(d), _) => {
                let files = find_files_in_dir(d)?;
                process_files(&files, ops)
            }
            (None, None) => eprintln!("Provide file path or directory"),
        },
        Commands::Tft { dir } => {
            let files = find_files_in_dir(dir)?;
            process_files_total(&files, ops);
        }
    }
    Ok(())
}
