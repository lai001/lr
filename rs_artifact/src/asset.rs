use crate::error::Result;
use crate::{
    EEndianType,
    file_header::{ASSET_FILE_MAGIC_NUMBERS, FileHeader, HEADER_LENGTH_SIZE},
};
use rs_artifact_types::asset::Asset;
use rs_artifact_types::resource_type::EResourceType;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssetHeader {
    pub resource_type: EResourceType,
}

/// The generic parameter `T` must be a **trait object type** (e.g.
/// `dyn Asset`, `dyn Asset2`, `dyn Asset3`, ...), not a concrete type.
/// Every supported trait must be a subtrait of [`Asset`] and registered
/// with `#[typetag::serde]` so that `Box<T>: DeserializeOwned` is
/// satisfied. This relies on `erased_serde` + `typetag` under the hood,
/// which means the asset payload in the artifact must have been encoded
/// with the typetag tag/content wrapping.
///
/// # Examples
///
/// ```ignore
/// let data: Result<Vec<u8>> = encode_asset::<dyn Asset>(&asset, None)?;
/// let data: Result<Vec<u8>> = encode_asset::<dyn Asset2>(&asset, None)?;
/// ```
///
/// # Note
///
/// Passing a concrete type (e.g. `encode_asset::<CustomAsset>`) will compile
/// because `CustomAsset` satisfies both `Asset` and `Serialize`, but
/// it will fail at runtime: the payload is wrapped with typetag's tag, so a
/// concrete type's plain serde deserialization cannot parse it. Always pass
/// a trait object type as `T`.
pub(crate) fn encode_asset<T>(asset: &T, endian_type: Option<EEndianType>) -> Result<Vec<u8>>
where
    T: Asset + Serialize + ?Sized,
{
    let resource_type = asset.resource_type();
    let asset_header = AssetHeader { resource_type };
    let header_data =
        FileHeader::write_header(ASSET_FILE_MAGIC_NUMBERS, &asset_header, endian_type)?;
    // let endian_type = endian_type.unwrap_or(EEndianType::Native);
    let payload = crate::bincode_legacy::serialize(asset, endian_type)?;
    let mut data = vec![0; header_data.len() + payload.len()];
    data[0..header_data.len()].copy_from_slice(&header_data);
    data[header_data.len()..].copy_from_slice(&payload);
    Ok(data)
}

pub(crate) fn decode_asset<T>(
    data: &[u8],
    endian_type: Option<EEndianType>,
    expected_resource_type: Option<EResourceType>,
) -> Result<T>
where
    T: Asset + DeserializeOwned + ?Sized,
{
    let mut reader = std::io::Cursor::new(data);
    let _ = FileHeader::check_identification(&mut reader, ASSET_FILE_MAGIC_NUMBERS)?;
    let length = FileHeader::get_header_encoded_data_length(&mut reader, endian_type)?;
    let asset_header: AssetHeader = FileHeader::get_header2(&mut reader, endian_type)?;
    if let Some(expected_resource_type) = expected_resource_type {
        if asset_header.resource_type != expected_resource_type {
            return Err(crate::error::Error::ResourceTypeNotMatch(Some(format!(
                "{:?} != {:?}",
                asset_header.resource_type, expected_resource_type
            ))));
        }
    }
    let offset = length + ASSET_FILE_MAGIC_NUMBERS.len() as u64 + HEADER_LENGTH_SIZE as u64;
    let _ = reader
        .seek(std::io::SeekFrom::Start(offset))
        .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to seek {}", offset))));
    let mut payload: Vec<u8> = vec![];
    let _ = reader
        .read_to_end(&mut payload)
        .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to read all bytes."))))?;
    // let endian_type = endian_type.unwrap_or(EEndianType::Native);
    let asset = crate::bincode_legacy::deserialize::<T>(&payload, endian_type)?;
    Ok(asset)
}

