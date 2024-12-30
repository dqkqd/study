use std::io::Read;

use serde::{de, ser};

use crate::Result;

pub(crate) trait ByteParser
where
    Self: ser::Serialize + de::DeserializeOwned,
{
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let v = bson::to_vec(&self)?;
        Ok(v)
    }

    fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: Read,
    {
        let value: Self = bson::from_reader(reader)?;
        Ok(value)
    }
}
