use hapax::cli::run;
use log::error;

fn main() {
    if let Err(e) = run() {
        error!("{}", e);
    }
}
