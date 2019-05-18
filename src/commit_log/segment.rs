mod index;
mod log;

use self::index::Index;
use self::log::Log;
use std::io::Error;
use std::path::PathBuf;

/// Segment
///
/// A high-level wrapper for writing/reading records.
///
/// Every segment is composed of a log-file and an index, e.g.:
///
/// 00000000000011812312.log
/// 00000000000011812312.idx
///
/// The role of the segment is to manage writes to the logfile and ensure
/// the entries can be read later on by doing lookups on the index.
///
/// On every write, the segment writes an entry to the index
/// with the record's position and size, in the log-file, for later use.
///
/// The segment also manages the size of the log file, preventing it from
/// being written once it reaches the specified.
///
#[derive(Debug)]
pub struct Segment {
    /// Log file wrapper
    log: Log,

    /// Index file wrapper
    index: Index,

    /// Offset (Only used as name of the file at the moment)
    offset: usize,
}

impl Segment {
    /// Return a new segment
    pub fn new(
        path: PathBuf,
        offset: usize,
        max_log_size: usize,
        max_index_size: usize,
    ) -> Result<Self, Error> {
        Ok(Self {
            log: Log::new(path.clone(), offset, max_log_size)?,
            index: Index::new(path.clone(), offset, max_index_size)?,
            offset: offset,
        })
    }

    /// Return true if both the log and the index support the given buffer
    pub fn fit(&mut self, buffer_size: usize) -> bool {
        self.log.fit(buffer_size) && self.index.fit(1)
    }

    /// Write the buffer to the log, also making sure to create an index entry
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.index
            .write(index::Entry::new(self.log.offset(), buffer.len()))?;
        self.log.write(buffer)
    }

    /// Read the log at a given index offset
    pub fn read_at(&mut self, offset: usize) -> Result<Vec<u8>, Error> {
        let entry = self.index.read_at(offset)?;

        self.log.read_at(entry.offset, entry.size)
    }

    /// Flush both the index and the log to ensure persistence
    pub fn flush(&mut self) -> Result<(), Error> {
        self.index.flush()?;
        self.log.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commit_log::test::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    #[test]
    #[should_panic]
    fn it_fails_when_the_dir_is_invalid() {
        Segment::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100, 1000).unwrap();
    }

    #[test]
    fn it_creates_the_file_when_it_does_not_exist() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        Segment::new(tmp_dir.clone(), 0, 10, 1000).unwrap();

        assert!(expected_file.as_path().exists());
    }

    #[test]
    fn it_does_not_create_the_file_again_when_it_already_exists() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"2104").unwrap();

        Segment::new(tmp_dir.clone(), 0, 100, 1000).unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(
            fs::read_to_string(expected_file).unwrap()[0..4],
            String::from("2104")
        );
    }

    #[test]
    fn it_writes_to_a_new_segment_file() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 100, 1000).unwrap();
        s.write(b"2104").unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(
            fs::read_to_string(expected_file).unwrap()[0..4],
            String::from("2104")
        );
    }

    #[test]
    #[should_panic]
    fn it_fails_to_write_to_a_pre_existing_full_file() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"initial-content-18").unwrap(); // occupies 18 bytes

        let mut s = Segment::new(tmp_dir.clone(), 0, 20, 1000).unwrap(); // set the limit to 20 bytes
        s.write(b"1").unwrap(); // should be able to write 1 byte (total 19)

        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("initial-content-181")
        );

        // should not be able to write another 16 bytes
        s.write(b"this-should-error").unwrap();
    }

    #[test]
    #[should_panic]
    fn it_fails_when_writing_to_a_file_that_is_full() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 20, 1000).unwrap();
        s.write(b"this-has-17-bytes").unwrap();

        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("this-has-17-bytes")
        );

        s.write(b"this-should-error").unwrap();
    }

    #[test]
    fn it_reads_at_a_given_location() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let mut s = Segment::new(tmp_dir.clone(), 0, 100, 1000).unwrap();

        s.write(b"first-message").unwrap();
        s.write(b"second-message").unwrap();

        assert_eq!(s.read_at(0).unwrap(), b"first-message");
        assert_eq!(s.read_at(1).unwrap(), b"second-message");
    }
}
