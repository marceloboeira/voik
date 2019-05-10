extern crate rand;

use self::rand::{distributions, thread_rng, Rng};
use std::env;
use std::path::PathBuf;

pub fn random_hash() -> String {
    thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(30)
        .collect()
}

pub fn tmp_file_path() -> PathBuf {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push(random_hash());

    tmp_dir
}
