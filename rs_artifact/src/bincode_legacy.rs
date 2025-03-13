use crate::error::Result;
use crate::EEndianType;
use serde::de::DeserializeOwned;

pub fn serialize<T>(val: &T, endian_type: Option<EEndianType>) -> Result<Vec<u8>>
where
    T: serde::ser::Serialize,
{
    let endian_type = endian_type.unwrap_or_default();
    match endian_type {
        EEndianType::Big => {
            bincode::serde::encode_to_vec(val, bincode::config::standard().with_big_endian())
        }
        EEndianType::Little => {
            bincode::serde::encode_to_vec(val, bincode::config::standard().with_little_endian())
        }
        EEndianType::Native => bincode::serde::encode_to_vec(val, bincode::config::standard()),
    }
    .map_err(|err| {
        let msg = format!("Fail to serialize.");
        crate::error::Error::EncodeError(err, Some(msg))
    })
}

pub fn deserialize<D: DeserializeOwned>(src: &[u8], endian_type: Option<EEndianType>) -> Result<D> {
    let endian_type = endian_type.unwrap_or_default();
    let result: std::result::Result<(D, usize), bincode::error::DecodeError> = match endian_type {
        EEndianType::Big => {
            bincode::serde::decode_from_slice(src, bincode::config::standard().with_big_endian())
        }
        EEndianType::Little => {
            bincode::serde::decode_from_slice(src, bincode::config::standard().with_little_endian())
        }
        EEndianType::Native => bincode::serde::decode_from_slice(src, bincode::config::standard()),
    };
    match result {
        Ok((object, _)) => Ok(object),
        Err(err) => {
            let msg = format!("Fail to deserialize.");
            Err(crate::error::Error::Bincode(err, Some(msg)))
        }
    }
}

pub fn deserialize_from<'r, R: std::io::Read, D: DeserializeOwned>(
    src: &'r mut R,
    endian_type: Option<EEndianType>,
) -> Result<D> {
    let mut data = vec![];
    src.read_to_end(&mut data)
        .map_err(|err| crate::error::Error::IO(err, None))?;
    deserialize(&data, endian_type)
    // let endian_type = endian_type.unwrap_or_default();
    // let result: std::result::Result<(D, usize), bincode::error::DecodeError> = match endian_type {
    //     EEndianType::Big => {
    //         bincode::serde::decode_from_std_read(src, bincode::config::standard().with_big_endian())
    //     }
    //     EEndianType::Little => {
    //         //
    //         bincode::serde::decode_from_std_read(
    //             src,
    //             bincode::config::standard().with_little_endian(),
    //         )
    //     }
    //     EEndianType::Native => {
    //         bincode::serde::decode_from_std_read(src, bincode::config::standard())
    //     }
    // };
    // match result {
    //     Ok((object, _)) => Ok(object),
    //     Err(err) => {
    //         let msg = format!("Fail to deserialize.");
    //         Err(crate::error::Error::Bincode(err, Some(msg)))
    //     }
    // }
}
