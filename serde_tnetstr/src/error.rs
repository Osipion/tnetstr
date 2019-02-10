use tnetstr::TNetStrError;
use serde;
use std::fmt::{Formatter, Error as FmtError};

#[derive(Debug)]
pub struct SerdeTNetError(pub TNetStrError);

impl std::fmt::Display for SerdeTNetError {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), FmtError> {
        self.0.fmt(f)
    }
}

impl std::error::Error for SerdeTNetError {

}

impl serde::ser::Error for SerdeTNetError {

    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SerdeTNetError(TNetStrError::SerializationError(format!("{}", msg)))
    }
}

pub type Result<T> = std::result::Result<T, SerdeTNetError>;