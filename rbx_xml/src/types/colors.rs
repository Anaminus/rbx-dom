use std::io::Read;

use xml::{
    reader::{XmlEvent as XmlReadEvent},
};

use rbx_tree::RbxValue;

use crate::{
    deserializer::{DecodeError, EventIterator},
};

pub fn deserialize_color3<R: Read>(reader: &mut EventIterator<R>) -> Result<RbxValue, DecodeError> {
    // Color3s have two possibilities:
    // They are either a packed int (like Color3uint8) or they are a triple of
    // <R>, <G>, and <B> tags with floating-point values inside them.
    // First we have to find out if we have a packed int in.
    if let Ok(XmlReadEvent::Characters(content)) = reader.peek().ok_or(DecodeError::MalformedDocument)? {
        let [r, g, b] = decode_packed_color3(content)?;
        // advance the reader; we peeked in the if statement!
        reader.next();
        Ok(RbxValue::Color3 {
            // floating-point Color3s go from 0 to 1 instead of 0 to 255
            value: [ f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0 ],
        })
    } else {
        let r: f32 = reader.read_tag_contents("R")?.parse()?;
        let g: f32 = reader.read_tag_contents("G")?.parse()?;
        let b: f32 = reader.read_tag_contents("B")?.parse()?;
        Ok(RbxValue::Color3 {
            value: [ r, g, b ],
        })
    }
}

pub fn deserialize_color3uint8<R: Read>(reader: &mut EventIterator<R>) -> Result<RbxValue, DecodeError> {
    // Color3uint8s are stored as packed u32s.
    read_event!(reader, XmlReadEvent::Characters(content) => {
        Ok(RbxValue::Color3uint8 {
            value: decode_packed_color3(&content)?,
        })
    })
}

fn decode_packed_color3(source: &str) -> Result<[u8; 3], DecodeError> {
    let packed_color: u32 = source.parse()?;
    let r = (packed_color >> 16) & 0xFF;
    let g = (packed_color >> 8) & 0xFF;
    let b = packed_color & 0xFF;
    Ok([ r as u8, g as u8, b as u8 ])
}