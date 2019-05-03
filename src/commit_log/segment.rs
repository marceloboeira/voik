use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Segment {
    file: File,
    offset: i64,
    size: i64,
    max_bytes: i64,
}

impl Segment {
    pub fn new(path: PathBuf, offset: i64, max_bytes: i64) -> Result<Self, Error> {
        //TODO we never close this file, ...
        //TODO should we truncate the file instead of appending?
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path.join(format!("ll-{}", offset)))?; //TODO improve file formatting

        Ok(Self {
            file: file,
            offset: offset,
            size: 0, //TODO should it be zero?
            max_bytes: max_bytes,
        })
    }

    pub fn space_left(&self) -> i64 {
        self.max_bytes - self.size
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len() as i64;

        if buffer_size > self.space_left() {
            return Err(Error::new(ErrorKind::Other, "No space left on the segment"));
        }

        self.size += buffer_size;
        self.file.write(buffer)
    }

    //TODO create a SegmentReader/SegmentWriter?
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.file.read(buffer)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate speculate;
    use self::rand::{distributions, thread_rng, Rng};
    use self::speculate::speculate;
    use commit_log::segment::Segment;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{self, Path, PathBuf};

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
                    match Segment::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100) {
                        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
                        _ => assert!(false) // it should have failed
                    }
                }
            }

            describe "when the path is valid" {
                describe "and the file does not exist" {
                    it "creates a new file" {
                        let tmp_dir = tmp_file_path();
                        fs::create_dir_all(tmp_dir.clone()).unwrap();
                        let expected_file = tmp_dir.clone().join("ll-0");

                        Segment::new(tmp_dir.clone(), 0, 10).unwrap();

                        assert!(expected_file.as_path().exists());
                    }
                }

                describe "and the file already exists" {
                    it "does not recreate it" {
                        let tmp_dir = tmp_file_path();
                        let expected_file = tmp_dir.clone().join("ll-0");

                        fs::create_dir_all(tmp_dir.clone()).unwrap();

                        let mut file = File::create(expected_file.clone()).unwrap();
                        file.write(b"2104").unwrap();

                        Segment::new(tmp_dir.clone(), 0, 100).unwrap();

                        assert!(expected_file.as_path().exists());
                        assert_eq!(fs::read_to_string(expected_file).unwrap(), String::from("2104"));
                    }
                }
            }
        }

        describe "writing" {
            describe "when the buffer is not full" {
                describe "and the file does not exist" {
                    it "writes to a new file" {
                        let tmp_dir = tmp_file_path();
                        let expected_file = tmp_dir.clone().join("ll-0");

                        fs::create_dir_all(tmp_dir.clone()).unwrap();

                        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();
                        s.write(b"2104").unwrap();

                        assert!(expected_file.as_path().exists());
                        assert_eq!(fs::read_to_string(expected_file).unwrap(), String::from("2104"));
                    }
                }

                describe "and the file already exists" {
                    it "appends to the existing file" {
                        let tmp_dir = tmp_file_path();
                        let expected_file = tmp_dir.clone().join("ll-0");

                        fs::create_dir_all(tmp_dir.clone()).unwrap();

                        let mut file = File::create(expected_file.clone()).unwrap();
                        file.write(b"date-").unwrap();

                        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();
                        s.write(b"2104").unwrap();

                        assert!(expected_file.as_path().exists());
                        assert_eq!(fs::read_to_string(expected_file).unwrap(), String::from("date-2104"));
                    }
                }
            }

            describe "when the buffer is full" {
                it "errors" {
                    let tmp_dir = tmp_file_path();
                    let expected_file = tmp_dir.clone().join("ll-0");
                    fs::create_dir_all(tmp_dir.clone()).unwrap();

                    let mut s = Segment::new(tmp_dir.clone(), 0, 20).unwrap();
                    s.write(b"this-has-17-bytes").unwrap();

                    match s.write(b"this-should-error") {
                        Ok(_) => assert!(false), // it should have errored
                        Err(e) => {
                            assert_eq!(e.kind(), std::io::ErrorKind::Other);
                        }
                    }


                    assert_eq!(fs::read_to_string(expected_file).unwrap(), String::from("this-has-17-bytes"));
                }
            }
        }

        describe "reading" {
            it "reads the content content" {
                let tmp_dir = tmp_file_path();
                let expected_file = tmp_dir.clone().join("ll-0");
                fs::create_dir_all(tmp_dir.clone()).unwrap();

                let mut file = File::create(expected_file.clone()).unwrap();
                file.write(b"2104").unwrap();

                let mut s = Segment::new(tmp_dir.clone(), 0, 20).unwrap();

                let mut buffer = [0; 4];
                s.read(&mut buffer).unwrap();

                assert_eq!(buffer, *b"2104");
            }
        }

        describe "space left" {
            it "returns the amount of space left on the segment" {
                let tmp_dir = tmp_file_path();
                fs::create_dir_all(tmp_dir.clone()).unwrap();
                let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();

                s.write(b"this-has-17-bytes").unwrap();

                assert_eq!(s.space_left(), 100-17)
            }
        }
    }
}
