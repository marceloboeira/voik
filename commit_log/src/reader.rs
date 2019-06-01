use super::record::Record;

pub struct Reader {
    commit_log: &CommitLog
}

impl Reader {

    pub fn record(&mut self, record: Record) -> Result<&[u8], Error> {
        let segment = self.commit_log.segment[record.segment_index];
        segment.read_at(record.current_offset)
    }

    pub fn position(record: Record) -> Position {
        Position::Offset(record.current_offset)
    }

    pub fn record_after(record: Record) -> Record {
        Record {
            segment_index: record.segment_index,
            current_offset: record.current_offset + offset
        }
    }

    pub fn next(record: Record) -> Record {
        Reader::record_after(1)
    }
}