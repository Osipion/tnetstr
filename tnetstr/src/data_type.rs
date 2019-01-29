use std::fmt::{
    Display,
    Formatter,
    Error
};
pub use super::errors::TNetStrError;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum DataType {
    Bytes,
    Integer,
    Float,
    Boolean,
    Null,
    Dictionary,
    List
}


impl DataType {

    pub fn from_byte(c: &u8) -> Result<DataType, TNetStrError> {
        match c {
            b',' => Ok(DataType::Bytes),
            b'#' => Ok(DataType::Integer),
            b'^' => Ok(DataType::Float),
            b'!' => Ok(DataType::Boolean),
            b'~' => Ok(DataType::Null),
            b'}' => Ok(DataType::Dictionary),
            b']' => Ok(DataType::List),
            _ => Err(TNetStrError::UnrecognizedDataType(c.clone()))
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            DataType::Bytes => b',',
            DataType::Integer => b'#',
            DataType::Float => b'^',
            DataType::Boolean => b'!',
            DataType::Null => b'~',
            DataType::Dictionary => b'}',
            DataType::List => b']',
        }
    }

    pub fn to_char(&self) -> char {
        self.to_byte() as char
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_char())
    }
}