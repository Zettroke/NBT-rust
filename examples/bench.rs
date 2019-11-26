use nbt_rust::nbt::NBT;
use std::fs::File;
use std::io::{Write, Read};
use std::cell::RefCell;
use std::rc::Rc;
use std::borrow::{Borrow, BorrowMut};
use time::{PreciseTime, Duration};
use std::cmp::Ordering;

//use crate::time:
pub fn main() {
    let mut f = File::open("Bikini Bottom V2/level").unwrap();
    let mut buff = Vec::new();
    f.read_to_end(&mut buff);
    let mut v = Vec::new();
    let it_outer = 1000;
    let it_inner = 100;
    for _ in 0..it_outer {
        let st = PreciseTime::now();

        for _ in 0..it_inner {
            let mut s = buff.as_slice();
            let res = NBT::from(&mut s).unwrap();
        }

        let en = PreciseTime::now();
        let res = st.to(en).num_nanoseconds().unwrap() as f64 / it_inner as f64;
        v.push(res);
    }
    v.sort_by(|a, b| if a < b { Ordering::Less } else { Ordering::Greater } );

    let s: f64 = v[0..950].iter().sum();

    println!("first: {}, last: {}", v.first().unwrap(), v.last().unwrap());
    println!("fastest 95%: {} ns/it.", s / v.len() as f64);
    println!("fastest med 95%: {} ns/it.", v[950/2]);
}