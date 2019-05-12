extern crate memmap;

use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str;
use self::memmap::{Mmap, MmapMut};

/// Index
///
/// A wrapper for writing/reading entries to the index file
///
/// Every log has an index companion, e.g.:
///
/// 00000000000011812312.log
/// 00000000000011812312.idx
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
#[derive(Debug)]
pub struct Index {
    /// File Descriptor
    file: File,

    /// Reader memory buffer
    reader: Mmap,

    /// Reader memory buffer
    writer: MmapMut,

    /// Current size of the index in bytes
    offset: usize,
}

/// Amount of bytes for each entry on the index
const ENTRY_SIZE: usize = 20;

impl Index {
    /// Creates a new Index / reads the existing Index
    pub fn new(path: PathBuf, offset: usize) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.join(format!("{:020}.idx", offset)))?; //TODO improve file formatting

        // TODO find a better way of setting up the index file size
        file.set_len(10 * 1_048_576).unwrap();

        let reader = unsafe { Mmap::map(&file).expect("failed to map the file") };
        let writer = unsafe { MmapMut::map_mut(&file).expect("failed to map the file") };

        Ok(Self {
            offset: 0,
            file: file,
            reader: reader,
            writer: writer,
        })
    }

    /// Writes an entry to the index
    pub fn write(&mut self, entry: Entry) -> Result<usize, Error> {
        // TODO set the file to its end since the read can seek to specific parts
        let from = self.offset;
        let to = from + ENTRY_SIZE;
        self.offset += ENTRY_SIZE;

        (&mut self.writer[from..=to]).write(entry.to_string().as_bytes())
    }

    pub fn flush(&mut self) {
        self.writer.flush().unwrap();
    }

    /// Reads an entry from the index
    pub fn read_at(&mut self, offset: usize) -> Result<(Entry), Error> {
        let seek = (offset * ENTRY_SIZE) as u64;
        self.file.seek(SeekFrom::Start(seek))?;

        // TODO avoid panics... throw the error up
        // TODO avoid reading 2 times from the file
        // TODO avoid parsing to string -> usize...
        //reads 10 pieces at a time
        let mut buffer = [0; 10]; //TODO use entry-size/2

        // Reads
        self.file.read(&mut buffer)?;
        let position = str::from_utf8(&buffer).unwrap().parse::<usize>().unwrap();

        //reads 10 pieces at a time
        self.file.read(&mut buffer)?;
        let size = str::from_utf8(&buffer).unwrap().parse::<usize>().unwrap();

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
    pub fn new(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }

    pub fn to_string(&self) -> String {
        format!("{:010}{:010}", self.offset, self.size)
    }
}

#[cfg(test)]
mod tests {
    use commit_log::segment::index::{Entry, Index};
    use commit_log::test::*;
    use std::fs;

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
    fn test_write() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        let expected_file = tmp_dir.clone().join("00000000000000000000.idx");

        let mut i = Index::new(tmp_dir.clone(), 0).unwrap();

        i.write(Entry::new(0, 12)).unwrap();
        i.write(Entry::new(12, 15)).unwrap();
        i.write(Entry::new(15, 50)).unwrap();

        assert_eq!(i.read_at(0).unwrap(), Entry::new(0, 12));
        assert_eq!(i.read_at(1).unwrap(), Entry::new(12, 15));
        assert_eq!(i.read_at(2).unwrap(), Entry::new(15, 50));
    }
}
