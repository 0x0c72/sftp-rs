use nom::Err::Failure;
use nom::error::Error as NomError;
use nom::error::ErrorKind as NomErrorKind;
use nom::IResult;
use nom::number::complete::be_u32;
use nom::take;
use nom::take_str;

use serde::ser::SerializeTuple;
use serde::Serialize;
use serde::Serializer;

use uuid::Uuid;

pub fn vec_u8_as_slice<S: Serializer>(elements: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
	let mut tuple = serializer.serialize_tuple(elements.len())?;
	for e in elements {
		tuple.serialize_element(e)?;
	}
	tuple.end()
}

pub fn vec_with_u32_length<S: Serializer, T: Serialize>(elements: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error> {
	let mut seq = serializer.serialize_tuple(elements.len() + 1)?;
	seq.serialize_element(&(elements.len() as u32))?;
	for e in elements {
		seq.serialize_element(e)?;
	}
	seq.end()
}

pub fn str_with_u32_length<S: Serializer>(s: &str, serializer: S) -> Result<S::Ok, S::Error> {
	let slice = s.as_bytes();
	let mut seq = serializer.serialize_tuple(slice.len() + 1)?;
	seq.serialize_element(&(slice.len() as u32))?;
	for c in slice {
		seq.serialize_element(c)?;
	}
	seq.end()
}

pub fn serialize_uuid<S: Serializer>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
	str_with_u32_length(&uuid.to_string(), serializer)
}

pub fn parse_u8_vec(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
	let (i, len) = be_u32(i)?;
	if(i.len() < len as usize) {
		return Ok((i, Vec::new()));
	}
	let (i, slice) = take!(i, len)?;
	Ok((i, Vec::from(slice)))
}

// TODO:  Generic over I instead of hardcoded &[u8]
pub fn parse_string(i: &[u8]) -> IResult<&[u8], String> {
	let (i, len) = be_u32(i)?;
	let (i, string) = take_str!(i, len)?;
	Ok((i, string.to_string()))
}

pub fn parse_uuid(i: &[u8]) -> IResult<&[u8], Uuid> {
	let (i, s) = parse_string(i)?;
	match Uuid::parse_str(&s) {
		Ok(v) => Ok((i, v)),
		Err(_) => Err(Failure(NomError::new(i, NomErrorKind::ParseTo)))
	}
}

