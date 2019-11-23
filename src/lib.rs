

pub mod nbt {
    extern crate bytes;
//    use std::collections::HashMap;
    use std::io::{Read, Error, ErrorKind};
    use bytes::BigEndian;
    use bytes::ByteOrder;

    use linked_hash_map::LinkedHashMap;
    use core::fmt;

    static BUFF_SIZE: usize = 65536;

    #[derive(Debug)]
    pub enum NBT{
        Byte(u8),
        Short(i16),
        Int(i32),
        Long(i64),
        Float(f32),
        Double(f64),
        ByteArray(Vec<u8>),
        String(String),
        List(Vec<NBT>),
        Compound(LinkedHashMap<String, NBT>)

    }

//    #[derive(Debug)]
//    pub struct NBT {
//        pub name: String,
//        pub data: NBT
//    }

    pub struct ReadWrapper<T: Read> {
        inner: T,
        name_buff: [u8; 2]
    }

    impl<T: Read> ReadWrapper<T> {
        pub fn from(input: T) -> ReadWrapper<T> {
            return ReadWrapper{
                inner: input,
                name_buff: [0u8; 2]
            };
        }

        pub fn read_n(&mut self, buff: &mut Vec<u8>, n: usize) {
            self.read(&mut buff[0..n]);
        }

        pub fn read_name(&mut self) -> String {
            let mut name_buff = &mut [0u8; 2];
            self.read(name_buff);
            let mut n = BigEndian::read_u16(name_buff) as usize;
            if n == 0{
                String::new()
            } else{
                let mut buff = vec![0; n];
                self.read(&mut buff);

                String::from_utf8(buff).unwrap()
            }
        }

        pub fn read_plain_nbt(&mut self, t: u8, buff: &mut Vec<u8>) -> NBT {
            match t {
                0x01 => { // Byte
                    self.read_n(buff, 1);
                    NBT::Byte(buff[0])
                },
                0x02 => { // Short
                    self.read_n(buff, 2);
                    NBT::Short(BigEndian::read_i16(buff))
                },
                0x03 => { // Int
                    self.read_n(buff, 4);
                    NBT::Int(BigEndian::read_i32(buff))
                },
                0x04 => { // Long
                    self.read_n(buff, 8);
                    NBT::Long(BigEndian::read_i64(buff))
                },
                0x05 => { // Float
                    self.read_n(buff, 4);
                    NBT::Float(BigEndian::read_f32(buff))
                },
                0x06 => { // Double
                    self.read_n(buff, 8);
                    NBT::Double(BigEndian::read_f64(buff))
                },
                0x07 => { // Byte Array
                    self.read_n(buff, 4);
                    let n = BigEndian::read_u32(buff) as usize;
                    let mut cur = 0usize;

                    let mut res: Vec<u8> = Vec::with_capacity(n);

                    while cur < n {
                        cur += self.read(buff).unwrap();
                        res.extend_from_slice(buff.as_mut_slice());
                    }

                    NBT::ByteArray(res)
                },
                0x08 => {
                    NBT::String(self.read_name())
                },
                _ => NBT::String(String::from("Invalid tag!!!"))
            }
        }
    }

    impl<T: Read> Read for ReadWrapper<T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            return self.inner.read(buf);
        }
    }

    impl NBT {
        pub fn as_mut_compound(&mut self) -> Result<&mut LinkedHashMap<String, NBT>, Error>{
            match self {
                NBT::Compound(v) => Result::Ok(v),
                _ => Result::Err(Error::new(ErrorKind::Other, "not a compound"))
            }
        }
        pub fn as_compound(&self) -> Result<&LinkedHashMap<String, NBT>, Error>{
            match self {
                NBT::Compound(v) => Result::Ok(v),
                _ => Result::Err(Error::new(ErrorKind::Other, "not a compound"))
            }
        }

        pub fn new() {

        }
        pub fn from(input: &mut impl Read) -> Result<NBT, Error> {

            let mut stack: Vec<(String, NBT)> = vec![];
            let mut src = ReadWrapper::from(input);
            let buff = &mut vec![0u8; BUFF_SIZE];

            loop {
                src.read_n(buff, 1);
                let t = buff[0];

                if t == 0 {
                    let (name, closed_tag) = stack.pop().unwrap();
                    if stack.len() > 0 {
                        stack.last_mut().unwrap().1.as_mut_compound().unwrap().insert(name, closed_tag);
                        continue
                    } else {
                        return Ok(closed_tag);
                    }
                }

                let name = src.read_name();
                let v = match t {
                    0x01..=0x08 => {
                        src.read_plain_nbt(t, buff)
                    },
                    0x09 => { // List
                        src.read_n(buff, 1);
                        let list_t = buff[0];
                        src.read_n(buff, 4);
                        let n = u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]) as usize;
                        let mut res: Vec<NBT> = Vec::with_capacity(n);
                        for _ in 0..n {
                            res.push(src.read_plain_nbt(list_t, buff));
                        }

                        NBT::List(res)
                    }, // Compound
                    0x0A => {
                        stack.push((name, NBT::Compound(LinkedHashMap::new())));
                        continue
                    }
                    _ => NBT::String(String::from("Invalid tag!!!"))
                };

                let mut parent = stack.last_mut().unwrap().1.as_mut_compound().unwrap();
                parent.insert(name, v);
            }
        }

        fn rec_fmt(&self, f: &mut fmt::Formatter<'_>, padding: &str) -> fmt::Result {
            match self {
                NBT::Compound(v) => {
                    let mut new_padding = padding.to_owned();
                    new_padding.push_str("  ");
                    for (k, value) in self.as_compound().unwrap() {
                        write!(f, "{}", padding);
                        write!(f, "{}: ", k);

                        if let NBT::Compound(_) = value {
                            writeln!(f, "");
                            value.rec_fmt(f, &new_padding);
                        } else {
                            value.rec_fmt(f, "");
                            writeln!(f, "");
                        }


                    }
                }
                _ => {
                    write!(f, "{:?}", self);
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
    use std::fs::File;
    use crate::nbt::{NBT};
    use std::collections::HashMap;
    use std::fmt::Debug;

    #[test]
    fn it_works() {
        let mut f = File::open("Bikini Bottom V2/level").unwrap();
        let mut buff = vec![0u8; 16];
        let res = NBT::from(&mut f);
        println!("{:?}", res);

//        let s = String::from("Kappa");
//        let v = NBT {
//            name: s.clone(),
//            data: NBT::Byte(12u8)
//        };
//        let mut m: HashMap<String, Box<NBT>> = HashMap::new();
//        m.insert(s, Box::from(v));
//        let mut n = NBT {
//            name: String::from("dasdsa"),
//            data: NBT::Compound(m)
//        };

//        println!("{:?}", n);
    }
}
