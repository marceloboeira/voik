mod segment;

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
    /// Root directory for the Commitlog files
    path: PathBuf,

    /// Size in bytes for the segments
    segment_size: usize,

    /// Size in bytes for the index
    index_size: usize,

    /// List of segments
    segments: Vec<Segment>, //TODO if too many Segments are created, and not "garbage collected", we have too many files opened
}

impl CommitLog {
    pub fn new(path: PathBuf, segment_size: usize, index_size: usize) -> Result<Self, Error> {
        if !path.as_path().exists() {
            fs::create_dir_all(path.clone())?;
        }

        //TODO figure it out the segment starting in 0, should we truncate the file?
        let segments = vec![Segment::new(path.clone(), 0, segment_size, index_size)?];

        Ok(Self {
            path: path,
            segments: segments,
            segment_size: segment_size,
            index_size: index_size,
        })
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        let buffer_size = buffer.len();

        //TODO find a better place for this?
        if buffer_size > self.segment_size {
            return Err(Error::new(
                ErrorKind::Other,
                "Buffer size is bigger than segment size",
            ));
        }

        //TODO find a better place for this?
        if !self.active_segment().fit(buffer_size) {
            let segments_size = self.segments.len();

            //TODO close/truncate segment
            self.active_segment().flush()?;

            self.segments.push(Segment::new(
                self.path.clone(),
                segments_size,
                self.segment_size,
                self.index_size,
            )?);
        }

        self.active_segment().write(buffer)
    }

    pub fn read_at(&mut self, segment_index: usize, offset: usize) -> Result<&[u8], Error> {
        if segment_index >= self.segments.len() {
            return Err(Error::new(ErrorKind::Other, "Segment not available"));
        }
        self.segments[segment_index].read_at(offset)
    }

    fn active_segment(&mut self) -> &mut Segment {
        let index = self.segments.len() - 1;
        &mut self.segments[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bench::Bencher;
    use std::fs;
    use std::path::Path;
    use test::*;

    #[test]
    #[should_panic]
    fn test_invalid_create() {
        CommitLog::new(Path::new("\0").to_path_buf(), 100, 10000).unwrap();
    }

    #[test]
    fn test_create() {
        // create folder
        let tmp_dir = tmp_file_path();
        CommitLog::new(tmp_dir.clone(), 100, 1000).unwrap();
        assert!(tmp_dir.as_path().exists());

        // accept an existing folder
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();
        CommitLog::new(tmp_dir.clone(), 100, 1000).unwrap();
        assert!(tmp_dir.as_path().exists());
    }

    #[test]
    fn test_write() {
        let tmp_dir = tmp_file_path();
        let mut c = CommitLog::new(tmp_dir, 100, 1000).unwrap();

        assert_eq!(c.write(b"this-has-less-than-100-bytes").unwrap(), 28);
    }

    #[test]
    fn test_write_rotate_segments() {
        let tmp_dir = tmp_file_path();
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
        let tmp_dir = tmp_file_path();
        let mut c = CommitLog::new(tmp_dir, 10, 10000).unwrap();

        // it should fail since the buffer is bigger than the max size of the segment
        c.write(b"the-buffer-is-too-big").unwrap();
    }

    #[test]
    fn test_read() {
        let tmp_dir = tmp_file_path();
        let mut c = CommitLog::new(tmp_dir, 30, 10000).unwrap();

        c.write(b"this-has-less-20b").unwrap();
        c.write(b"second-record").unwrap();
        c.write(b"this-is-gonna-switch-segment").unwrap();

        assert_eq!(c.read_at(0, 0).unwrap(), b"this-has-less-20b");
        assert_eq!(c.read_at(0, 1).unwrap(), b"second-record");
        assert_eq!(c.read_at(1, 0).unwrap(), b"this-is-gonna-switch-segment");
    }

    /// Benchmarks
    #[bench]
    fn bench_write(b: &mut Bencher) {
        let tmp_dir = tmp_file_path();
        let segment_size = 20_000_000; // 20MB
        let index_size = 10_000_000; // 10MB

        let mut c = CommitLog::new(tmp_dir.clone(), segment_size, index_size).unwrap();

        b.iter(|| {
            let n = bench::black_box(1_000);

            for i in 0..n {
                bench::black_box(c.write(format!("{:0100}", i).as_bytes()).unwrap()); // 100 bytes record
            }
        });
    }
}
