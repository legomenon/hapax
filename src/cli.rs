use crate::helpers::{find_files_in_dir, Options, Output};
use crate::process;
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
    /// type of the output file: [json|csv|txt]
    #[clap(default_value = "json")]
    #[arg(short, long)]
    output: String,
    /// path to the folder where result will be saved
    #[clap(default_value = "./")]
    #[arg(short, long)]
    path: String,
    /// log filter [info|warn|error]
    #[clap(default_value = "info")]
    #[arg(short, long)]
    log: String,
    /// if set skip junk words
    #[arg(short, long)]
    junk: bool,
    /// if set skip lemmatization
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
    let cli = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(cli.log))
        .format(|buf, record| {
            let mut style = buf.style();
            let color = match record.level() {
                log::Level::Error => Color::Red,
                log::Level::Warn => Color::Yellow,
                log::Level::Info => Color::Green,
                _ => Color::White,
            };
            style.set_color(color).set_bold(true);
            writeln!(buf, "{}: {}", style.value(record.level()), record.args())
        })
        .init();

    let o = cli.output.parse::<Output>()?;

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
                process::multi_threaded::for_each(&files, ops)
            }
            (Some(d), _) => {
                let files = find_files_in_dir(d)?;
                process::multi_threaded::for_each(&files, ops)
            }
            (None, None) => {
                error!("Provide file path or directory");
                std::process::exit(2);
            }
        },
        Commands::Tft { dir } => {
            let files = find_files_in_dir(dir)?;
            process::multi_threaded::total(&files, ops);
        }
    }
    Ok(())
}
