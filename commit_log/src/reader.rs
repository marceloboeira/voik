use super::record::Record;
use super::CommitLog;
use super::position::Position;
use std::result::Result;
use std::io::Error;

pub struct Reader<'a> {
    commit_log: &'a CommitLog
}

impl<'a> Reader<'a> {

    /// Read the log according to record's information.
    ///
    /// # Arguments
    /// * `record` - A Record to be read.
    pub fn read(& mut self, record: Record) -> Result<&[u8], Error> {
        let segment = &self.commit_log.segments[record.segment_index];
        segment.read_at(record.current_offset)
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
    pub fn record_after(record: Record, offset: usize) -> Record {
        Record {
            segment_index: record.segment_index,
            current_offset: record.current_offset + offset
        }
    }

    /// Get the next record's information.
    pub fn next(record: Record) -> Record {
        Reader::record_after(record, 1)
    }
}