

pub mod nbt {
    extern crate bytes;
    use std::collections::HashMap;
    use std::io::{Read, Error, ErrorKind};
    use std::convert::TryInto;
    use std::iter::Map;
    use bytes::BigEndian;
    use self::bytes::ByteOrder;

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
        Compound(HashMap<String, NBT>)

    }

//    #[derive(Debug)]
//    pub struct NBT {
//        pub name: String,
//        pub data: NBT
//    }

    pub struct ReadWrapper<T: Read> {
        inner: T
    }

    impl<T: Read> ReadWrapper<T> {
        pub fn from(input: T) -> ReadWrapper<T> {
            return ReadWrapper{
                inner: input
            };
        }

        pub fn read_n(&mut self, buff: &mut Vec<u8>, n: usize) {
            self.read(&mut buff[0..n]);
        }

        pub fn read_name(&mut self) -> String {
            let mut n = 0;
            {
                let mut buff = vec![0; 2];
                self.read(&mut buff[0..2]);
                n = BigEndian::read_u16(&mut buff) as usize;
            }

            let mut buff = vec![0; n];
            self.read(&mut buff);
            return String::from_utf8(buff).unwrap();
        }

        pub fn readPlainNBT(&mut self, t: u8, buff: &mut Vec<u8>, buff_size: usize) -> NBT {
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
                    let step = n / buff_size;
                    let mut res: Vec<u8> = Vec::new();
                    for i in 0..step {
                        self.read_n(buff, buff_size);
                        res.extend_from_slice(buff.as_mut_slice());
                    }
                    let last_part = n  - step * buff_size;
                    self.read_n(buff,  last_part);
                    res.extend_from_slice(&mut buff[0..last_part]);

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
        pub fn as_compound(&mut self) -> Result<&mut HashMap<String, NBT>, Error>{
            match self {
                NBT::Compound(v) => Result::Ok(v),
                _ => Result::Err(Error::new(ErrorKind::Other, "not a compound"))
            }
        }

        pub fn new() {

        }
        pub fn from(input: &mut impl Read, buff_size: usize) {
            let buff_size = 65536;

            let mut stack: Vec<NBT> = vec![NBT::Compound(HashMap::new())];
            let mut src = ReadWrapper::from(input);
            let mut bufff = vec![0u8; buff_size];
            let buff = &mut bufff;
            loop {
                src.read_n(buff, 1);
                let t = buff[0];

                let name = src.read_name();
                let v = match t {
                    0x01..=0x08 => {
                        src.readPlainNBT(t, buff, buff_size)
                    },
                    0x09 => { // List
                        src.read_n(buff, 1);
                        let list_t = buff[0];
                        src.read_n(buff, 4);
                        let n = u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]) as usize;
                        let mut res: Vec<NBT> = Vec::with_capacity(n);
                        for i in 0..n {
                            res.push(src.readPlainNBT(list_t, buff, buff_size));
                        }

                        NBT::List(res)
                    }, // Compound
                    0x10 => {
                        NBT::String(String::from("lul"))
                    }
                    _ => NBT::String(String::from("Invalid tag!!!"))
                };

                let mut parent: &mut HashMap<String, NBT> = stack.last_mut().unwrap().as_compound().unwrap();
                parent.insert(name, v);
            }

        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::nbt::{NBT};
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let mut f = File::open("Bikini Bottom V2/level").unwrap();
        let mut buff = vec![0u8; 16];
        NBT::from(&mut f, 65536);
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
