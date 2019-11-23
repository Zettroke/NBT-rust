use nbt_rust::nbt::NBT;
use std::fs::File;

pub fn main() {
    let mut f = File::open("Bikini Bottom V2/level").unwrap();
    let res = NBT::from(&mut f).unwrap();

    println!("{}", res.to_string());

    println!("{:?}", res);
}