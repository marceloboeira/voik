extern crate memmap;

use self::memmap::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

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

    /// Reader memory buffer
    reader: Mmap,

    /// Writer memory buffer
    writer: MmapMut,

    /// Base offset of the log on the global commit-log
    base_offset: usize,

    /// Current size of the file in bytes
    offset: usize,

    /// Max size of the file in bytes
    max_size: usize,
}

impl Log {
    /// Creates a new log file, from the scratch.
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

        let reader = unsafe { Mmap::map(&file).expect("failed to map the file") };
        let writer = unsafe { MmapMut::map_mut(&file).expect("failed to map the file") };

        Ok(Self {
            file: file,
            base_offset: base_offset,
            offset: offset,
            max_size: max_size,
            reader: reader,
            writer: writer,
        })
    }

    /// Return the offset of space left
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Check is a given buffer size fits in this log-file
    pub fn fit(&mut self, buffer_size: usize) -> bool {
        self.space_left() > buffer_size
    }

    /// Writes a buffer to the log-file
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len();
        if !self.fit(buffer_size) {
            return Err(Error::new(ErrorKind::Other, "No space left in the log"));
        }

        self.offset += buffer_size;
        (&mut self.writer[(self.offset - buffer_size)..=(self.offset)]).write(buffer)
    }

    /// Return the amount of space left
    fn space_left(&self) -> usize {
        self.max_size - self.offset
    }

    //TODO read from the segment mmap reader
    pub fn read_at(&mut self, offset: usize, size: usize) -> Result<Vec<u8>, Error> {
        // We seek the file to the moffset position
        self.file.seek(SeekFrom::Start(offset as u64))?;

        // load the buffer
        let mut buf = vec![0u8; size];
        self.file.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush_async()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commit_log::test::*;
    use std::fs;
    use std::path::Path;

    #[test]
    #[should_panic]
    fn it_fails_when_the_dir_is_invalid() {
        Log::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100).unwrap();
    }

    #[test]
    fn it_creates_a_new_file() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");

        Log::new(tmp_dir.clone(), 0, 10).unwrap();

        assert!(expected_file.as_path().exists());
    }

    #[test]
    fn it_writes_to_a_log() {
        let tmp_dir = tmp_file_path();
        let expected_file = tmp_dir.clone().join("00000000000000000000.log");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 20).unwrap();
        l.write(b"this-has-17-bytes").unwrap();

        // Notice that the log fills the void of the max_size with empty bytes
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("this-has-17-bytes\u{0}\u{0}\u{0}")
        );
    }

    #[test]
    #[should_panic]
    fn it_fails_to_write_to_a_full_log() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 15).unwrap();
        l.write(b"this-has-17-bytes").unwrap();
    }

    #[test]
    fn it_checks_if_buffer_fit() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut l = Log::new(tmp_dir.clone(), 0, 100).unwrap();
        l.write(b"this-has-17-bytes").unwrap();

        assert!(l.fit(20)); //  20 =< (100 - 17)
        assert!(l.fit(82)); //  82 =< (100 - 17)
        assert!(!l.fit(83)); //  83 =< (100 - 17)
        assert!(!l.fit(84)); //  84 =< (100 - 17)
        assert!(!l.fit(200)); // 200 =< (100 - 17)
    }
}
