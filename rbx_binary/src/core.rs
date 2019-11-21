use std::io::{self, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rbx_dom_weak::{RbxValue, RbxValueType};

pub static FILE_MAGIC_HEADER: &[u8] = b"<roblox!";
pub static FILE_SIGNATURE: &[u8] = b"\x89\xff\x0d\x0a\x1a\x0a";
pub const FILE_VERSION: u16 = 0;

pub trait BinaryType<T: ?Sized + 'static> {
    fn read_array<R: Read>(source: &mut R, count: usize) -> io::Result<Vec<RbxValue>>;

    fn write_array<'write, I, W: Write>(output: &mut W, values: I) -> io::Result<()>
    where
        I: Iterator<Item = &'write T>;
}

pub trait RbxReadExt: Read {
    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_u32::<LittleEndian>()?;

        let mut value = String::with_capacity(length as usize);
        self.take(length as u64).read_to_string(&mut value)?;

        Ok(value)
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }
}

impl<R> RbxReadExt for R where R: Read {}

pub trait RbxWriteExt: Write {
    fn write_string(&mut self, value: &str) -> io::Result<()> {
        self.write_u32::<LittleEndian>(value.len() as u32)?;
        write!(self, "{}", value)?;

        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_u8(value as u8)
    }

    fn write_interleaved_i32_array<I>(&mut self, values: I) -> io::Result<()>
    where
        I: Iterator<Item = i32>,
    {
        let values: Vec<_> = values.collect();

        for shift in &[24, 16, 8, 0] {
            for value in values.iter().copied() {
                let encoded = transform_i32(value) >> shift;
                self.write_u8(encoded as u8)?;
            }
        }

        Ok(())
    }

    fn write_referents<I>(&mut self, values: I) -> io::Result<()>
    where
        I: Iterator<Item = i32>,
    {
        let mut last_value = 0;
        let delta_encoded = values.map(|value| {
            let encoded = value - last_value;
            last_value = value;
            encoded
        });

        self.write_interleaved_i32_array(delta_encoded)
    }
}

impl<W> RbxWriteExt for W where W: Write {}

pub fn id_from_value_type(value_type: RbxValueType) -> Option<u8> {
    use RbxValueType::*;

    match value_type {
        String => Some(0x1),
        Bool => Some(0x2),
        _ => None,
    }
}

/// Applies the integer transformation generally used in property data in the
/// Roblox binary format.
pub fn transform_i32(value: i32) -> i32 {
    (value << 1) ^ (value >> 31)
}

/// The inverse of `transform_i32`.
pub fn untransform_i32(value: i32) -> i32 {
    ((value as u32) >> 1) as i32 ^ -(value & 1)
}
