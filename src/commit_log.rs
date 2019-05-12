mod segment;
#[cfg(feature = "test-prelude")]
mod test;

use self::segment::Segment;

use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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
///
///                          current cursor
/// segment 0                       ^
/// |-------------------------------|
/// | record 0  |  record 1  |  ... |  --> time
/// |-------------------------------|
///
/// When a segment is full, the commit log makes sure to rotate to a new one, closing the
/// old one.
///
/// See how it looks like on disk (on a high-level):
///                                                        current cursor
/// segment 0                                                     ^
/// |-------------------------------|                             |
/// | record 0  |  record 1  |  ... | segment 1 (current)         |
/// |-------------------------------|-----------------------------| --> time
///                                 |  record 2  | record 3 | ... |
///                                 |-----------------------------|
///
/// Under the hood is a bit more complex, the management of writing to the file to disk is
/// of the Segments', as well as managing the Index file.
///
/// More info in the segment.rs and segment/index.rs files.
///
pub struct CommitLog {
    // Root directory for the Commitlog files
    path: PathBuf,

    // Size in bytes for the segments
    segment_size: usize,

    // List of segments
    segments: Vec<Segment>, //TODO if too many Segments are created, and not "garbage collected", we have too many files opened
}

impl CommitLog {
    pub fn new(path: PathBuf, segment_size: usize) -> Result<Self, std::io::Error> {
        if !path.as_path().exists() {
            fs::create_dir_all(path.clone())?;
        }

        //TODO figure it out the segment starting in 0, should we truncate the file?
        let segments = vec![Segment::new(path.clone(), 0, segment_size)?];

        Ok(Self {
            path: path,
            segments: segments,
            segment_size: segment_size,
        })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        let buffer_size = buffer.len();

        //TODO find a better place for this?
        if buffer_size > self.segment_size {
            return Err(Error::new(
                ErrorKind::Other,
                "Buffer size is bigger than segment size",
            ));
        }

        if buffer_size > self.active_segment().space_left() {
            let segments_size = self.segments.len();
            self.segments.push(Segment::new(
                self.path.clone(),
                segments_size,
                self.segment_size,
            )?);
        }
        self.active_segment().write(buffer)
    }

    fn active_segment(&mut self) -> &mut Segment {
        let index = self.segments.len() - 1;
        &mut self.segments[index]
    }

    pub fn read_at(&mut self, segment_index: usize, offset: usize) -> Result<Vec<u8>, Error> {
        self.segments[segment_index].read_at(offset)
    }
}

#[cfg(test)]
mod tests {
    use commit_log::test::*;
    use commit_log::CommitLog;
    use std::fs;
    use std::path::Path;

    #[test]
    #[should_panic]
    fn it_fails_to_initialize_when_the_path_is_invalid() {
        CommitLog::new(Path::new("\0").to_path_buf(), 100).unwrap();
    }

    #[test]
    fn it_creates_the_folder_when_it_does_not_already_exist() {
        let tmp_dir = tmp_file_path();
        CommitLog::new(tmp_dir.clone(), 100).unwrap();

        assert!(tmp_dir.as_path().exists());
    }

    #[test]
    fn it_does_not_recreate_the_folder_when_it_already_exist() {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        CommitLog::new(tmp_dir, 100).unwrap();
    }

    #[test]
    fn it_writes_to_a_segment() {
        let tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 100).unwrap();

        assert_eq!(c.write(b"this-has-less-than-100-bytes").unwrap(), 28);
    }

    #[test]
    fn it_writes_to_a_new_segment_when_full() {
        let tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 100).unwrap();
        c.write(
            b"this-should-have-about-80-bytes-but-not-really-sure-to-be-honest-maybe-it-doesn't",
        )
        .unwrap();

        assert_eq!(c.write(b"a-bit-more-than-20-bytes").unwrap(), 24);
    }

    #[test]
    #[should_panic]
    fn it_fails_to_write_a_record_bigger_than_the_segment_size() {
        let tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 10).unwrap();
        c.write(b"the-buffer-is-too-big").unwrap();
    }

    #[test]
    fn test_reads_at_a_given_position() {
        let tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 50).unwrap();
        c.write(b"this-has-less-20b").unwrap();
        c.write(b"second-record").unwrap();
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
