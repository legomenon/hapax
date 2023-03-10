use std::{
    collections::HashSet,
    io::{self, Error, ErrorKind},
    path::Path,
};

use log::{info, warn};
use rustc_hash::FxHashMap;

use crate::helpers::{exclude_junk, find_words_in_file, lemmatization, Options};

#[allow(unused)]

pub fn file(
    f: &Path,
    ops: &Options,
    lemma: &FxHashMap<String, String>,
    junk: &HashSet<String>,
) -> io::Result<Vec<String>> {
    let mut words = find_words_in_file(&f.to_string_lossy()).unwrap_or(Vec::new());

    if !ops.skip_lemmanization {
        words = lemmatization(&words, lemma).unwrap_or(Vec::new());
    }
    if !ops.skip_junk_words {
        words = exclude_junk(&words, junk).unwrap_or(Vec::new());
    }  
    if words.is_empty() {
        warn!(
            "{}: is empty or could not be read",
            f.file_name()
                .expect("file name is invalid")
                .to_string_lossy()
        );
        return Err(Error::new(ErrorKind::Other, "invalid output option"));
    }

    info!(
        "{}",
        f.file_name()
            .expect("file name is invalid")
            .to_string_lossy()
    );
    Ok(words)
}

pub mod multi_threaded {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use log::{error, info};
    use rayon::prelude::*;

    use crate::{
        helpers::{preload_junk, preload_lemma, Options},
        process, Stats,
    };
    pub fn for_each(files: &Vec<PathBuf>, ops: Arc<Options>) {
        info!("parsing {} files", files.len());

        let junk =
            Arc::new(preload_junk(ops.skip_junk_words).expect("could not preload junk words list"));
        let lemma = Arc::new(
            preload_lemma(ops.skip_lemmanization).expect("could not preload lemmatization list"),
        );

        files.par_iter().for_each(|f| {
            let words = match process::file(f, &ops, &lemma, &junk) {
                Ok(w) => w,
                Err(_) => return,
            };

            let st = Stats::new(&words, f);

            if let Err(e) = st.write(ops.output_type, &ops.output_path) {
                error!("{}: {}", st.file_name, e);
            }
        });
    }

    pub fn total(files: &Vec<PathBuf>, ops: Arc<Options>) {
        info!("parsing {} files:", files.len());
        let words: Mutex<Vec<String>> = Mutex::new(Vec::new());
        let st = Mutex::new(Stats::new_total());

        let junk =
            Arc::new(preload_junk(ops.skip_junk_words).expect("could not preload junk words list"));
        let lemma = Arc::new(
            preload_lemma(ops.skip_lemmanization).expect("could not preload lemmatization list"),
        );

        files.par_iter().for_each(|f| {
            let mut w = match process::file(f, &ops, &lemma, &junk) {
                Ok(w) => w,
                Err(_) => return,
            };

            words.lock().unwrap().append(&mut w);
        });

        let words = words.lock().unwrap();
        let mut st = st.lock().unwrap();

        st.extend(words.as_slice());

        if let Err(e) = st.write(ops.output_type, &ops.output_path) {
            error!("{}: {}", st.file_name, e);
        }
    }
}

pub mod single_threaded {
    use std::path::PathBuf;

    use log::{error, info};

    use crate::{
        helpers::{preload_junk, preload_lemma, Options},
        process, Stats,
    };
    pub fn for_each(files: &Vec<PathBuf>, ops: Options) {
        info!("parsing {} files", files.len());
        let junk = preload_junk(ops.skip_junk_words).unwrap();
        let lemma = preload_lemma(ops.skip_lemmanization).unwrap();

        files.iter().for_each(|f| {
            let words = match process::file(f, &ops, &lemma, &junk) {
                Ok(w) => w,
                Err(_) => return,
            };
            let st = Stats::new(&words, f);
            if let Err(e) = st.write(ops.output_type, &ops.output_path) {
                error!("{}: {}", st.file_name, e);
            }
        });
    }

    pub fn total(files: &Vec<PathBuf>, ops: Options) {
        info!("parsing {} files:", files.len());
        let mut words: Vec<String> = Vec::new();
        let mut st = Stats::new_total();

        let junk = preload_junk(ops.skip_junk_words).unwrap();
        let lemma = preload_lemma(ops.skip_lemmanization).unwrap();

        files.iter().for_each(|f| {
            let mut w = match process::file(f, &ops, &lemma, &junk) {
                Ok(w) => w,
                Err(_) => return,
            };
            words.append(&mut w);
        });

        st.extend(words.as_slice());

        if let Err(e) = st.write(ops.output_type, &ops.output_path) {
            error!("{}: {}", st.file_name, e);
        }
    }
}
