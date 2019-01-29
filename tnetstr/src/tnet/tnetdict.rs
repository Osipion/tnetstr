use super::{
    TNetEntry,
    TNetData
};
use super::super::{
    DataType,
    TNetStrError
};

/// A tnetdictionary, containing pairs of (Key: Bytes, Value AnyData)
/// It is unclear from the spec whether duplicate keys are permitted, so
/// this is implemented so as to support them.
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct TNetDictionary {
    pub entries: Vec<(TNetEntry, TNetEntry)>
}

impl TNetDictionary {

    pub fn new() -> TNetDictionary {
        TNetDictionary {
            entries: vec![]
        }
    }

    pub fn from_vec(mut input: Vec<TNetEntry>) -> Result<TNetDictionary, TNetStrError> {
        let mut d = TNetDictionary::new();

        while input.len() > 0 {
            let mut pairs: Vec<_> = input.drain(0..2).collect();
            let value = pairs.pop().unwrap();
            let key = match pairs.pop() {
                None => return Err(TNetStrError::UnbalancedDictionary),
                Some(t) => t
            };
            d.add(key, value)?;
        }
        Ok(d)
    }

    pub fn add(&mut self, key: TNetEntry, value: TNetEntry) -> Result<(), TNetStrError> {
        match key.data_type {
            DataType::Bytes => Ok(self.entries.push((key, value))),
            _ => Err(TNetStrError::DictionaryKeyWasNotBytes)
        }
    }

    pub fn get(&self, key: &[u8]) -> Vec<&TNetEntry> {
        let mut r : Vec<&TNetEntry> = vec![];
        for entry in &self.entries {
            match &entry.0.data {
                TNetData::Bytes(b) => match b.as_slice() == key {
                    true => r.push(&entry.1),
                    false => {}
                } ,
                _ => panic!("TNetDictionary had non-byte key")
            }
        };
        r
    }
}