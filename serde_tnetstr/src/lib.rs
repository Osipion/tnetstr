extern crate serde;
extern crate tnetstr;
// The serde_derive crate provides the macros for #[derive(Serialize)] and
// #[derive(Deserialize)]. You won't need these for implementing a data format
// but your unit tests will probably use them - hence #[cfg(test)].
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod de;
mod error;
mod ser;

//pub use de::{from_str, Deserializer};
pub use error::{Result, SerdeTNetError};
pub use ser::{to_string, Serializer};
