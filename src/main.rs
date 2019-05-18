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

    let segment_size = 5_000_000;
    let index_size = 5_000_000;
    let total_messages = 1_000_000;
    let mut clog = CommitLog::new(target_path, segment_size, index_size)?;

    let start = SystemTime::now();
    for i in 0..total_messages {
        clog.write(format!("m-{:010}", i).as_bytes())?;
    }

    let write_time = SystemTime::now();
    println!(
        "{} messages written in {:?}",
        total_messages,
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
        "{} messages read in {:?}",
        total_messages,
        SystemTime::now()
            .duration_since(write_time)
            .expect("Time went backwards")
    );

    Ok(())
}
