// define data structure in .proto file
// save, transmit data as compact binary format (serialize) : more efficient than json -> deserialize 

// WireType can be Varint, Len or I32
// tag = field number + wiretype (in binary format)

use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("Invalid varint")]
    InvalidVarint,
    #[error("Invalid wire-type")]
    InvalidWireType,
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Invalid length")]
    InvalidSize(#[from] std::num::TryFromIntError),
    #[error("Unexpected wire-type)")]
    UnexpectedWireType,
    #[error("Invalid string (not UTF-8)")]
    InvalidString,
}

/// A wire type as seen on the wire.
enum WireType { 
    // The Varint WireType indicates the value is a single VARINT.
    Varint,
    //I64,  -- not needed for this exercise
    // The Len WireType indicates that the value is a length represented as a
    // VARINT followed by exactly that number of bytes.
    Len,
    // The I32 WireType indicates that the value is precisely 4 bytes in
    // little-endian order containing a 32-bit signed integer.
    I32,
}

#[derive(Debug)]
/// A field's value, typed based on the wire type.
enum FieldValue<'a> {
    Varint(u64),
    //I64(i64),  -- not needed for this exercise
    Len(&'a [u8]),
    I32(i32),
}

#[derive(Debug)]
// A field, containing the field number and its value.
struct Field<'a> {
    field_num: u64,
    value: FieldValue<'a>,
}

trait ProtoMessage<'a>: Default + 'a {
    fn add_field(&mut self, field: Field<'a>) -> Result<(), Error>;
}

impl TryFrom<u64> for WireType {
    type Error = Error;

    fn try_from(value: u64) -> Result<WireType, Error> {
        Ok(match value {
            0 => WireType::Varint,
            //1 => WireType::I64,  -- not needed for this exercise
            2 => WireType::Len,
            5 => WireType::I32,
            _ => return Err(Error::InvalidWireType),
        })
    }
}

impl<'a> FieldValue<'a> {
    fn as_string(&self) -> Result<&'a str, Error> {
        let FieldValue::Len(data) = self else {
            return Err(Error::UnexpectedWireType);
        };

        std::str::from_utf8(data).map_err(|_| Error::InvalidString)
    }

    fn as_bytes(&self) -> Result<&'a [u8], Error> {
        let FieldValue::Len(data) = self else {
            return Err(Error::UnexpectedWireType);
        };

        Ok(data)
    }

    fn as_u64(&self) -> Result<u64, Error> {
        let FieldValue::Varint(value) = self else {
            return Err(Error::UnexpectedWireType);
        };

        Ok(*value)
    }
}

// Parse a VARINT, returning the parsed value and the remaining bytes.
fn parse_varint(data: &[u8]) -> Result<(u64, &[u8]), Error> {
    for i in 0..7 {
        let Some(b) = data.get(i) else {
            return Err(Error::InvalidVarint);
        };

        if b & 0x80 == 0 { // check continuation bit
            // This is the last byte of the VARINT, so convert it to
            // a u64 and return it.
            let mut value = 0u64;

            for b in data[..=i].iter().rev() {
                value = (value << 7) | (b & 0x7f) as u64;
            }

            return Ok((value, &data[i + 1..]));
        }
    }

    // More than 7 bytes is invalid.
    Err(Error::InvalidVarint)
}

// Convert a tag into a field number and a WireType.
fn unpack_tag(tag: u64) -> Result<(u64, WireType), Error> {
    let field_num = tag >> 3;
    let wire_type = WireType::try_from(tag & 0x7)?;

    Ok((field_num, wire_type))
}

// Parse a field, returning the remaining bytes
fn parse_field(data: &[u8]) -> Result<(Field, &[u8]), Error> {
    let (tag, remainder) = parse_varint(data)?; // tag itself is a Varint
    let (field_num, wire_type) = unpack_tag(tag)?;
    match wire_type {
        WireType::Varint => {
            let (value, remainder) = parse_varint(remainder)?;
            Ok((Field {field_num, value: FieldValue::Varint(value)}, remainder))
        },
        WireType::Len => { 
            let (length, remainder) = parse_varint(remainder)?;
            let length = usize::try_from(length)?;
            // Error Checking 
            if remainder.len() < length {
                return Err(Error::UnexpectedEOF);
            }

            Ok((Field {field_num, value: FieldValue::Len(&remainder[..length])}, &remainder[length..]))
        },
        WireType::I32 => {
            // Error Checking
            if remainder.len() < 4 {
                return Err(Error::UnexpectedEOF);
            }

            let value = i32::from_le_bytes(remainder[..4].try_into().unwrap());
            Ok((Field {field_num, value: FieldValue::I32(value)}, &remainder[4..]))
        }
    }
}

