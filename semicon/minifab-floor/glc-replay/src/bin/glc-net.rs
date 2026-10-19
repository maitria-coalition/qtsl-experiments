//! Print the sealed net of a container (what the bytes say, nothing more).
fn main() {
    let path = std::env::args().nth(1).expect("usage: glc-net FILE.glc");
    let bytes = std::fs::read(&path).expect("read");
    let c = minifab_glc_replay::decode::open(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let net = c.sealed_net(0).unwrap_or_else(|e| panic!("{path}: {e}"));
    println!("mech {} plates {} marking {:?}", net.mech, net.marking.len(), net.marking);
    for (i, t) in net.trs.iter().enumerate() {
        println!("tr#{i} sel={:?} clock={:?} scale={:?}", t.sel, t.clock, t.scale);
    }
}
