use std::{
    fmt::{
        Display,
        Error,
        Formatter
    }
};
use super::super::TNetStrError;

// A hashable, equatable float
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct TNetFloat {
    pub integral: u64,
    pub fractional: u64,
}

impl TNetFloat {

    /// Parses a TNetFloat from a string representing it's decimal value (e.g. 10.21)
    pub fn from_decimal_str(s: &str) -> Result<TNetFloat, TNetStrError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            Err(TNetStrError::FloatParseError(s.to_string()))
        } else {
            match parts[0].parse::<u64>() {
                Err(_) => Err(TNetStrError::FloatParseError(s.to_string())),
                Ok(integral) => match parts[1].parse::<u64>() {
                    Err(_) => Err(TNetStrError::FloatParseError(s.to_string())),
                    Ok(fractional) => Ok(TNetFloat { integral, fractional })
                }
            }
        }
    }

    /// Returns the TNetFloat's value as an f64
    pub fn to_f64(&self) -> f64 {
        format!("{}.{}", self.integral, self.fractional).parse().expect("Bug - parse to f64 failed")
    }
}

impl Display for TNetFloat {

    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:.32}", self.to_f64())
    }
}