use std::collections::HashMap;
use std::io::Write;

pub enum Error {
    Error
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum Value {
    Null,
    Boolean(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Map(HashMap<Value, Value>)
}

impl PartialEq for Value {
    fn eq(&self, that: &Value) -> bool {
        match self {
            Value::Null => match that {
                Value::Null => true,
                _ => false
            },
            Value::Boolean(b) => match that {
                Value::Boolean(b2) => b == b2,
                _ => false
            },
            Value::Integer(i) => match that {
                Value::Integer(i2) => i == i2,
                _ => false
            },
            Value::Float(f) => match that {
                Value::Float(f2) => f == f2,
                _ => false
            },
            Value::String(s) => match that {
                Value::String(s2) => s == s2,
                _ => false
            },
            Value::Bytes(b) => match that {
                Value::Bytes(b2) => b == b2,
                _ => false
            },
            Value::Array(a) => match that {
                Value::Array(a2) => a == a2,
                _ => false
            },
            Value::Map(m) => match that {
                Value::Map(m2) => {
                    if m.len() == m2.len() {
                        let mut eq = true;
                        for (k, v) in m {
                            match m2.get(k) {
                                Some(v2) => eq = eq && v == v2,
                                None => eq = false
                            }
                        }
                        eq
                    } else {
                        false
                    }
                },
                _ => false
            }
        }
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Bytes(b) => b.hash(state),
            Value::Array(a) => a.hash(state),
            Value::Map(m) => for (k, v) in m {
                k.hash(state);
                v.hash(state);
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Error {
        Error::Error
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_e: std::string::FromUtf8Error) -> Error {
        Error::Error
    }
}

pub fn encode_to(w: &mut dyn Write, value: Value) -> Result<()> {
    match value {
        Value::Null => {
            w.write(&[0xc0])?;
            Ok(())
        },
        Value::Boolean(b) => {
            let v: u8 = if b { 0xc3 } else { 0xc2 };
            w.write(&[v])?;
            Ok(())
        },
        Value::Integer(i) => {
            if 0 <= i && i <= 0x7f {
                w.write(&[i as u8])?;
                Ok(())
            } else if -32 <= i && i <= -1 {
                w.write(&[i as u8])?;
                Ok(())
            } else if -128 <= i && i <= 127 {
                w.write(&[0xd0, i as u8])?;
                Ok(())
            } else if -32768 <= i && i <= 32767 {
                w.write(&[0xd1])?;
                w.write(&(i as u16).to_be_bytes())?;
                Ok(())
            } else if -2147483648 <= i && i <= 2147483647 {
                w.write(&[0xd2])?;
                w.write(&(i as u32).to_be_bytes())?;
                Ok(())
            } else {
                w.write(&[0xd3])?;
                w.write(&i.to_be_bytes());
                Ok(())
            }
        }
        Value::Float(f) => {
            w.write(&[0xcb]);
            w.write(&f.to_bits().to_be_bytes());
            Ok(())
        },
        Value::String(s) => {
            let len = s.len();
            if s.len() <= 31 {
                w.write(&[0xa0 | len as u8])?;
                w.write(&s.as_bytes())?;
                Ok(())
            } else if s.len() <= 255 {
                w.write(&[0xd9, len as u8])?;
                w.write(&s.as_bytes())?;
                Ok(())
            } else if s.len() <= 65535 {
                w.write(&[0xd9])?;
                w.write(&(len as u16).to_be_bytes())?;
                w.write(&s.as_bytes())?;
                Ok(())
            } else {
                w.write(&[0xd9])?;
                w.write(&(len as u32).to_be_bytes())?;
                w.write(&s.as_bytes())?;
                Ok(())
            }
        },
        Value::Bytes(b) => {
            let len = b.len();
            if len <= 255 {
                w.write(&[0xc4, len as u8])?;
                w.write(&b)?;
                Ok(())
            } else if len <= 65535 {
                w.write(&[0xc5])?;
                w.write(&(len as u16).to_be_bytes())?;
                w.write(&b)?;
                Ok(())
            } else {
                let buf = [0xc6];
                w.write(&[0xc6])?;
                w.write(&(len as u32).to_be_bytes())?;
                w.write(&b)?;
                Ok(())
            }
        },
        Value::Array(a) => {
            let len = a.len();
            if len <= 15 {
                w.write(&[0x90 | len as u8])?;
                for v in a {
                    encode_to(w, v)?;
                }
                Ok(())
            } else if len <= 65535 {
                w.write(&[0xdc])?;
                w.write(&(len as u16).to_be_bytes())?;
                for v in a {
                    encode_to(w, v)?;
                }
                Ok(())
            } else {
                w.write(&[0xdd])?;
                w.write(&(len as u16).to_be_bytes())?;
                for v in a {
                    encode_to(w, v)?;
                }
                Ok(())
            }
        }
        Value::Map(m) => {
            let len = m.len();
            if len <= 15 {
                w.write(&[0x80 | len as u8])?;
                for (k, v) in m {
                    encode_to(w, k)?;
                    encode_to(w, v)?;
                }
                Ok(())
            } else if len <= 65535 {
                w.write(&[0xde])?;
                w.write(&(len as u16).to_be_bytes())?;
                for (k, v) in m {
                    encode_to(w, k)?;
                    encode_to(w, v)?;
                }
                Ok(())
            } else {
                w.write(&[0xdf])?;
                w.write(&(len as u32).to_be_bytes())?;
                for (k, v) in m {
                    encode_to(w, k)?;
                    encode_to(w, v)?;
                }
                Ok(())
            }
        }
    }
}

pub fn decode_from(r: &mut dyn std::io::Read) -> Result<Value> {
    let mut b: u8 = 0;
    r.read(std::slice::from_mut(&mut b))?;
    match b {
        0x00..=0x7f => Ok(Value::Integer(b as i128)),
        0x80..=0x8f => {
            let len = b & 0xf;
            let mut m = HashMap::new();
            for _i in 0..len {
                let k = decode_from(r)?;
                let v = decode_from(r)?;
                m.insert(k, v);
            }
            Ok(Value::Map(m))
        },
        0x90..=0x9f => {
            let len = b & 0xf;
            let mut v = Vec::new();
            for _i in 0..len {
                v.push(decode_from(r)?);
            }
            Ok(Value::Array(v))
        },
        0xa0..=0xbf => {
            let len = b & 0x1f;
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v)?;
            Ok(Value::String(String::from_utf8(v)?))
        },
        0xc0 => Ok(Value::Null),
        0xc1 => Err(Error::Error),
        0xc2 => Ok(Value::Boolean(false)),
        0xc3 => Ok(Value::Boolean(true)),
        0xc4 => {
            let mut len: u8 = 0;
            r.read(std::slice::from_mut(&mut len))?;
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v)?;
            Ok(Value::Bytes(v))
        },
        0xc5 => {
            let mut l = [0 as u8; 2];
            r.read(&mut l)?;
            let len = u16::from_be_bytes(l);
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v)?;
            Ok(Value::Bytes(v))
        },
        0xc6 => {
            let mut l = [0 as u8; 4];
            r.read(&mut l)?;
            let len = u32::from_be_bytes(l);
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v)?;
            Ok(Value::Bytes(v))
        },
        0xc7 => unimplemented!(),
        0xc8 => unimplemented!(),
        0xc9 => unimplemented!(),
        0xca => {
            let mut buf = [0 as u8; 4];
            r.read(&mut buf)?;
            Ok(Value::Float(f32::from_be_bytes(buf) as f64))
        },
        0xcb => {
            let mut buf = [0 as u8; 8];
            r.read(&mut buf)?;
            Ok(Value::Float(f64::from_be_bytes(buf)))
        },
        0xcc => {
            let mut v: u8 = 0;
            r.read(std::slice::from_mut(&mut v))?;
            Ok(Value::Integer(v as i128))
        }
        0xcd => {
            let mut buf = [0 as u8; 2];
            r.read(&mut buf)?;
            Ok(Value::Integer(u16::from_be_bytes(buf) as i128))
        },
        0xce => {
            let mut buf = [0 as u8; 4];
            r.read(&mut buf)?;
            Ok(Value::Integer(u32::from_be_bytes(buf) as i128))
        },
        0xcf => {
            let mut buf = [0 as u8; 8];
            r.read(&mut buf)?;
            Ok(Value::Integer(u64::from_be_bytes(buf) as i128))
        },
        0xd0 => {
            let mut v: u8 = 0;
            r.read(std::slice::from_mut(&mut v))?;
            Ok(Value::Integer((v as i8) as i128))
        },
        0xd1 => {
            let mut buf = [0 as u8; 2];
            r.read(&mut buf)?;
            Ok(Value::Integer(i16::from_be_bytes(buf) as i128))
        },
        0xd2 => {
            let mut buf = [0 as u8; 4];
            r.read(&mut buf)?;
            Ok(Value::Integer(i32::from_be_bytes(buf) as i128))
        },
        0xd3 => {
            let mut buf = [0 as u8; 8];
            r.read(&mut buf)?;
            Ok(Value::Integer(i64::from_be_bytes(buf) as i128))
        },
        0xd4 => unimplemented!(),
        0xd5 => unimplemented!(),
        0xd6 => unimplemented!(),
        0xd7 => unimplemented!(),
        0xd8 => unimplemented!(),
        0xd9 => {
            let mut len: u8 = 0;
            r.read(std::slice::from_mut(&mut len))?;
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v)?;
            Ok(Value::String(String::from_utf8(v)?))
        },
        0xda => {
            let mut buf = [0; 2];
            r.read(&mut buf)?;
            let len = u16::from_be_bytes(buf);
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v);
            Ok(Value::String(String::from_utf8(v)?))
        },
        0xdb => {
            let mut buf = [0; 4];
            r.read(&mut buf)?;
            let len = u32::from_be_bytes(buf);
            let mut v = vec![0 as u8; len as usize];
            r.read(&mut v);
            Ok(Value::String(String::from_utf8(v)?))
        },
        0xdc => {
            
        },
        0xdd => unimplemented!(),
        0xde => unimplemented!(),
        0xdf => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
