mod commit_log;

use commit_log::CommitLog;

fn main() -> Result<(), std::io::Error> {
    let mut target_path = std::env::home_dir().unwrap();
    target_path.push("loglady");

    println!("👵 loglady loggin to {:?}", target_path);

    let mut clog = CommitLog::new(target_path)?;
    for _ in 0..100_000 {
        clog.write(b"one day my log will have something to say about this")?;
    }

    Ok(())
}
