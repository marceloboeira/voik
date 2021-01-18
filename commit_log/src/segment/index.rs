extern crate memmap;

use self::memmap::MmapMut;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::num;
use std::path::PathBuf;
use std::str::from_utf8_unchecked;

use derive_more::From;
use std::fmt;

#[derive(Debug, From)]
pub enum Error {
    Io(io::Error),
    Num(num::ParseIntError),
    NoSpaceLeft,
    InvalidIndex,
}

/// Index
///
/// A wrapper for writing/reading entries to the index file.
///
/// Every log has an index companion, e.g.:
///
/// 00000000000011812312.log
/// 00000000000011812312.idx
///
/// e.g.:
///                          current cursor
///                                 ^
/// |-------------------------------|
/// | offset-size | offset-size |...|----> time
/// |-------------------------------|
///
/// The role of the index is to provide pointers to records in the log file.
/// Each entry of the index is 20 bytes long, 10 bytes are used for the offset address of the
/// record in the log file, the other 10 bytes for the size of the record.
///
/// e.g.:
/// 00000001000000000020
///
/// is actually,
/// 000000010 -> offset
/// 000000020 -> size
///
/// Important:
///   Neither reads nor writes to the index are directly triggering disk-level actions.
///   Both operations are being intermediated by a memory-mapping buffers, managed by
///   the OS and operated by public/privated methods of this struct.
///
#[derive(Debug)]
pub struct Index {
    /// File Descriptor
    file: File,

    /// Memory map buffer
    mmap: MmapMut,

    /// Max size of the index
    max_size: usize,

    /// Base offset of the index across the commit-log
    base_offset: usize,

    /// Current size of the index in bytes (used as a cursor when writing)
    offset: usize,
}

/// Amount of bytes for each entry on the index
const ENTRY_SIZE: usize = 20;

impl Index {
    /// Create a new Index / reads the existing Index
    pub fn new(path: PathBuf, base_offset: usize, max_size: usize) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.join(format!("{:020}.idx", base_offset)))?; //TODO improve file formatting

        file.set_len(max_size as u64).unwrap(); //TODO Should we avoid truncating when size is given?

        let mmap = unsafe { MmapMut::map_mut(&file).expect("failed to map the file") };

        Ok(Self {
            base_offset,
            max_size,
            offset: 0,
            file,
            mmap,
        })
    }

    /// Check if the given amount of entries fit
    pub fn fit(&mut self, entry: usize) -> bool {
        self.max_size >= (self.offset + (entry * ENTRY_SIZE))
    }

    /// Write an entry to the index
    pub fn write(&mut self, entry: Entry) -> Result<usize, Error> {
        if !self.fit(1) {
            return Err(Error::NoSpaceLeft);
        }
        self.offset += ENTRY_SIZE;

        let size = (&mut self.mmap[(self.offset - ENTRY_SIZE)..(self.offset)])
            .write(entry.to_string().as_bytes())?;
        Ok(size)
    }

    /// Flush to ensure the content on memory is written to the file
    pub fn flush(&mut self) -> Result<(), Error> {
        self.mmap.flush_async()?;
        Ok(())
    }

    /// Read an entry from the index
    pub fn read_at(&self, offset: usize) -> Result<Entry, Error> {
        let real_offset = offset * ENTRY_SIZE;

        if (real_offset + ENTRY_SIZE) >= self.mmap.len() {
            return Err(Error::InvalidIndex);
        }

        let buffer = &self.mmap[real_offset..(real_offset + ENTRY_SIZE)];

        let position = unsafe {
            let position = from_utf8_unchecked(&buffer[0..(ENTRY_SIZE / 2)]).parse()?;
            position
        };

        let size = unsafe {
            let size = from_utf8_unchecked(&buffer[(ENTRY_SIZE / 2)..ENTRY_SIZE]).parse()?;
            size
        };

        Ok(Entry::new(position, size))
    }
}

/// Entry
///
/// A tuple to store the offset and size of a record present in the logfile
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// Offset of the record
    pub offset: usize,

    /// Size of the record
    pub size: usize,
}

impl Entry {
    /// Return a new entry reference
    pub fn new(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:010}{:010}", self.offset, self.size)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    /// Entry tests
    #[test]
    fn test_entry_to_string() {
        let e0 = Entry::new(0, 0);
        let e1 = Entry::new(1, 2);
        let e2 = Entry::new(1521230, 91028317);

        assert_eq!(e0.to_string(), "00000000000000000000".to_string());
        assert_eq!(e1.to_string(), "00000000010000000002".to_string());
        assert_eq!(e2.to_string(), "00015212300091028317".to_string());
    }

    /// Index tests
    #[test]
    fn test_create() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.idx");

        Index::new(tmp_dir.clone(), 0, 10).unwrap();

        assert!(expected_file.as_path().exists());
    }

    #[test]
    #[should_panic]
    fn test_invalid_create() {
        Index::new(Path::new("/invalid/dir/").to_path_buf(), 0, 100).unwrap();
    }

    #[test]
    fn test_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let expected_file = tmp_dir.clone().join("00000000000000000000.idx");
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut i = Index::new(tmp_dir.clone(), 0, 25).unwrap();
        i.write(Entry::new(0, 10)).unwrap();
        i.flush().unwrap(); // flush the file to ensure content is gonna be written

        // Notice that the log file is truncated with empty bytes
        assert_eq!(
            fs::read_to_string(expected_file).unwrap(),
            String::from("00000000000000000010\u{0}\u{0}\u{0}\u{0}\u{0}")
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut i = Index::new(tmp_dir.clone(), 0, 10).unwrap();
        // buffer is bigger than log size
        i.write(Entry::new(0, 10)).unwrap();
    }

    #[test]
    fn test_record_fit() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut i = Index::new(tmp_dir.clone(), 0, 100).unwrap();
        i.write(Entry::new(0, 10)).unwrap();

        assert!(i.fit(4));
        assert!(!i.fit(5));
    }

    #[test]
    fn test_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut i = Index::new(tmp_dir.clone(), 0, 50).unwrap();
        i.write(Entry::new(0, 10)).unwrap();
        i.write(Entry::new(10, 20)).unwrap();

        assert_eq!(i.read_at(0).unwrap(), Entry::new(0, 10));
        assert_eq!(i.read_at(1).unwrap(), Entry::new(10, 20));
    }

    #[test]
    #[should_panic]
    fn test_invalid_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        let mut i = Index::new(tmp_dir.clone(), 0, 50).unwrap();
        i.write(Entry::new(0, 10)).unwrap();

        i.read_at(20).unwrap(); // should fail since the position is invalid
    }
}
