extern crate memmap;
mod index;

use self::index::Index;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use self::memmap::{Mmap, MmapMut};

/// Segment
///
/// A wrapper for the log-file and the index
///
/// Every segment is composed of a logfile and an index, e.g.:
///
/// 00000000000011812312.log
/// 00000000000011812312.idx
///
/// The role of the segment is to manage writes to the logfile and ensure
/// the entries can be read later on by doing lookups on the index.
///
/// Every time a write happens, the segment writes an entry to the index
/// with the record's position and size, for later use.
///
/// The segment also manages the size of the log file, preventing it from
/// being written once it reaches the specified.
///
#[derive(Debug)]
pub struct Segment {
    /// File Descriptor
    file: File,

    /// Index file wrapper
    index: Index,

    /// Offset (Only used as name of the file at the moment)
    offset: usize,

    /// Current size of the file in bytes
    size: usize,

    /// Max size of the file in bytes
    max_size: usize,

    /// Reader memory buffer
    reader: Mmap,

    /// Reader memory buffer
    writer: MmapMut,
}

impl Segment {
    pub fn new(path: PathBuf, offset: usize, max_size: usize, max_index_size: usize) -> Result<Self, Error> {
        //TODO we never close this file, ...
        //TODO should we truncate the file instead of appending?
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.join(format!("{:020}.log", offset)))?; //TODO improve file formatting
        file.set_len(max_size as u64)?;
        //let size = file.metadata()?.len() as usize;

        //TODO Improve this..
        let index = Index::new(path.clone(), offset, max_index_size)?;

        let reader = unsafe { Mmap::map(&file).expect("failed to map the file") };
        let writer = unsafe { MmapMut::map_mut(&file).expect("failed to map the file") };

        Ok(Self {
            file: file,
            index: index,
            offset: offset,
            size: 0, //TODO improve this,
                     // it's zero to set the correct cursor, but if the file was opened it must be the size
            max_size: max_size,
            reader: reader,
            writer: writer,
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

        //TODO check index capacity before the attempt to write
        self.index
            .write(index::Entry::new(self.size, buffer_size))?;

        let from = self.size;
        let to = from + buffer_size;
        self.size += buffer_size;

        (&mut self.writer[from..=to]).write(buffer)
    }

    pub fn flush(&mut self) {
        self.writer.flush().unwrap();
        self.index.flush();
    }

    //TODO create a SegmentReader/SegmentWriter?
    #[allow(dead_code)]
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.file.read(buffer)
    }

    #[allow(dead_code)]
    pub fn read_at(&mut self, offset: usize) -> Result<Vec<u8>, Error> {
        // We get the size/position from index
        let e = self.index.read_at(offset)?;

        // We seek the file to the moffset position
        self.file.seek(SeekFrom::Start(e.offset as u64))?;

        // load the buffer
        let mut buf = vec![0u8; e.size];
        self.file.read_exact(&mut buf)?;

        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commit_log::test::*;
    use std::fs::{self, File};
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

    //TODO figure it out this when we start re-opening segments for Write
    //#[test]
    //fn it_writes_to_the_end_of_a_existing_segment_file() {
    //    let tmp_dir = tmp_file_path();
    //    let expected_file = tmp_dir.clone().join("00000000000000000000.log");

    //    fs::create_dir_all(tmp_dir.clone()).unwrap();

    //    let mut file = File::create(expected_file.clone()).unwrap();
    //    file.write(b"date-").unwrap();

    //    let mut s = Segment::new(tmp_dir.clone(), 0, 100, 1000).unwrap();
    //    s.write(b"2104").unwrap();

    //    assert!(expected_file.as_path().exists());
    //    assert_eq!(
    //        fs::read_to_string(expected_file).unwrap()[0..9],
    //        String::from("date-2104")
    //    );
    //}

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
    fn it_reads_the_whole_content_when_the_segment_has_content() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut file = File::create(expected_file.clone()).unwrap();
        file.write(b"2104").unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 20, 2000).unwrap();

        let mut buffer = [0; 4];
        s.read(&mut buffer).unwrap();

        assert_eq!(buffer, *b"2104");
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

    #[test]
    fn it_returns_the_space_left_on_the_segment() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut s = Segment::new(tmp_dir.clone(), 0, 100, 1000).unwrap();
        s.write(b"this-has-17-bytes").unwrap();

        assert_eq!(s.space_left(), 100 - 17)
    }
}
