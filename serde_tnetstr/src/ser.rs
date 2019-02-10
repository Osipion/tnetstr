use std::fmt::Display;
use std::str;
use serde::ser::{self, Serialize};
use tnetstr::TNetStrError;
use super::{Result, SerdeTNetError};
use std::fmt::Error;
use std::fmt::Formatter;

const TRUE: [u8; 7] = [b'4', b':', b't', b'r', b'u', b'e', b'!'];
const FALSE: [u8; 8] = [b'4', b':', b'f', b'a', b'l', b's', b'e', b'!'];
const NULL: [u8; 3] = [b'0', b':', b'~'];

pub struct Serializer {
    stack: Vec<Vec<u8>>
}

fn byte_string(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| if b.is_ascii() { b.clone() as char} else { '.' })
        .collect()
}

impl Display for Serializer {

    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), Error> {
        for i in (0..self.stack.len()).rev() {
            if self.stack[i].is_empty() {
                writeln!(f, "{} -> EMPTY", i)?;
            } else {
                writeln!(f, "{} -> [{}]", i, byte_string(&self.stack[i]))?;
            }
        }
        Ok(())
    }
}

pub fn to_string<T>(value: &T) -> Result<String>
    where
        T: Serialize,
{
    let s = byte_string(&to_bytes(value)?);
    Ok(s)
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
    where
        T: Serialize, {
    let mut serializer = Serializer {
        stack: vec![vec![]]
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.stack.pop().unwrap())
}

impl Serializer {

    fn append_bytes(&mut self, bytes: &[u8]) {
        let mut appendable: Vec<u8> = bytes.iter().map(|b|b.clone()).collect();
        let mut frame = self.stack.pop().unwrap();
        frame.append(&mut appendable);
        self.stack.push(frame);
    }

    fn end_seq(&mut self, type_char: u8) -> Result<()> {
        match self.stack.pop() {
            None => Err(SerdeTNetError(TNetStrError::SerializationError("Attempt to end sequence without start".to_string()))),
            Some(seq) => {
                let start = format!("{}:", seq.len());
                self.append_bytes(start.as_bytes());
                self.append_bytes(&seq);
                self.append_bytes(&vec![type_char]);
                Ok(())
            }
        }
    }

    fn end_list(&mut self) -> Result<()> {
        self.end_seq(b']')
    }

    fn end_map(&mut self) -> Result<()> {
        self.end_seq(b'}')
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {

    type Ok = ();

    type Error = SerdeTNetError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.append_bytes(if v { &TRUE } else { &FALSE });
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let s = format!("{}", v);
        let f = format!("{}:{}#", s.len(), s);
        self.append_bytes(&f.as_bytes());
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let s = format!("{}", v);
        let f = format!("{}:{}#", s.len(), s);
        self.append_bytes(&f.as_bytes());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let s = format!("{:.32}", v);
        let f = format!("{}:{}^", s.len(), s);
        self.append_bytes(&f.as_bytes());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_bytes(&v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let len_str = v.len().to_string();
        let len_bytes = len_str.as_bytes();
        let mut f = vec![0 as u8; len_bytes.len() + 1 + v.len() + 1];
        let mut index = 0;
        while index < len_bytes.len() {
            f[index] = len_bytes[index].clone();
            index += 1;
        }
        f[index] = b':';
        index += 1;
        while index - (len_bytes.len() + 1) < v.len() {
            let rindex = index - (len_bytes.len() + 1);
            f[index] = v[rindex].clone();
            index += 1
        }
        f[index] = b',';
        self.append_bytes(&f);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.append_bytes(&NULL);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.append_bytes(&NULL);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        self.serialize_map(None)?;
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        self.end_map()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.stack.push(vec![]);
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    /// Tuple structs look just like sequences in TNet.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Tuple variants are represented in TNetStrings as a dictionary with a single key and a list of
    /// values.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_map(None)?;
        variant.serialize(&mut *self)?;
        self.serialize_seq(None)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.stack.push(vec![]);
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(_len))?;
        variant.serialize(&mut *self)?;
        self.serialize_map(None)?;
        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<()> where
        T: Display {
        unimplemented!()
    }
}


impl<'a> ser::SerializeSeq for &'a mut Serializer {

    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {

        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.end_list()
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_list()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_list()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_list()?;
        self.end_map()
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_map()
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_map()
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerdeTNetError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.end_map()?;
        self.end_map()
    }
}

////////////////////////////////////////////////////////////////////////////////
#[test]
fn test_seq() {
    let seq = vec![1,2,3,4];
    let expected = "16:1:1#1:2#1:3#1:4#]";
    assert_eq!(to_string(&seq).unwrap(), expected);
}

#[test]
fn test_map() {
    use std::collections::HashMap;
    let mut map = HashMap::<String, i32>::new();
    map.insert("a".to_owned(), 1);
    map.insert("b".to_owned(), 2);
    let expected = "16:1:b,1:2#1:a,1:1#}";
    assert_eq!(to_string(&map).unwrap(), expected);
}

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let expected = r#"27:3:int,1:1#3:seq,8:1:a,1:b,]}"#;
    assert_eq!(to_string(&test).unwrap(), expected);
}

#[test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    let expected = r#"4:Unit,"#;
    assert_eq!(to_string(&u).unwrap(), expected);

    let n = E::Newtype(1);
    let expected = r#"14:7:Newtype,1:1#}"#;
    assert_eq!(to_string(&n).unwrap(), expected);

    let t = E::Tuple(1, 2);
    let expected = r#"19:5:Tuple,8:1:1#1:2#]}"#;
    assert_eq!(to_string(&t).unwrap(), expected);

    let s = E::Struct { a: 1 };
    let expected = r#"20:6:Struct,8:1:a,1:1#}}"#;
    assert_eq!(to_string(&s).unwrap(), expected);
}