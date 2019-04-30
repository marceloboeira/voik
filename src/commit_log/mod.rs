mod segment;

use self::segment::Segment;

use std::fs;
use std::path::PathBuf;

pub struct CommitLog {
    segment: Segment,
}

impl CommitLog {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if !path.as_path().exists() {
            fs::create_dir_all(path.clone())?;
        }

        Ok(Self {
            segment: Segment::new(path, 0)?,
        })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.segment.write(buffer)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate speculate;
    use self::rand::{distributions, thread_rng, Rng};
    use self::speculate::speculate;
    use commit_log::CommitLog;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    fn random_hash() -> String {
        thread_rng()
            .sample_iter(&distributions::Alphanumeric)
            .take(30)
            .collect()
    }

    fn tmp_file_path() -> PathBuf {
        let mut tmp_dir = env::temp_dir();
        tmp_dir.push(random_hash());

        tmp_dir
    }

    speculate! {
        describe "initializing" {
            describe "when the path is invalid" {
                it "fails accordingly" {
                    match CommitLog::new(Path::new("/invalid/dir").to_path_buf()) {
                        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied),
                        _ => assert!(false) // it should have failed
                    }
                }
            }

            describe "when the path is valid" {
                describe "and the folder does not exist" {
                    it "creates the folder" {
                        let tmp_dir = tmp_file_path();

                        CommitLog::new(tmp_dir.clone()).unwrap();

                        assert!(tmp_dir.as_path().exists());
                    }
                }

                describe "and the folder already exists" {
                    it "does not recreate it" {
                        let mut tmp_dir = tmp_file_path();
                        fs::create_dir_all(tmp_dir.clone()).unwrap();

                        match CommitLog::new(tmp_dir) {
                            Ok(_) => assert!(true),
                            _ => assert!(false),
                        };
                    }
                }
            }
        }
    }
}
