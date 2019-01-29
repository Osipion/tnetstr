pub mod tnetfloat;
pub mod tnetdata;
pub mod tnetdict;
pub mod tnetentry;

pub use self::tnetfloat::TNetFloat;
pub use self::tnetdict::TNetDictionary;
pub use self::tnetdata::TNetData;
pub use self::tnetentry::TNetEntry;
pub type TNetList = Vec<TNetEntry>;