//! Dump a sealed container's ops with every external reference resolved to
//! its declared name through the checker's derived baked vocabulary.
use minifab_glc_replay::decode::{open, show_op};
fn main() {
    let path = std::env::args().nth(1).expect("usage: glc-dump FILE.glc");
    let bytes = std::fs::read(&path).expect("read");
    let c = open(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    for (ci, ops) in c.commits.iter().enumerate() {
        for (i, op) in ops.iter().enumerate() {
            println!("c{ci} #{i} {}", show_op(&c, op));
        }
    }
}
