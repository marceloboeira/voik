#![feature(test,rustc_private)]
#[cfg(test)]
extern crate test as bench;
pub mod test;
pub mod commit_log;

fn main() {
    println!("voik");
}
