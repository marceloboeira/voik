use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Segment {
    // File Descriptor
    file: File,

    // Offset (Only used as name of the file at the moment)
    offset: usize,

    // Current size of the file in bytes
    size: usize,

    // Max size of the file in bytes
    max_size: usize,
}

impl Segment {
    pub fn new(path: PathBuf, offset: usize, max_size: usize) -> Result<Self, Error> {
        //TODO we never close this file, ...
        //TODO should we truncate the file instead of appending?
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path.join(format!("{:020}.log", offset)))?; //TODO improve file formatting
        let size = file.metadata()?.len() as usize;

        Ok(Self {
            file: file,
            offset: offset,
            size: size,
            max_size: max_size,
        })
    }

    pub fn space_left(&self) -> usize {
        self.max_size - self.size
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len();

        if buffer_size > self.space_left() {
            return Err(Error::new(ErrorKind::Other, "No space left on the segment"));
        }

        self.size += buffer_size;
        self.file.write(buffer)
    }

    //TODO create a SegmentReader/SegmentWriter?
    #[allow(dead_code)]
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.file.read(buffer)
    }

    #[allow(dead_code)]
    pub fn read_at(&mut self, buffer: &mut [u8], offset: usize) -> Result<usize, Error> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read(buffer)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use self::rand::{distributions, thread_rng, Rng};
    use commit_log::segment::Segment;
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

    #[test]
    fn test_initializing_when_the_path_is_invalid_it_fails_accordingly() {
        match Segment::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100) {
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            _ => assert!(false), // it should have failed
        }
    }

    #[test]
    fn test_initializing_when_the_path_and_the_file_does_not_exist_creates_the_file() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        Segment::new(tmp_dir.clone(), 0, 10).unwrap();

        assert!(expected_file.as_path().exists());
    }

    #[test]
    fn test_initializing_when_the_path_and_the_file_already_exists_does_not_create_or_override() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"2104").unwrap();

        Segment::new(tmp_dir.clone(), 0, 100).unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("2104")
        );
    }

    #[test]
    fn test_writing_when_segment_is_not_full_and_the_file_does_not_exist_writes_to_a_new_file() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();
        s.write(b"2104").unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("2104")
        );
    }

    #[test]
    fn test_writing_when_segment_is_not_full_and_the_file_already_exists_appends_to_the_existing_file(
    ) {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"date-").unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();
        s.write(b"2104").unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("date-2104")
        );
    }

    #[test]
    fn test_writing_when_segment_is_not_full_and_the_file_already_exists_respects_the_file_size() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"initial-content-18").unwrap(); // occupies 18 bytes

        let mut s = Segment::new(tmp_dir.clone(), 0, 20).unwrap(); // set the limit to 20 bytes
        s.write(b"1").unwrap(); // should be able to write 1 byte (total 19)

        // should not be able to write another 16 bytes
        match s.write(b"this-should-error") {
            Ok(_) => assert!(false), // it should have errored
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::Other);
            }
        }

        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("initial-content-181")
        );
    }

    #[test]
    fn test_writing_when_segment_is_full_it_errors() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 20).unwrap();
        s.write(b"this-has-17-bytes").unwrap();

        match s.write(b"this-should-error") {
            Ok(_) => assert!(false), // it should have errored
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::Other);
            }
        }

        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("this-has-17-bytes")
        );
    }

    #[test]
    fn test_reading_when_segment_has_content_reads() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"2104").unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 20).unwrap();

        let mut buffer = [0; 4];
        s.read(&mut buffer).unwrap();

        assert_eq!(buffer, *b"2104");
    }

    #[test]
    fn test_reading_when_segment_has_content_read_at_a_specific_location() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();

        s.write(b"first-message").unwrap();
        s.write(b"second-message").unwrap();

        let mut buffer1 = [0; 13];
        s.read_at(&mut buffer1, 0).unwrap();

        let mut buffer2 = [0; 14];
        s.read_at(&mut buffer2, 13).unwrap();

        assert_eq!(buffer1, *b"first-message");
        assert_eq!(buffer2, *b"second-message");
    }

    #[test]
    fn test_space_lefta_mount_of_space_left_on_the_segment() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let mut s = Segment::new(tmp_dir.clone(), 0, 100).unwrap();

        s.write(b"this-has-17-bytes").unwrap();

        assert_eq!(s.space_left(), 100 - 17)
    }
}
