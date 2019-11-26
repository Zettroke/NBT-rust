#![feature(test)]
pub mod nbt {
    extern crate bytes;

    use std::io::{Read, Error, ErrorKind};
    use bytes::BigEndian;
    use bytes::ByteOrder;

    use linked_hash_map::LinkedHashMap;
    use core::fmt;

    static BUFF_SIZE: usize = 65536;

    #[derive(Debug)]
    pub enum NBT {
        Byte(u8),
        Short(i16),
        Int(i32),
        Long(i64),
        Float(f32),
        Double(f64),
        ByteArray(Vec<u8>),
        String(String),
        List(Vec<NBT>),
        Compound(LinkedHashMap<String, NBT>),
    }

    pub struct ReadWrapper<T: Read> {
        inner: T
    }

    impl<T: Read> ReadWrapper<T> {
        pub fn from(input: T) -> ReadWrapper<T> {
            return ReadWrapper {
                inner: input
            };
        }

        #[allow(unused_must_use)]
        pub fn read_n(&mut self, buff: &mut [u8], n: usize) {
            self.read(&mut buff[0..n]);
        }

        #[allow(unused_must_use)]
        pub fn read_name(&mut self) -> String {
            let name_buff = &mut [0u8; 2];
            self.read(name_buff);
            let n = BigEndian::read_u16(name_buff) as usize;
            if n == 0 {
                "empty".to_string()
            } else {
                let mut buff = vec![0; n];
                self.read(&mut buff);

                String::from_utf8(buff).unwrap()
            }
        }

        pub fn read_plain_nbt(&mut self, t: u8, buff: &mut [u8]) -> NBT {
            match t {
                0x01 => { // Byte
                    self.read_n(buff, 1);
                    NBT::Byte(buff[0])
                }
                0x02 => { // Short
                    self.read_n(buff, 2);
                    NBT::Short(BigEndian::read_i16(buff))
                }
                0x03 => { // Int
                    self.read_n(buff, 4);
                    NBT::Int(BigEndian::read_i32(buff))
                }
                0x04 => { // Long
                    self.read_n(buff, 8);
                    NBT::Long(BigEndian::read_i64(buff))
                }
                0x05 => { // Float
                    self.read_n(buff, 4);
                    NBT::Float(BigEndian::read_f32(buff))
                }
                0x06 => { // Double
                    self.read_n(buff, 8);
                    NBT::Double(BigEndian::read_f64(buff))
                }
                0x07 => { // Byte Array
                    self.read_n(buff, 4);
                    let n = BigEndian::read_u32(buff) as usize;
                    let mut cur = 0usize;

                    let mut res: Vec<u8> = Vec::with_capacity(n);

                    while cur < n {
                        cur += self.read(buff).unwrap();
                        res.extend_from_slice(buff);
                    }

                    NBT::ByteArray(res)
                }
                0x08 => {
                    NBT::String(self.read_name())
                }
                _ => NBT::String(String::from("Invalid tag!!!"))
            }
        }
        pub fn read_compound(&mut self, buff: &mut [u8]) -> NBT {
            let mut container = LinkedHashMap::new();
            loop {
                self.read_n(buff, 1);
                let t = buff[0];

                if t == 0 {
                    break
                }

                let name = self.read_name();

                let value = match t {
                    1..=8 => {
                        self.read_plain_nbt(t, buff)
                    },
                    9 => {
                        self.read_list(buff)
                    },
                    10 => {
                        self.read_compound(buff)
                    },
                    _ => NBT::String("Invalid tag!!".to_string())
                };

                container.insert(name, value);
            }

            return NBT::Compound(container);
        }

        pub fn read_list(&mut self, buff: &mut [u8]) -> NBT {
            self.read_n(buff, 1);
            let list_t = buff[0];

            self.read_n(buff, 4);
            let n = u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]) as usize;

            let mut container: Vec<NBT> = Vec::with_capacity(n);

            match list_t {
                1..=8 => {
                    for _ in 0..n {
                        container.push(self.read_plain_nbt(list_t, buff));
                    }
                }
                9 => {
                    for _ in 0..n {
                        container.push(self.read_list(buff));
                    }
                }
                10 => {
                    for _ in 0..n {
                        container.push(self.read_compound(buff));
                    }
                }
                _ => {}
            };

