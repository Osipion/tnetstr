use std::fmt::{
    Display,
    Formatter,
    Error
};
use super::super::{
    TNetData,
    DataType
};

/// Represents a tnetstring
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct TNetEntry {
    /// The data content of the tnetstring
    pub data: TNetData,
    /// The number of bytes in the tnetstring data
    pub size: u32,
    /// The type of the tnetstring data
    pub data_type: DataType
}

impl Display for TNetEntry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}:{}{}", self.size, self.data, self.data_type)
    }
}