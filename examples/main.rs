use nbt_rust::nbt::NBT;
use std::fs::File;
use std::io::Write;

pub fn main() {
    let mut f = File::open("Bikini Bottom V2/level").unwrap();
    let res2 = NBT::from(&mut f).unwrap();

    let s = res2.to_string();

    let mut out = File::create("dump.txt").unwrap();
    out.write(s.as_bytes()).unwrap();
    drop(out);

    println!("{}", res2.to_string());

    println!("{:?}", res2);
}