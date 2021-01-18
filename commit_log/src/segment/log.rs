extern crate memmap;

use self::memmap::MmapMut;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    Io(io::Error),
    NoSpaceLeft,
    InvalidIndex,
}

/// Log
///
/// A wrapper for the log-file, where data is stored.
///
/// The log is an append-only file where data is stored sequencially.
///
/// Each new record is basically places sequencially on the byte-stream.
///
/// e.g.:
///                          current cursor
///                                 ^
/// |-------------------------------|
/// | record 0  |  record 1  |  ... |----> time
/// |-------------------------------|
///
/// Important:
///   Neither reads nor writes to the log are directly triggering disk-level actions.
///   Both operations are being intermediated by a memory-mapping buffers, managed by
///   the OS and operated by public/privated methods of this struct.
///
#[derive(Debug)]
pub struct Log {
    /// File Descriptor
    file: File,

    /// Memory buffer
    mmap: MmapMut,

    /// Base offset of the log on the global commit-log
    base_offset: usize,

    /// Current size of the file in bytes
    offset: usize,

    /// Max size of the file in bytes
    max_size: usize,
}

impl Log {
    /// Create a new log file, from the scratch.
    pub fn new(path: PathBuf, base_offset: usize, max_size: usize) -> Result<Self, Error> {
        //TODO we never close this file, ...
        //TODO should we truncate the file instead of appending?
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.join(format!("{:020}.log", base_offset)))?; //TODO improve file formatting

        file.set_len(max_size as u64)?;

        //TODO improve this, it's zero to set the correct cursor, but if the file was opened it must be the size
        //let size = file.metadata()?.len() as usize;
        let offset = 0;

        let mmap = unsafe { MmapMut::map_mut(&file).expect("failed to map the file") };

        Ok(Self {
            file,
            base_offset,
            offset,
            max_size,
            mmap,
        })
    }

    /// Return the offset of space left
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Check is a given buffer size fits in this log-file
    pub fn fit(&mut self, buffer_size: usize) -> bool {
        (self.max_size - self.offset) >= buffer_size
    }

    /// Flush to ensure the content on memory is written to the file
    pub fn flush(&mut self) -> Result<(), Error> {
        self.mmap.flush_async()?;
        Ok(())
    }

    /// Write a buffer to the log-file
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len();
        if !self.fit(buffer_size) {
            return Err(Error::NoSpaceLeft);
        }

        self.offset += buffer_size;
        let size = (&mut self.mmap[(self.offset - buffer_size)..(self.offset)]).write(buffer)?;
        Ok(size)
    }

    //TODO read from the segment mmap reader
    /// Read the log on a specific position
    pub fn read_at(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        if (offset + size) > self.mmap.len() {
            return Err(Error::InvalidIndex);
        }

        Ok(&self.mmap[(offset)..(offset + size)])
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_create() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        let l = Log::new(tmp_dir.clone(), 0, 10).unwrap();

        assert!(expected_file.as_path().exists());
        assert_eq!(l.offset(), 0); // should be zero when creating
    }

    #[test]
    #[should_panic]
    fn test_invalid_create() {
        Log::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100).unwrap();
    }

    #[test]
    fn test_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 20).unwrap();
        l.write(b"this-has-17-bytes").unwrap();
        l.flush().unwrap(); // flush the file to ensure content is gonna be written

        // Notice that the log file is truncated with empty bytes
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("this-has-17-bytes\u{0}\u{0}\u{0}")
        );

        assert_eq!(l.offset(), 17); // should update the offset when writing
    }

    #[test]
    #[should_panic]
    fn test_invalid_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 15).unwrap();
        // buffer is bigger than log size
        l.write(b"this-has-17-bytes").unwrap();
    }

    #[test]
    fn test_record_fit() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 100).unwrap();
        l.write(b"this-has-17-bytes").unwrap();

        assert!(l.fit(20)); //  20 =< (100 - 17)
        assert!(l.fit(82)); //  82 =< (100 - 17)
        assert!(l.fit(83)); //  83 =< (100 - 17)
        assert!(!l.fit(84)); //  84 =< (100 - 17)
        assert!(!l.fit(200)); // 200 =< (100 - 17)
    }

    #[test]
    fn test_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 50).unwrap();
        l.write(b"hello-from-the-other-side").unwrap();
        l.flush().unwrap();

        assert_eq!(l.read_at(0, 25).unwrap(), b"hello-from-the-other-side");
        assert_eq!(l.read_at(1, 24).unwrap(), b"ello-from-the-other-side");
    }

    #[test]
    #[should_panic]
    fn test_invalid_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 50).unwrap();
        l.write(b"hello-from-the-other-side").unwrap();

        l.read_at(51, 20).unwrap(); // should fail since the position is invalid
    }
}
