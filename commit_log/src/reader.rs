use crate::{CommitLog, Position, Record};

use std::io;
use std::result::Result;

use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    Io(io::Error),
    Segment(super::segment::Error),
    InvalidPosition,
}

pub struct Reader<'a> {
    pub commit_log: &'a CommitLog,
}

impl<'a> Reader<'a> {
    /// Read the log according to record's information.
    ///
    /// # Arguments
    /// * `record` - A Record to be read.
    pub fn read(&self, record: &Record) -> Result<&[u8], Error> {
        let segment_index = record.segment_index;
        let total_segments = self.commit_log.segments.len();
        if segment_index >= total_segments {
            Err(Error::InvalidPosition)
        } else {
            let segment = &self.commit_log.segments[segment_index];
            let buf = segment.read_at(record.current_offset)?;
            Ok(buf)
        }
    }

    /// Read the position of one record
    ///
    /// # Arguments
    /// * `record` - A Record to be read.
    pub fn position(record: Record) -> Position {
        Position::Offset(record.current_offset)
    }

    /// Get record information after number of offset.
    ///
    /// # Arguments
    /// * `record` - the current record.
    /// * `offset` - the offset from expected record to current record.
    pub fn record_after(record: &Record, offset: usize) -> Record {
        Record {
            segment_index: record.segment_index,
            current_offset: record.current_offset + offset,
        }
    }

    /// Get the next record's information.
    pub fn next(record: &Record) -> Record {
        Reader::record_after(record, 1)
    }

    pub fn next_segment(record: &Record) -> Record {
        Record {
            segment_index: record.segment_index + 1,
            current_offset: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 50, 10000).unwrap();

        c.write(b"this-has-less-20b").unwrap();
        c.write(b"second-record").unwrap();
        c.write(b"third-record-bigger-goes-to-another-segment")
            .unwrap(); // segment switch trigger

        let record = Record {
            current_offset: 0,
            segment_index: 0,
        };
        let reader = Reader { commit_log: &c };
        assert_eq!(
            reader.read(&record).unwrap(),
            "this-has-less-20b".as_bytes()
        );
    }

    #[test]
    fn test_record_after() {
        let tmp_dir = tempdir().unwrap().path().to_owned();
        let mut c = CommitLog::new(tmp_dir, 50, 10000).unwrap();

        c.write(b"this-has-less-20b").unwrap();
        c.write(b"second-record").unwrap();
        c.write(b"third-record-bigger-goes-to-another-segment")
            .unwrap(); // segment switch trigger

        let record = Record {
            current_offset: 0,
            segment_index: 0,
        };
        let reader = Reader { commit_log: &c };
        let record_after = Reader::record_after(&record, 1);
        assert_eq!(record_after.current_offset, 1);
        assert_eq!(record_after.segment_index, 0);
        assert_eq!(
            reader.read(&record_after).unwrap(),
            "second-record".as_bytes()
        );
    }
}
