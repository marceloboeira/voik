use crc::Hasher64;
use tempfile::tempdir;

use commit_log::{CommitLog, Error};
use consts::*;
use utils::{crc_digest, generate_random_values};

mod consts;
mod utils;

#[test]
fn test_commit_log_data_consistency_of_random_values() {
    let tmp_dir = tempdir().unwrap().path().to_owned();
    let mut write_crc = crc_digest();
    let mut commit_log = CommitLog::new(tmp_dir.clone(), SEGMENT_SIZE, INDEX_SIZE).unwrap();

    generate_random_values(
        NUMBER_OF_ELEMENTS_TO_INSERT,
        DATA_ITEM_SIZE,
        |random_value| {
            write_crc.write(&random_value);
            commit_log.write(&*random_value).unwrap();
        },
    );

    let mut read_crc = crc_digest();
    let mut segment = 0;
    let mut offset = 0;

    loop {
        match commit_log.read_at(segment, offset) {
            Ok(value) => {
                read_crc.write(value);
                offset += 1;
            }
            Err(Error::SegmentUnavailable) => break,
            Err(_) => {
                segment += 1;
                offset = 0;
            }
        };
    }

    assert_eq!(write_crc.sum64(), read_crc.sum64());
}
