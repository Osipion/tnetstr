use std::{
    error::Error,
    fmt::{Formatter, Display}
};

#[derive(Debug, Eq, PartialEq)]
pub enum TNetStrError {
    UnrecognizedDataType(u8),
    LengthTooLong,
    NoLengthSpecified,
    LengthWasIncorrect(u64, u64),
    NonASCIINumericValueInLengthField(u8),
    LengthTerminatorNotFound,
    LengthWasNotValidUTF8,
    LengthWasNotAccurate,
    CouldNotParseLength,
    CouldNotParseData,
    DataNotUTF8Compatible,
    UnbalancedDictionary,
    DictionaryKeyWasNotBytes,
    FloatParseError(String),
    StreamReadFailed(String),
    SerializationError(String)
}

impl Display for TNetStrError {

    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            TNetStrError::UnrecognizedDataType(c) => write!(f, "No data type match char '{:?}'", *c as char),
            TNetStrError::LengthTooLong => write!(f, "The length specified was too long"),
            TNetStrError::NoLengthSpecified => write!(f, "The length of he tnet string was not specified"),
            TNetStrError::LengthWasIncorrect(expected, actual) =>  write!(f, "The stated length was incorrect: expect {}, actual {}", expected, actual),
            TNetStrError::NonASCIINumericValueInLengthField(c) => write!(f, "Non-Numeric ASCII byte found in length field: '{}'", *c as char),
            TNetStrError::LengthTerminatorNotFound => write!(f, "The length was not terminated before the string ended"),
            TNetStrError::LengthWasNotValidUTF8 => write!(f, "The length bytes could not be converted to UTF8. This indicates a bug in this parser."),
            TNetStrError::CouldNotParseLength => write!(f, "UTF8 of length bytes could not be parsed to u32. Indicates a bug in this parser"),
            TNetStrError::CouldNotParseData => write!(f, "Failed to parse data"),
            TNetStrError::DataNotUTF8Compatible => write!(f, "Input of non-raw byte data was not utf-8 compatible."),
            TNetStrError::UnbalancedDictionary => write!(f, "A dictionary value contained an uneven number of entries."),
            TNetStrError::DictionaryKeyWasNotBytes => write!(f, "A dictionary key was found that was not of the 'Bytes' type."),
            TNetStrError::FloatParseError(s) => write!(f, "Failed to parse float '{}.", s),
            TNetStrError::StreamReadFailed(s) => write!(f, "Failed to read stream '{}.", s),
            TNetStrError::LengthWasNotAccurate => write!(f, "The length specified was beyond the end of the data"),
            TNetStrError::SerializationError(s) => write!(f, "{}", s)
        }
    }
}

impl std::error::Error for TNetStrError {

}