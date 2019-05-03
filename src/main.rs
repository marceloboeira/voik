mod commit_log;

use commit_log::CommitLog;

use std::time::SystemTime;

fn main() -> Result<(), std::io::Error> {
    let now = SystemTime::now();
    let mut target_path = std::env::home_dir().unwrap();
    target_path.push("loglady/");

    println!("👵 loglady logging to {:?}", target_path);

    let segment_size = 2097152; // 2MB
    let total_messages = 1_000_000;
    let mut clog = CommitLog::new(target_path, segment_size)?;

    for _ in 0..total_messages {
        clog.write(b"one day my log will have something to say about this ")?;
    }

    let ms = now.elapsed().unwrap().as_millis();

    println!("{} messages written in {} ms", total_messages, ms);
    Ok(())
}
