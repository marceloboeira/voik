extern crate rand;

use self::rand::{distributions, thread_rng, Rng};
use std::env;
use std::path::PathBuf;

/// Test Utilities
///

/// Random Hash for testing purposes
///
/// Returns a string with random alphanumerical characters
///
/// e.g.: 5V0eWCjkEyO8HEbhGbMuG8A2104Km
pub fn random_hash() -> String {
    thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(30)
        .collect()
}

/// Temp file path
///
/// Returns a unexistent temporary path for testing purposes
///
/// e.g.: /tmp/_/5V0eWCjkEyO8HEbhGbMuG8A2104Km
pub fn tmp_file_path() -> PathBuf {
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push(random_hash());

    tmp_dir
}
