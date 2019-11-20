

pub mod nbt {
    use std::collections::HashMap;
    use std::io::{Read, Error};

    #[derive(Debug)]
    pub enum NBTPayload{
        Byte(u8),
        Short(u16),
        Int(u32),
        Long(u64),
        Float(f32),
        Double(f64),
        ByteArray(Vec<u8>),
        String(String),
        List(Vec<NBTPayload>),
        Compound(HashMap<String, Box<NBT>>)

    }

    #[derive(Debug)]
    pub struct NBT {
        pub name: String,
        pub data: NBTPayload
    }

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
                n = u16::from_be_bytes([buff[0], buff[1]]) as usize;
            }

            let mut buff = vec![0; n];
            self.read(&mut buff);
            return String::from_utf8(buff).unwrap();
        }

        pub fn readPlainNBTPayload(&mut self, t: u8, buff: &mut Vec<u8>, buff_size: usize) -> NBTPayload {
            match t {
                0x01 => { // Byte
                    self.read_n(buff, 1);
                    NBTPayload::Byte(buff[0])
                },
                0x02 => { // Short
                    self.read_n(buff, 2);
                    NBTPayload::Short((buff[0] as u16) << 8 | buff[1] as u16)
                },
                0x03 => { // Int
                    self.read_n(buff, 4);
                    NBTPayload::Int(u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]))
                },
                0x04 => { // Long
                    self.read_n(buff, 8);
                    NBTPayload::Long(u64::from_be_bytes([buff[0], buff[1], buff[2], buff[3], buff[4], buff[5], buff[6], buff[7]]))
                },
                0x05 => { // Float
                    self.read_n(buff, 4);
                    NBTPayload::Float(f32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]))
                },
                0x06 => { // Double
                    self.read_n(buff, 8);
                    NBTPayload::Double(f64::from_be_bytes([buff[0], buff[1], buff[2], buff[3], buff[4], buff[5], buff[6], buff[7]]))
                },
                0x07 => { // Byte Array
                    self.read_n(buff, 4);
                    let n = u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]) as usize;
                    let step = n / buff_size;
                    let mut res: Vec<u8> = Vec::new();
                    for i in 0..step {
                        self.read_n(buff, buff_size);
                        res.extend_from_slice(buff.as_mut_slice());
                    }
                    let last_part = n  - step * buff_size;
                    self.read_n(buff,  last_part);
                    res.extend_from_slice(&mut buff[0..last_part]);

                    NBTPayload::ByteArray(res)
                },
                0x08 => {
                    NBTPayload::String(self.read_name())
                },
                _ => NBTPayload::String(String::from("Invalid tag!!!"))
            }
        }
    }

    impl<T: Read> Read for ReadWrapper<T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            return self.inner.read(buf);
        }
    }

    impl NBT {


        pub fn new() {

        }
        pub fn from(input: &mut impl Read, buff_size: usize) {
            let buff_size = 65536;
            let mut src = ReadWrapper::from(input);
            let mut bufff = vec![0u8; buff_size];
            let buff = &mut bufff;
            loop {
                src.read_n(buff, 1);
                let t = buff[0];

                let name = src.read_name();
                let v = match t {
                    0x01..=0x08 => {
                        src.readPlainNBTPayload(t, buff, buff_size)
                    },
                    0x09 => { // List
                        src.read_n(buff, 1);
                        let list_t = buff[0];
                        src.read_n(buff, 4);
                        let n = u32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]]) as usize;
                        let mut res: Vec<NBTPayload> = Vec::with_capacity(n);
                        for i in 0..n {
                            res.push(src.readPlainNBTPayload(list_t, buff, buff_size));
                        }

                        NBTPayload::List(res)
                    }, // Compound
                    0x10 => {
                        NBTPayload::String(String::from("lul"))
                    }
                    _ => NBTPayload::String(String::from("Invalid tag!!!"))
                };
            }

        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::nbt::{NBT, NBTPayload};
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let mut f = File::open("Bikini Bottom V2/level").unwrap();
        let mut buff = vec![0u8; 16];
        let s = String::from("Kappa");
        let v = NBT {
            name: s.clone(),
            data: NBTPayload::Byte(12u8)
        };
        let mut m: HashMap<String, Box<NBT>> = HashMap::new();
        m.insert(s, Box::from(v));
        let mut n = NBT {
            name: String::from("dasdsa"),
            data: NBTPayload::Compound(m)
        };

        println!("{:?}", n);
    }
}
