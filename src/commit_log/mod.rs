mod segment;

use self::segment::Segment;

use std::fs;
use std::path::PathBuf;

pub struct CommitLog {
    segment_size: i64,
    path: PathBuf,
    segments: Vec<Segment>, //TODO if too many Segments are created, and not "garbage collected", we have too many files opened
}

impl CommitLog {
    pub fn new(path: PathBuf, segment_size: i64) -> Result<Self, std::io::Error> {
        if !path.as_path().exists() {
            fs::create_dir_all(path.clone())?;
        }

        //TODO figure it out the segment starting in 0, should we truncate the file?
        let segments = vec![Segment::new(path.clone(), 0, segment_size)?];

        Ok(Self {
            path: path,
            segments: segments,
            segment_size: segment_size,
        })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        let buffer_size = buffer.len() as i64;

        //TODO find a better place for this
        //TODO what if the buffer_size is bigger than the segment_size? loops forever
        if buffer_size > self.active_segment().space_left() {
            let segments_size = self.segments.len() as i64;
            self.segments.push(Segment::new(
                self.path.clone(),
                segments_size,
                self.segment_size,
            )?);
        }
        self.active_segment().write(buffer)
    }

    fn active_segment(&mut self) -> &mut Segment {
        let index = self.segments.len() - 1;
        &mut self.segments[index]
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
                    match CommitLog::new(Path::new("/invalid/dir").to_path_buf(), 100) {
                        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied),
                        _ => assert!(false) // it should have failed
                    }
                }
            }

            describe "when the path is valid" {
                describe "and the folder does not exist" {
                    it "creates the folder" {
                        let tmp_dir = tmp_file_path();

                        CommitLog::new(tmp_dir.clone(), 100).unwrap();

                        assert!(tmp_dir.as_path().exists());
                    }
                }

                describe "and the folder already exists" {
                    it "does not recreate it" {
                        let mut tmp_dir = tmp_file_path();
                        fs::create_dir_all(tmp_dir.clone()).unwrap();

                        match CommitLog::new(tmp_dir, 100) {
                            Ok(_) => assert!(true),
                            _ => assert!(false),
                        };
                    }
                }
            }
        }

        describe "writing" {
            it "rotates segments when a segment if full" {
                let mut tmp_dir = tmp_file_path();

                let mut c = CommitLog::new(tmp_dir, 100).unwrap();
                c.write(b"this-should-have-about-80-bytes-but-not-really-sure-to-be-honest-maybe-it-doesn't").unwrap();

                match c.write(b"a-bit-more-than-20-bytes") {
                    Ok(_) => assert!(true),
                    Err(_) => assert!(false),
                }
            }
        }
    }
}
