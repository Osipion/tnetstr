use std::{
    fmt::{
        Display,
        Formatter,
        Error
    }
};
use super::{
    TNetFloat,
    TNetDictionary,
    TNetList
};

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum TNetData {
    Bytes(Vec<u8>),
    Integer(i64),
    Float(TNetFloat),
    Boolean(bool),
    Null,
    Dictionary(TNetDictionary),
    List(TNetList)
}

impl Display for TNetData {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            TNetData::Null => write!(f, ""),
            TNetData::Bytes(b) => {
                let c = b.iter().map(|c| *c as char).collect::<String>();
                write!(f, "{}", c)
            },
            TNetData::Integer(n) => write!(f, "{}", n),
            TNetData::Float(n) => write!(f, "{}", n),
            TNetData::Boolean(b) => write!(f, "{}", b),
            TNetData::Dictionary(dict) => {
                let s = dict.entries.iter()
                    .map(|e| format!("{}{}", e.0, e.1))
                    .fold("".to_string(), |a, v| a + &format!("{}", v));
                write!(f, "{}", s)
            },
            TNetData::List(l) => {
                let s = l.iter()
                    .fold("".to_string(), |a, v| a + &format!("{}", v));
                write!(f, "{}", s)
            }
        }
    }
}

