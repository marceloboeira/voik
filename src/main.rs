mod commit_log;

extern crate dirs;

use commit_log::CommitLog;
use std::fs;
use std::time::SystemTime;

fn main() -> Result<(), std::io::Error> {
    let mut target_path = dirs::home_dir().unwrap();
    target_path.push("loglady/");

    fs::remove_dir_all(target_path.clone())?;
    println!("👵 loglady logging to {:?}", target_path);

    let segment_size = 100;
    let total_messages = 100;
    let mut clog = CommitLog::new(target_path, segment_size)?;

    let start = SystemTime::now();
    for i in 0..total_messages {
        clog.write(format!("m-{:010}", i).as_bytes())?;
    }

    let ms = SystemTime::now()
        .duration_since(start)
        .expect("Time went backwards");
    println!("{} messages written in {:?}", total_messages, ms);

    let mut i = 0;
    let mut j = 0;
    let mut segment_error = false;
    loop {
        match clog.read_at(i, j) {
            Ok(s) => {
                println!("ok");
                segment_error = false;
                j += 1;
                println!("{:?}", String::from_utf8(s));
            }
            _ => {
                println!("nope1");
                if segment_error {
                    println!("nope2");
                    break;
                } else {
                    println!("nope3");
                    segment_error = true;
                    i += 1;
                    j = 0;
                }
            }
        }
    }

    let ms = SystemTime::now()
        .duration_since(start)
        .expect("Time went backwards");
    println!("{} messages read in {:?}", total_messages, ms);

    Ok(())
}
