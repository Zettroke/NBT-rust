use nbt_rust::nbt_rust::NBT;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut f = File::open("Bikini Bottom V2/level").unwrap();
    let data = NBT::load(&mut f).unwrap();

    let mut out = File::create("res.dat").unwrap();
    let res = data.dump();
    out.write(&res);

    println!("{}", data);
}