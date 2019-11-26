use nbt_rust::nbt::NBT;
use std::fs::File;

fn main() {
    let mut f = File::open("Bikini Bottom V2/level").unwrap();
    let data = NBT::from(&mut f).unwrap();

    println!("{}", data);
}