            return NBT::List(container);
        }
    }

    impl<T: Read> Read for ReadWrapper<T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            return self.inner.read(buf);
        }
    }

    impl NBT {
        pub fn as_mut_compound(&mut self) -> Result<&mut LinkedHashMap<String, NBT>, Error> {
            match self {
                NBT::Compound(v) => Result::Ok(v),
                _ => Result::Err(Error::new(ErrorKind::Other, "not a compound"))
            }
        }
        pub fn as_compound(&self) -> Result<&LinkedHashMap<String, NBT>, Error> {
            match self {
                NBT::Compound(v) => Result::Ok(v),
                _ => Result::Err(Error::new(ErrorKind::Other, "not a compound"))
            }
        }

        pub fn get(&self, name: &str) -> Option<&NBT> {
            match self {
                NBT::Compound(v) => v.get(name),
                _ => Option::None
            }
        }

        pub fn get_mut(&mut self, name: &str) -> Option<&mut NBT> {
            match self {
                NBT::Compound(v) => v.get_mut(name),
                _ => Option::None
            }
        }

        pub fn new() {}

        pub fn from<T: Read>(input: &mut T) -> Result<NBT, Error> {
            let mut src = ReadWrapper::from(input);
            let buff = &mut vec![0u8; BUFF_SIZE];
            let mut res = src.read_compound(buff.as_mut_slice());
            return Ok(res.as_mut_compound().unwrap().remove("empty").unwrap());
        }

        fn rec_fmt(&self, f: &mut fmt::Formatter<'_>, padding: &str) -> fmt::Result {
            match self {
                NBT::Compound(v) => {
                    let mut new_padding = padding.to_owned();
                    new_padding.push_str("  ");
                    for (k, value) in v {
                        write!(f, "{}{}: ", padding, k)?;
                        match value {
                            NBT::Compound(tmp) => {
                                writeln!(f, "({}) entries", tmp.len())?;
                                value.rec_fmt(f, &new_padding)?;
                            }
                            NBT::List(_) => {
                                value.rec_fmt(f, &new_padding)?;
                                writeln!(f, "")?;
                            }
                            _ => {
                                value.rec_fmt(f, "")?;
                                writeln!(f, "")?;
                            }
                        };
                    }
                }
                NBT::List(v) => {
                    if let NBT::Compound(_) = v.first().unwrap_or(&NBT::Byte(1)) {
                        let mut new_padding = padding.to_owned();
                        new_padding.push_str("  ");
                        writeln!(f, "({}) entries [", v.len())?;
                        for value in v {
                            value.rec_fmt(f, &new_padding)?;
                            writeln!(f, "{},", padding)?;
                        }
                        write!(f, "{}]", padding)?;
                    } else {
                        write!(f, "({}) entries [", v.len())?;
                        let mut values = Vec::with_capacity(v.len());
                        for i in v {
                            values.push(i.to_string());
                        }
                        write!(f, "{}]", values.join(", "))?;
                    }
                }
                NBT::Byte(v) => {
                    write!(f, "0x{:0>2X}", v)?;
                }
                NBT::Short(v) => {
                    write!(f, "{}", v)?;
                }
                NBT::Int(v) => {
                    write!(f, "{}", v)?;
                }
                NBT::Long(v) => {
                    write!(f, "{}", v)?;
                }
                NBT::Float(v) => {
                    write!(f, "{}", v)?;
                }
                NBT::Double(v) => {
                    write!(f, "{}", v)?;
                }
                NBT::String(v) => {
                    write!(f, "\"{}\"", v)?;
                }
                _ => {
                    write!(f, "{:?}", self)?;
                }
            };
            return Ok(());
        }
    }

    impl fmt::Display for NBT {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.rec_fmt(f, "")
        }
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    use std::fs::File;
    use crate::nbt::NBT;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::io::Read;
    use test::Bencher;
    use std::convert::TryInto;

    #[test]
    fn it_works() {
        let mut f = File::open("Bikini Bottom V2/level").unwrap();
        let mut buff = vec![0u8; 16];
        let res = NBT::from(&mut f).unwrap();
        println!("{:?}", res);
    }

    #[bench]
    fn bench_read(b: &mut Bencher) {
        let mut f = File::open("Bikini Bottom V2/level").unwrap();
        let mut buff = Vec::new();
        f.read_to_end(&mut buff);
        let mut v = vec![];
        b.iter(|| {
            let mut s = buff.as_slice();
            let res = NBT::from(&mut s).unwrap();
            println!("{}", res.as_compound().unwrap().len());
            v.push(1);
        });
        println!("{}", v.len());
    }
}


