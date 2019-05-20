#![feature(test)]
#[cfg(test)]
extern crate test as bench;
pub mod test;
pub mod commit_log;

extern crate dirs;

use commit_log::CommitLog;
use std::fs;
use std::time::SystemTime;

fn main() -> Result<(), std::io::Error> {
    let mut target_path = dirs::home_dir().unwrap();
    target_path.push("voik/");

    fs::remove_dir_all(target_path.clone())?;
    println!("⚫️ voik logging to {:?}", target_path);

    let segment_size = 20_000_000; // 20MB
    let index_size = 10_000_000; // 10MB
    let total_messages = 100_000_000;
    let total_size_mb = (total_messages * 100) / 1_000_000;
    let mut clog = CommitLog::new(target_path, segment_size, index_size)?;

    let start = SystemTime::now();
    for i in 0..total_messages {
        clog.write(format!("{:0100}", i).as_bytes())?; // 100 bytes record
    }

    let write_time = SystemTime::now();
    println!(
        "{} Megabytes worth records written in {:?}",
        total_size_mb,
        write_time
            .duration_since(start)
            .expect("Time went backwards")
    );

    // TODO implement a better way of READING sequencially, PLEASE
    // Read from first record, on the first segment
    let mut i = 0;
    let mut j = 0;
    let mut segment_error = false;
    loop {
        match clog.read_at(i, j) {
            Ok(_) => {
                segment_error = false;
                j += 1;
                //println!("{}", String::from_utf8(s).unwrap());
            }
            _ => {
                if segment_error {
                    //println!("error 2 {:?}", e);
                    break;
                } else {
                    //println!("error 1 {:?}", e);
                    segment_error = true;
                    i += 1;
                    j = 0;
                }
            }
        }
    }

    println!(
        "{} Megabytes worth records read in {:?}",
        total_size_mb,
        SystemTime::now()
            .duration_since(write_time)
            .expect("Time went backwards")
    );

    let warm = SystemTime::now();

    let mut i = 0;
    let mut j = 0;
    let mut segment_error = false;
    loop {
        match clog.read_at(i, j) {
            Ok(_) => {
                segment_error = false;
                j += 1;
                //println!("{}", std::str::from_utf8(s).unwrap());
            }
            _ => {
                if segment_error {
                    //println!("error 2 {:?}", e);
                    break;
                } else {
                    //println!("error 1 {:?}", e);
                    segment_error = true;
                    i += 1;
                    j = 0;
                }
            }
        }
    }

    println!(
        "{} Megabytes worth warm records read in {:?}",
        total_size_mb,
        SystemTime::now()
            .duration_since(warm)
            .expect("Time went backwards")
    );

    Ok(())
}
