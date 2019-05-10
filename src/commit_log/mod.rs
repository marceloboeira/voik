mod segment;

use self::segment::Segment;

use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::{distributions, thread_rng, Rng};
    use commit_log::CommitLog;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn random_hash() -> String {
        thread_rng()
            .sample_iter(&distributions::Alphanumeric)
            .take(30)
            .collect()
    }

    fn tmp_file_path() -> PathBuf {
        let mut tmp_dir = env::temp_dir();
        tmp_dir.push(random_hash());

        tmp_dir
    }

    #[test]
    fn test_initializing_when_the_path_is_invalid_it_fails() {
        match CommitLog::new(Path::new("\0").to_path_buf(), 100) {
            Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput),
            _ => assert!(false), // it should have failed
        }
    }

    #[test]
    fn test_initializing_when_the_path_is_valid_and_the_folder_does_not_exist_it_creates_the_folder(
    ) {
        let tmp_dir = tmp_file_path();
        CommitLog::new(tmp_dir.clone(), 100).unwrap();

        assert!(tmp_dir.as_path().exists());
    }

    #[test]
    fn test_initializing_when_the_path_is_valid_and_the_folder_already_exists_it_does_not_recreate_it(
    ) {
        let tmp_dir = tmp_file_path();
        fs::create_dir_all(tmp_dir.clone()).unwrap();

        CommitLog::new(tmp_dir, 100).unwrap();
    }

    #[test]
    fn test_writing_when_the_segment_has_space_it_writes_to_it() {
        let tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 100).unwrap();

        assert_eq!(c.write(b"this-has-less-than-100-bytes").unwrap(), 28);
    }

    #[test]
    fn test_writing_when_the_segment_has_no_space_it_rotates_to_a_new_segment() {
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
    fn test_writing_when_the_buffer_is_bigger_than_segment_size_it_errors() {
        let mut tmp_dir = tmp_file_path();

        let mut c = CommitLog::new(tmp_dir, 10).unwrap();
        c.write(b"the-buffer-is-too-big").unwrap();
    }
}
