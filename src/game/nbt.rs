use crate::data_writer::DataWriter;
use std::any::Any;
use std::collections::HashMap;
use std::iter::Map;

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
    Compound { compound: Vec<CompoundElement> },
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

    pub fn write<'a>(&self, data: &'a mut Vec<u8>, name: Option<&String>) -> &'a mut Vec<u8> {
        data.push(self.type_id());
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
            NBTTag::ByteArray { bytes } => data.extend(bytes),
            NBTTag::String { string } => {
                data.extend_from_slice(&(string.len() as i16).to_be_bytes());
                data.extend_from_slice(string.as_bytes());
            }
            NBTTag::List { list, type_id } => {
                data.push(*type_id);
                data.extend_from_slice(&(list.len() as i32).to_be_bytes());
                for element in list {
                    element.write(data, None);
                }
            }
            NBTTag::Compound { compound } => {
                for element in compound {
                    element.tag.write(data, Some(&element.name));
                }
                data.push(0);
            }
            NBTTag::IntArray { array } => {
                data.extend_from_slice(&array.len().to_be_bytes());
                for x in array {
                    data.extend_from_slice(&x.to_be_bytes());
                }
            }
            NBTTag::LongArray { array } => {
                data.extend_from_slice(&array.len().to_be_bytes());
                for x in array {
                    data.extend_from_slice(&x.to_be_bytes());
                }
            }
        }

        data
    }
}
