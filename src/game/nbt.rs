use crate::data_writer::DataWriter;
use std::any::Any;
use std::collections::HashMap;
use std::io::{Error, Read};
use std::iter::Map;
use std::string::FromUtf8Error;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use fxhash::{FxBuildHasher, FxHashMap};

#[derive(Debug)]
pub enum NBTTag {
    End,
    Byte { byte: i8 },
    Short { short: i16 },
    Int { int: i32 },
    Long { long: i64 },
    Float { float: f32 },
    Double { double: f64 },
    ByteArray { bytes: Vec<u8> },
    String { string: String },
    List { list: Vec<NBTTag>, type_id: u8 },
    Compound { compound: HashMap<String, NBTTag, FxBuildHasher> },
    IntArray { array: Vec<i32> },
    LongArray { array: Vec<i64> },
}

pub struct CompoundElement {
    pub name: String,
    pub tag: NBTTag,
}

impl NBTTag {
    pub const fn type_id(&self) -> u8 {
        match self {
            NBTTag::End => 0,
            NBTTag::Byte { .. } => 1,
            NBTTag::Short { .. } => 2,
            NBTTag::Int { .. } => 3,
            NBTTag::Long { .. } => 4,
            NBTTag::Float { .. } => 5,
            NBTTag::Double { .. } => 6,
            NBTTag::ByteArray { .. } => 7,
            NBTTag::String { .. } => 8,
            NBTTag::List { .. } => 9,
            NBTTag::Compound { .. } => 10,
            NBTTag::IntArray { .. } => 11,
            NBTTag::LongArray { .. } => 12,
        }
    }

    pub fn write<'a>(&self, data: &'a mut Vec<u8>, name: Option<&String>, include_type_id: bool) {
        if include_type_id {
            data.push(self.type_id());
        }
        if name.is_some() {
            let name = name.unwrap();
            data.extend_from_slice(&(name.len() as i16).to_be_bytes());
            data.extend_from_slice(name.as_bytes());
        }
        match self {
            NBTTag::End => {}
            NBTTag::Byte { byte } => data.push(*byte as u8),
            NBTTag::Short { short } => data.extend_from_slice(&short.to_be_bytes()),
            NBTTag::Int { int } => data.extend_from_slice(&int.to_be_bytes()),
            NBTTag::Long { long } => data.extend_from_slice(&long.to_be_bytes()),
            NBTTag::Float { float } => data.extend_from_slice(&float.to_be_bytes()),
            NBTTag::Double { double } => data.extend_from_slice(&double.to_be_bytes()),
            NBTTag::ByteArray { bytes } => {
                data.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
                data.extend(bytes);
            },
            NBTTag::String { string } => {
                data.extend_from_slice(&(string.len() as i16).to_be_bytes());
                data.extend_from_slice(string.as_bytes());
            }
            NBTTag::List { list, type_id } => {
                data.push(*type_id);
                data.extend_from_slice(&(list.len() as i32).to_be_bytes());
                for element in list {
                    element.write(data, None, false);
                }
            }
            NBTTag::Compound { compound } => {
                for element in compound {
                    element.1.write(data, Some(&element.0), true);
                }
                data.push(0);
            }
            NBTTag::IntArray { array } => {
                data.extend_from_slice(&(array.len() as i32).to_be_bytes());
                for x in array {
                    data.extend_from_slice(&x.to_be_bytes());
                }
            }
            NBTTag::LongArray { array } => {
                data.extend_from_slice(&(array.len() as i32).to_be_bytes());
                for x in array {
                    data.extend_from_slice(&x.to_be_bytes());
                }
            }
        }
    }

    pub fn read<R: Read>(data: &mut R, read_name: bool, type_id: Option<u8>) -> Result<(NBTTag, Option<String>), NBTParseError> {
        let type_id = match type_id {
            None => data.read_u8()?,
            Some(x) => x
        };

        if type_id == 0 {
            return Ok((NBTTag::End, None))
        }

        let mut string = None;

        if read_name {
            let name_size = data.read_i16::<BigEndian>()?;
            let mut vec = vec![0u8; name_size as usize];
            data.read(&mut vec);
            string = Some(String::from_utf8(vec)?);
        }

        match type_id {
            1 => Ok((NBTTag::Byte {byte: data.read_i8()?}, string)),
            2 => Ok((NBTTag::Short {short: data.read_i16::<BigEndian>()?}, string)),
            3 => Ok((NBTTag::Int {int: data.read_i32::<BigEndian>()?}, string)),
            4 => Ok((NBTTag::Long {long: data.read_i64::<BigEndian>()?}, string)),
            5 => Ok((NBTTag::Float {float: data.read_f32::<BigEndian>()?}, string)),
            6 => Ok((NBTTag::Double {double: data.read_f64::<BigEndian>()?}, string)),
            7 => {
                let size = data.read_i32::<BigEndian>()?;
                let mut bytes = vec![0u8; size as usize];
                data.read(&mut bytes);
                Ok((NBTTag::ByteArray {bytes}, string))
            }
            8 => {
                let name_size = data.read_i16::<BigEndian>()?;
                let mut vec = vec![0u8; name_size as usize];
                data.read(&mut vec);
                Ok((NBTTag::String {string: String::from_utf8(vec)?}, string))
            }
            9 => {
                let type_id = data.read_u8()?;
                let length = data.read_i32::<BigEndian>()?;

                let mut list = Vec::with_capacity(length as usize);

                for _ in 0..length {
                    let element = NBTTag::read(data, false, Some(type_id))?;
                    list.push(element.0);
                }

                Ok((NBTTag::List {list, type_id}, string))
            }
            10 => {
                let mut map = FxHashMap::default();

                loop {
                    let element = NBTTag::read(data, true, None)?;

                    if let NBTTag::End = element.0 {
                        break;
                    }

                    map.insert(element.1.unwrap(), element.0);
                }

                Ok((NBTTag::Compound {compound: map}, string))
            }
            11 => {
                let length = data.read_i32::<BigEndian>()?;
                let mut array = vec![0i32; length as usize];

                for i in 0..length as usize {
                    array[i] = data.read_i32::<BigEndian>()?;
                }

                Ok((NBTTag::IntArray {array}, string))
            }
            12 => {
                let length = data.read_i32::<BigEndian>()?;
                let mut array = vec![0i64; length as usize];

                for i in 0..length as usize {
                    array[i] = data.read_i64::<BigEndian>()?;
                }

                Ok((NBTTag::LongArray {array}, string))
            }
            _ => Err(NBTParseError::InvalidTypeId)
        }
    }
}

#[derive(Debug)]
pub enum NBTParseError {
    InvalidTypeId,
    IOError(std::io::Error),
    UTF8Error(FromUtf8Error),
    Test
}

impl From<std::io::Error> for NBTParseError {
    fn from(e: Error) -> Self {
        NBTParseError::IOError(e)
    }
}

impl From<FromUtf8Error> for NBTParseError {
    fn from(e: FromUtf8Error) -> Self {
        NBTParseError::UTF8Error(e)
    }
}