#[cfg(test)]
mod test {
    use crate::{
        asset::{Asset, decode_asset, encode_asset},
        bincode_legacy,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct AssetWrapper<T: Sized> {
        #[serde(rename = "type")]
        ty: String,
        asset: T,
    }

    #[typetag::serde(tag = "type", content = "asset2")]
    trait Asset2: Asset {}

    impl Asset for Box<dyn Asset2> {
        #[doc(hidden)]
        fn typetag_name(&self) -> &'static str {
            Asset2::typetag_name(self.as_ref())
        }

        #[doc(hidden)]
        fn typetag_deserialize(&self) {
            Asset2::typetag_deserialize(self.as_ref())
        }

        fn get_url(&self) -> url::Url {
            self.as_ref().get_url()
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CustomAsset {
        pub f32_value: f32,
    }

    #[typetag::serde]
    impl Asset for CustomAsset {
        fn get_url(&self) -> url::Url {
            unimplemented!()
        }
    }

    mod a1 {
        use crate::asset::{Asset, test::Asset2};
        use rs_artifact_types::resource_type::EResourceType;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct CustomAsset {
            pub f32_value: f32,
        }

        #[typetag::serde(name = "a1::CustomAsset")]
        impl Asset2 for CustomAsset {}

        #[typetag::serde(name = "a1::CustomAsset")]
        impl Asset for CustomAsset {
            fn get_url(&self) -> url::Url {
                unimplemented!()
            }
        }

        #[derive(Deserialize, Serialize)]
        pub struct AssetWrapper<T: Sized + serde::Serialize> {
            #[serde(rename = "type")]
            pub ty: String,
            #[serde(rename = "asset2")]
            pub asset: T,
        }

        impl<T: Sized + serde::Serialize + 'static + Asset> Asset for AssetWrapper<T> {
            fn get_url(&self) -> url::Url {
                self.asset.get_url()
            }

            fn resource_type(&self) -> EResourceType {
                self.asset.resource_type()
            }

            #[doc(hidden)]
            fn typetag_name(&self) -> &'static str {
                self.asset.typetag_name()
            }

            #[doc(hidden)]
            fn typetag_deserialize(&self) {
                self.asset.typetag_deserialize();
            }
        }
    }

    #[test]
    fn test() {
        let asset = CustomAsset { f32_value: 1.0 };

        let asset_wrapper = AssetWrapper::<CustomAsset> {
            ty: asset.typetag_name().to_string(),
            asset: asset.clone(),
        };
        let contents1 = serde_json::to_string_pretty(&asset_wrapper).unwrap();

        let asset = Box::new(asset);
        let asset_wrapper = AssetWrapper::<Box<CustomAsset>> {
            ty: asset.typetag_name().to_string(),
            asset: asset.clone(),
        };
        let contents2 = serde_json::to_string_pretty(&asset_wrapper).unwrap();

        let asset = asset as Box<dyn Asset>;
        let contents3 = serde_json::to_string_pretty(&asset).unwrap();

        assert_eq!(contents1, contents2);
        assert_eq!(contents2, contents3);
    }

    #[test]
    fn test1() {
        let asset = CustomAsset { f32_value: 1.0 };

        let asset_wrapper = AssetWrapper::<CustomAsset> {
            ty: asset.typetag_name().to_string(),
            asset: asset.clone(),
        };

        let contents1 = bincode_legacy::serialize(&asset_wrapper, None).unwrap();

        let asset = Box::new(asset);
        let asset_wrapper = AssetWrapper::<Box<CustomAsset>> {
            ty: asset.typetag_name().to_string(),
            asset: asset.clone(),
        };
        let contents2 = bincode_legacy::serialize(&asset_wrapper, None).unwrap();

        let asset = asset as Box<dyn Asset>;
        let contents3 = bincode_legacy::serialize(&asset, None).unwrap();

        assert_eq!(contents1, contents2);
        assert_eq!(contents2, contents3);
    }

    #[test]
    fn test2() {
        let asset = a1::CustomAsset { f32_value: 1.0 };

        let asset_wrapper = a1::AssetWrapper::<a1::CustomAsset> {
            ty: Asset2::typetag_name(&asset).to_string(),
            asset: asset.clone(),
        };
        let contents1 = serde_json::to_string_pretty(&asset_wrapper).unwrap();

        let asset = Box::new(asset);
        let asset_wrapper = a1::AssetWrapper::<Box<a1::CustomAsset>> {
            ty: Asset2::typetag_name(asset.as_ref()).to_string(),
            asset: asset.clone(),
        };
        let contents2 = serde_json::to_string_pretty(&asset_wrapper).unwrap();

        let asset = asset as Box<dyn Asset2>;
        let contents3 = serde_json::to_string_pretty(&asset).unwrap();

        assert_eq!(contents1, contents2);
        assert_eq!(contents2, contents3);

        assert!(serde_json::from_str::<Box<dyn Asset2>>(&contents3).is_ok());
        assert!(serde_json::from_str::<Box<a1::CustomAsset>>(&contents3).is_err());
    }

    #[test]
    fn test3() {
        let asset = a1::CustomAsset { f32_value: 1.0 };
        let asset_wrapper = a1::AssetWrapper::<a1::CustomAsset> {
            ty: Asset2::typetag_name(&asset).to_string(),
            asset: asset.clone(),
        };
        let contents1 = encode_asset(&asset_wrapper, None).unwrap();
        let asset = Box::new(asset);
        let dyn_asset = asset.clone() as Box<dyn Asset2>;
        let contents2 = encode_asset::<dyn Asset2>(dyn_asset.as_ref(), None).unwrap();

        assert_eq!(contents1, contents2);

        let decoded_asset = decode_asset::<Box<dyn Asset2>>(&contents2, None, None).unwrap();
        let decoded_asset = decoded_asset
            .as_ref()
            .as_any()
            .downcast_ref::<a1::CustomAsset>()
            .unwrap();
        assert_eq!(decoded_asset.f32_value, asset.f32_value);
    }
}