// Parse a message in the given data, calling `T::add_field` for each field in
// the message.
//
// The entire input is consumed.
fn parse_message<'a, T: ProtoMessage<'a>>(mut data: &'a [u8]) -> Result<T, Error> {
    let mut result = T::default();

    while !data.is_empty() {
        let parsed = parse_field(data)?;
        result.add_field(parsed.0)?;
        data = parsed.1;
    }

    Ok(result)
}

#[derive(Debug, Default)]
struct PhoneNumber<'a> {
    number: &'a str,
    type_: &'a str,
}

#[derive(Debug, Default)]
struct Person<'a> {
    name: &'a str,
    id: u64,
    phone: Vec<PhoneNumber<'a>>,
}

impl<'a> ProtoMessage<'a> for PhoneNumber<'a> {
    fn add_field(&mut self, field: Field<'a>) -> Result<(), Error> {
        match field.field_num {
            1 => self.number = field.value.as_string()?,
            2 => self.type_ = field.value.as_string()?,
            _ => (),
        }
        Ok(())
    }
}

impl<'a> ProtoMessage<'a> for Person<'a> {
    fn add_field(&mut self, field: Field<'a>) -> Result<(), Error> {
        match field.field_num {
            1 => self.name = field.value.as_string()?,
            2 => self.id = field.value.as_u64()?,
            3 => {
                let phone_data = field.value.as_bytes()?;
                let phonenumber: PhoneNumber = parse_message(phone_data).unwrap();
                self.phone.push(phonenumber);
            },
            _ => (),
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn person() {
        let person: Person = parse_message(&[
            0x0a, 0x07, 0x6d, 0x61, 0x78, 0x77, 0x65, 0x6c, 0x6c, 0x10, 0x2a, 0x1a, 0x16, 0x0a,
            0x0e, 0x2b, 0x31, 0x32, 0x30, 0x32, 0x2d, 0x35, 0x35, 0x35, 0x2d, 0x31, 0x32, 0x31,
            0x32, 0x12, 0x04, 0x68, 0x6f, 0x6d, 0x65, 0x1a, 0x18, 0x0a, 0x0e, 0x2b, 0x31, 0x38,
            0x30, 0x30, 0x2d, 0x38, 0x36, 0x37, 0x2d, 0x35, 0x33, 0x30, 0x38, 0x12, 0x06, 0x6d,
            0x6f, 0x62, 0x69, 0x6c, 0x65,
        ])
        .unwrap();
        assert_eq!(format!("{:#?}", person), "Person {\n    name: \"maxwell\",\n    id: 42,\n    phone: [\n        PhoneNumber {\n            number: \"+1202-555-1212\",\n            type_: \"home\",\n        },\n        PhoneNumber {\n            number: \"+1800-867-5308\",\n            type_: \"mobile\",\n        },\n    ],\n}");
    }

    #[test]
    fn as_string() {
        assert!(FieldValue::Varint(10).as_string().is_err());
        assert!(FieldValue::I32(10).as_string().is_err());
        assert_eq!(FieldValue::Len(b"hello").as_string().unwrap(), "hello");
    }

    #[test]
    fn as_bytes() {
        assert!(FieldValue::Varint(10).as_bytes().is_err());
        assert!(FieldValue::I32(10).as_bytes().is_err());
        assert_eq!(FieldValue::Len(b"hello").as_bytes().unwrap(), b"hello");
    }

    #[test]
    fn as_u64() {
        assert_eq!(FieldValue::Varint(10).as_u64().unwrap(), 10u64);
        assert!(FieldValue::I32(10).as_u64().is_err());
        assert!(FieldValue::Len(b"hello").as_u64().is_err());
    }
}
