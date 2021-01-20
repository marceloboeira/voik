extern crate memmap;
mod reader;
mod segment;

use self::segment::Segment;
pub use reader::Reader;

use std::fs;
use std::io;
use std::path::PathBuf;

use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    Io(io::Error),
    Segment(segment::Error),
    BufferSizeExceeded,
    SegmentUnavailable,
}

pub enum Position {
    /// The first entry available.
    Horizon,
    Offset(usize),
}

pub struct Record {
    /// The current offset within current segment.
    pub current_offset: usize,
    /// Index to current segment.
    pub segment_index: usize,
}

/// CommitLog
///
/// The commit log is an abstraction that manages writes/reads to segments creating an append-only
/// log. That's accomplished by storing a vector of Segments, and managing a pointer to the current
/// segment.
///
/// Records can be written to the log, always appending the last record over and over.
///
/// Each time a record is written, the segment is trusted to have enough space for the given
/// buffer, then the record is written to the current segment, and the pointer is updated.
/// ```ignore
///                          current cursor
/// segment 0                       ^
/// |-------------------------------|
/// | record 0  |  record 1  |  ... |  --> time
/// |-------------------------------|
///
/// ```
/// When a segment is full, the commit log makes sure to rotate to a new one, closing the
/// old one.
///
/// See how it looks like on disk (on a high-level):
/// ```ignore
///                                                        current cursor
/// segment 0                                                     ^
/// |-------------------------------|                             |
/// | record 0  |  record 1  |  ... | segment 1 (current)         |
/// |-------------------------------|-----------------------------| --> time
///                                 |  record 2  | record 3 | ... |
///                                 |-----------------------------|
/// ```
/// Under the hood is a bit more complex, the management of writing to the file to disk is
/// of the Segments', as well as managing the Index file.
///
/// More info in the segment.rs and segment/index.rs files.
///
pub struct CommitLog {
    /// Root directory for the Commitlog files
    path: PathBuf,

    /// Size in bytes for the segments
    segment_size: usize,

    /// Size in bytes for the index
    index_size: usize,

    /// List of segments
    segments: Vec<Segment>, //TODO if too many Segments are created, and not "garbage collected", we have too many files opened

    /// Current segment index
    current_segment: usize,
}

impl CommitLog {
    pub fn new<P: Into<PathBuf>>(
        path: P,
        segment_size: usize,
        index_size: usize,
    ) -> Result<Self, Error> {
        let path = path.into();
        if !path.as_path().exists() {
            fs::create_dir_all(path.clone())?;
        }

        let segments = vec![Segment::new(path.clone(), 0, segment_size, index_size)?];

        Ok(Self {
            path,
            segments,
            segment_size,
            index_size,
            current_segment: 0,
        })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len();

        if buffer_size > self.segment_size {
            return Err(Error::BufferSizeExceeded);
        }

        if !self.active_segment().fit(buffer_size) {
            self.rotate_segment()?;
        }

        let len = self.active_segment().write(buffer)?;
        Ok(len)
    }

    pub fn read_at(&mut self, segment_index: usize, offset: usize) -> Result<&[u8], Error> {
        if segment_index >= self.segments.len() {
            return Err(Error::SegmentUnavailable);
        }

        let buf = self.segments[segment_index].read_at(offset)?;
        Ok(buf)
    }

    pub fn read_after(&mut self, position: &Position, mut offset: usize) -> Result<Record, Error> {
        let horizon: usize = 1;
        let current_pos = match position {
            Position::Horizon => horizon,
            Position::Offset(offset) => *offset,
        };
        offset += current_pos;

        Ok(Record {
            segment_index: self.current_segment,
            current_offset: offset,
        })
    }

    pub fn read(&mut self, position: &Position) -> Result<Record, Error> {
        self.read_after(position, 0)
    }

    fn rotate_segment(&mut self) -> Result<(), Error> {
        let next_offset = self.segments.len();

        self.active_segment().flush()?;

        self.segments.push(Segment::new(
            self.path.clone(),
            next_offset,
            self.segment_size,
            self.index_size,
        )?);

        Ok(())
    }

    fn active_segment(&mut self) -> &mut Segment {
        let index = self.segments.len() - 1;
        &mut self.segments[index]
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
    #[should_panic]
    fn test_invalid_create() {
        CommitLog::new(Path::new("\0").to_path_buf(), 100, 10000).unwrap();
    }

    #[test]
    fn test_create() {
        // create folder
        let tmp_dir = tempdir().unwrap().path().to_owned();
        CommitLog::new(tmp_dir.clone(), 100, 1000).unwrap();
        assert!(tmp_dir.as_path().exists());

        // accept an existing folder
        let tmp_dir = tempdir().unwrap().path().to_owned();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        CommitLog::new(tmp_dir.clone(), 100, 1000).unwrap();
        assert!(tmp_dir.as_path().exists());
    }

    #[test]
    fn test_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 100, 1000).unwrap();

        assert_eq!(c.write(b"this-has-less-than-100-bytes").unwrap(), 28);
    }

    #[test]
    fn test_write_rotate_segments() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 100, 1000).unwrap();
        c.write(
            b"this-should-have-about-80-bytes-but-not-really-sure-to-be-honest-maybe-it-doesn't",
        )
        .unwrap();

        // it should 'fail' since the segment has only 100 bytes, but this triggers a rotation
        assert_eq!(c.write(b"a-bit-more-than-20-bytes").unwrap(), 24);
    }

    #[test]
    #[should_panic]
    fn test_invalid_write() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 10, 10000).unwrap();

        // it should fail since the buffer is bigger than the max size of the segment
        c.write(b"the-buffer-is-too-big").unwrap();
    }

    #[test]
    fn test_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 50, 10000).unwrap();

        c.write(b"this-has-less-20b").unwrap();
        c.write(b"second-record").unwrap();
        // segment switch trigger
        c.write(b"third-record-bigger-goes-to-another-segment")
            .unwrap();

        assert_eq!(c.read_at(0, 0).unwrap(), "this-has-less-20b".as_bytes());
        assert_eq!(c.read_at(0, 1).unwrap(), "second-record".as_bytes());
        assert_eq!(
            c.read_at(1, 0).unwrap(),
            "third-record-bigger-goes-to-another-segment".as_bytes()
        );
    }
}
