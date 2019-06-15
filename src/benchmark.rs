extern crate commit_log;
extern crate dirs;

use commit_log::CommitLog;
use commit_log::Reader;
use commit_log::Record;
use std::time::SystemTime;

fn loop_commit_log(clog: &CommitLog) -> Result<(), std::io::Error> {
    let mut record = Record {
        current_offset: 0,
        segment_index: 1,
    };
    let reader = Reader { commit_log: &clog };
    let mut segment_error = false;
    loop {
        match reader.read(&record) {
            Ok(_) => {
                segment_error = false;
                record = Reader::next(&record);
                //println!("{}", std::str::from_utf8(s).unwrap());
            }
            _ => {
                if segment_error {
                    //println!("error 2 {:?}", e);
                    break;
                } else {
                    //println!("error 1 {:?}", e);
                    segment_error = true;
                    record = Reader::next_segment(&record);
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let mut target_path = dirs::home_dir().unwrap();
    target_path.push("voik/");

    println!(
        "⚫️ voik benchmark starting, logging to {:?}",
        target_path
    );

    let segment_size: u64 = 20_000_000; // 20MB
    let index_size: u64 = 10_000_000; // 10MB
    let total_messages: u64 = 1_000_000; // 1M messages
    let total_size_gb = (total_messages * 1000) / 1_000_000_000; // (messages vs 1000 bytes) / 1 GB
    let mut clog = CommitLog::new(target_path, segment_size as usize, index_size as usize)?;

    // ########### Write Performance Benchmark  ###########
    let start = SystemTime::now();
    for i in 0..total_messages {
        clog.write(format!("{:01000}", i).as_bytes())?; // 1KB record
    }

    let write_time = SystemTime::now();
    println!(
        "{} GB worth records written in {:?}",
        total_size_gb,
        write_time
            .duration_since(start)
            .expect("Time went backwards")
    );

    // ########### Cold Read Performance Benchmark  ###########
    // TODO implement a better way of READING sequencially, PLEASE
    // Read from first record, on the first segment (Horizon)
    loop_commit_log(&clog)?;

    println!(
        "{} GB worth cold records read in {:?}",
        total_size_gb,
        SystemTime::now()
            .duration_since(write_time)
            .expect("Time went backwards")
    );

    // ########### Warm Read Performance Benchmark  ###########
    let warm = SystemTime::now();

    loop_commit_log(&clog)?;
    println!(
        "{} GB worth warm records read in {:?}",
        total_size_gb,
        SystemTime::now()
            .duration_since(warm)
            .expect("Time went backwards")
    );
    Ok(())
}
