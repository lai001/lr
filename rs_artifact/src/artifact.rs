use crate::error::Result;
use crate::{
    EEndianType,
    asset::{self},
    file_header::{
        self, ARTIFACT_FILE_MAGIC_NUMBERS, FileHeader, HEADER_LENGTH_SIZE, IDENTIFICATION_SIZE,
    },
    resource_info::ResourceInfo,
};
use rs_artifact_types::asset::{ASSET_KIND, Asset, ResourceEncodeTask};
use rs_artifact_types::resource_type::EResourceType;
use rs_content::{CONTENT_ASSET_KIND, Content};
use rs_core_minimal::settings::Settings;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{
    collections::HashMap,
    io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ArtifactFileHeader {
    pub settings: Settings,
    pub resource_map: std::collections::HashMap<url::Url, ResourceInfo>,
}

pub fn encode_artifact_tasks_disk<R>(
    endian_type: Option<EEndianType>,
    settings: Settings,
    tasks: &mut [ResourceEncodeTask<R>],
    target_path: &Path,
) -> Result<()>
where
    R: Seek + Read,
{
    let parent = target_path
        .parent()
        .ok_or(crate::error::Error::NotFound(Some(format!(
            "No parent folder of {:?}",
            target_path
        ))))?;
    let _ = std::fs::create_dir_all(parent).map_err(|err| {
        crate::error::Error::IO(err, Some(format!("Can not create folder {:?}", parent)))
    })?;
    let file = std::fs::File::create(target_path).map_err(|err| {
        crate::error::Error::IO(err, Some(format!("Can not create file {:?}", target_path)))
    })?;
    let mut buf_writer = BufWriter::new(file);
    let mut infos: Vec<ResourceInfo> = vec![];
    let mut offset: u64 = 0;
    for task in tasks.iter_mut() {
        let length = task
            .reader
            .seek(SeekFrom::End(0))
            .map_err(|err| crate::error::Error::IO(err, Some(format!("Seek fail"))))?;
        let _ = task
            .reader
            .seek(SeekFrom::Start(0))
            .map_err(|err| crate::error::Error::IO(err, Some(format!("Seek fail"))))?;
        let info = ResourceInfo {
            url: task.url.clone(),
            resource_type: task.resource_type.clone(),
            offset,
            length,
        };
        offset += length;
        infos.push(info);
    }
    let mut fileheader = ArtifactFileHeader {
        resource_map: HashMap::new(),
        settings,
    };
    for info in infos {
        fileheader
            .resource_map
            .insert(info.url.clone(), info.clone());
    }
    let header_encoded_data =
        FileHeader::write_header(ARTIFACT_FILE_MAGIC_NUMBERS, &fileheader, endian_type)?;
    buf_writer.write(&header_encoded_data).map_err(|err| {
        crate::error::Error::IO(err, Some(format!("Failed to write header data.")))
    })?;
    for task in tasks.iter_mut() {
        let bytes = std::io::copy(&mut task.reader, &mut buf_writer)
            .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to copy data."))))?;
        log::trace!(
            "Url: {}, {:?}, bytes: {}",
            task.url.to_string(),
            task.resource_type,
            bytes
        );
    }
    Ok(())
}

pub fn encode_artifact_assets_disk<T>(
    settings: Settings,
    assets: &[T],
    endian_type: Option<EEndianType>,
    target_path: &Path,
) -> Result<()>
where
    T: Asset + Serialize,
{
    let mut tasks: Vec<ResourceEncodeTask<Cursor<Vec<u8>>>> = Vec::new();
    for asset in assets {
        let asset_encoded_data = asset::encode_asset(asset, endian_type)?;
        let reader = Cursor::new(asset_encoded_data);
        let task = asset.build_resource_encode_task(reader);
        tasks.push(task);
    }
    encode_artifact_tasks_disk(endian_type, settings, &mut tasks, target_path)
}

pub struct ArtifactAssetEncoder {
    settings: Settings,
    tasks: Vec<ResourceEncodeTask<Cursor<Vec<u8>>>>,
    endian_type: Option<EEndianType>,
    target_path: PathBuf,
}

impl ArtifactAssetEncoder {
    pub fn new(endian_type: Option<EEndianType>, settings: Settings, target_path: &Path) -> Self {
        Self {
            settings,
            tasks: vec![],
            endian_type,
            target_path: target_path.to_path_buf(),
        }
    }

    /// Encode a pure asset type (not a content type) into this artifact as a task.
    ///
    /// The `asset` must be a `&dyn Asset` whose concrete type implements the
    /// [`Asset`] trait only, and must be registered with `#[typetag::serde]` on
    /// `Asset` (`tag = "type", content = "asset"`). The payload is serialized
    /// via `erased_serde` + `typetag`, which writes the typetag tag/content
    /// wrapping (`{"type": ..., "asset": ...}`).
    ///
    /// # Warning
    ///
    /// Do **NOT** pass a content type here. Because of the `erased_serde` +
    /// `typetag` combination, a type that implements [`Content`] (which is a
    /// subtrait of `Asset`) is registered under `Content`'s typetag config
    /// (`content = "content"`), not `Asset`'s (`content = "asset"`). Feeding a
    /// `&dyn Content` into this method serializes it with `Content`'s tag
    /// name, which the artifact reader cannot match back to an `Asset` when
    /// decoding. Use [`Self::encode_content`] for content types instead.
    pub fn encode_asset(&mut self, asset: &dyn Asset) {
        let asset_encoded_data = asset::encode_asset(asset, self.endian_type).unwrap();
        let reader = Cursor::new(asset_encoded_data);
        let task = asset.build_resource_encode_task(reader);
        self.tasks.push(task);
    }

    /// Encode a content type into this artifact as a task.
    ///
    /// The `asset` must be a `&dyn Content` registered with `#[typetag::serde]`
    /// on `Content` (`tag = "type", content = "content"`). The payload is
    /// serialized via `erased_serde` + `typetag`, which writes the typetag
    /// tag/content wrapping (`{"type": ..., "content": ...}`).
    ///
    /// # Why not `encode_asset`?
    ///
    /// [`Content`] is a subtrait of `Asset`, so a `&dyn Content` *could* be
    /// passed to [`Self::encode_asset`]. However, because of the
    /// `erased_serde` + `typetag` combination, the two traits register the
    /// same concrete type under different tag names and content keys. Encoding
    /// a content type via `encode_asset` produces a payload tagged with
    /// `Content`'s registration, which is incompatible with the `Asset`-based
    /// decoding path. Always use this method for anything that implements
    /// `Content`, and reserve [`Self::encode_asset`] for pure `Asset` types.
    pub fn encode_content(&mut self, asset: &dyn Content) {
        let asset_encoded_data = asset::encode_asset(asset, self.endian_type).unwrap();
        let reader = Cursor::new(asset_encoded_data);
        let task = asset.build_resource_encode_task(reader);
        self.tasks.push(task);
    }

    pub fn finish(&mut self) -> Result<()> {
        encode_artifact_tasks_disk(
            self.endian_type,
            self.settings.clone(),
            &mut self.tasks,
            &self.target_path,
        )
    }
}

pub struct ArtifactReader {
    artifact_file_header: ArtifactFileHeader,
    #[cfg(not(target_os = "android"))]
    buf_reader: std::io::BufReader<std::fs::File>,
    #[cfg(target_os = "android")]
    buf_reader: crate::java_input_stream::JavaInputStream,
    payload_offset: u64,
    endian_type: Option<EEndianType>,
}

impl ArtifactReader {
    #[cfg(target_os = "android")]
    pub fn new(
        mut buf_reader: crate::java_input_stream::JavaInputStream,
        endian_type: Option<EEndianType>,
    ) -> Result<ArtifactReader> {
        let result = FileHeader::check_identification(
            &mut buf_reader,
            file_header::ARTIFACT_FILE_MAGIC_NUMBERS,
        );
        if let Err(err) = result {
            return Err(err);
        }

        let header_encoded_data_length =
            match FileHeader::get_header_encoded_data_length(&mut buf_reader, endian_type) {
                Ok(header_encoded_data_length) => header_encoded_data_length,
                Err(err) => {
                    return Err(err);
                }
            };

        let artifact_file_header: ArtifactFileHeader =
            match FileHeader::get_header2(&mut buf_reader, endian_type) {
                Ok(artifact_file_header) => artifact_file_header,
                Err(err) => {
                    return Err(err);
                }
            };

        let payload_offset: u64 =
            (IDENTIFICATION_SIZE + HEADER_LENGTH_SIZE) as u64 + header_encoded_data_length;

        return Ok(ArtifactReader {
            artifact_file_header,
            buf_reader,
            payload_offset,
            endian_type,
        });
    }

    #[cfg(not(target_os = "android"))]
    pub fn new(path: &Path, endian_type: Option<EEndianType>) -> Result<ArtifactReader> {
        let file = std::fs::File::open(path).map_err(|err| {
            let msg = format!("Can not open file {}", path.to_string_lossy().to_string());
            crate::error::Error::IO(err, Some(msg))
        })?;

        let mut buf_reader = std::io::BufReader::new(file);
        let _ = FileHeader::check_identification(
            &mut buf_reader,
            file_header::ARTIFACT_FILE_MAGIC_NUMBERS,
        )?;

        let header_encoded_data_length =
            FileHeader::get_header_encoded_data_length(&mut buf_reader, endian_type)?;

        let artifact_file_header: ArtifactFileHeader =
            FileHeader::get_header2(&mut buf_reader, endian_type)?;

        let payload_offset =
            (IDENTIFICATION_SIZE + HEADER_LENGTH_SIZE) as u64 + header_encoded_data_length;

        Ok(ArtifactReader {
            artifact_file_header,
            buf_reader,
            payload_offset,
            endian_type,
        })
    }

    pub fn get_artifact_file_header(&self) -> &ArtifactFileHeader {
        &self.artifact_file_header
    }

    /// Decode a pure asset from this artifact into a boxed trait object.
    ///
    /// The payload must have been encoded with
    /// [`ArtifactAssetEncoder::encode_asset`] (i.e. serialized under `Asset`'s
    /// typetag config: `content = "asset"`).
    ///
    /// # Warning
    ///
    /// Do **NOT** use this to read a content type. Because of the
    /// `erased_serde` + `typetag` combination, a payload encoded via
    /// [`ArtifactAssetEncoder::encode_content`] is tagged with `Content`'s
    /// registration (`content = "content"`), which this method's `Asset`-based
    /// decoding cannot match. Use [`Self::content`] for content types instead.
    pub fn asset(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<Box<dyn Asset>> {
        self.asset_internal(url, expected_resource_type)
    }

    /// Decode a content type from this artifact into a boxed trait object.
    ///
    /// The payload must have been encoded with
    /// [`ArtifactAssetEncoder::encode_content`] (i.e. serialized under
    /// `Content`'s typetag config: `content = "content"`).
    ///
    /// # Why not `asset`?
    ///
    /// `Content` is a subtrait of `Asset`, but because of the `erased_serde` +
    /// `typetag` combination, `Content`-encoded payloads carry `Content`'s tag
    /// registration, which [`Self::asset`] (an `Asset`-based decoder) cannot
    /// match back. Always pair `encode_content` with this method, and pair
    /// `encode_asset` with `Self::asset`.
    pub fn content(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<Box<dyn Content>> {
        self.asset_internal(url, expected_resource_type)
    }

    fn asset_internal<T>(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<Box<T>>
    where
        T: Asset + Serialize + ?Sized,
        Box<T>: Asset + DeserializeOwned,
    {
        let resource_info = self.artifact_file_header.resource_map.get(url).ok_or(
            crate::error::Error::NotFound(Some(format!("Resource does not contain {}", url))),
        )?;
        if let Some(expected_resource_type) = expected_resource_type {
            if resource_info.resource_type != expected_resource_type {
                return Err(crate::error::Error::ResourceTypeNotMatch(Some(format!(
                    "{:?} != expected resource type: {:?}",
                    resource_info.resource_type, expected_resource_type
                ))));
            }
        }
        let offset = resource_info.offset;
        let length = resource_info.length;
        let _ = self
            .buf_reader
            .seek(SeekFrom::Start(self.payload_offset + offset))
            .map_err(|err| {
                crate::error::Error::IO(err, Some(format!("Failed to seek {}", offset)))
            })?;
        let mut buf: Vec<u8> = vec![0; length as usize];
        let _ = self.buf_reader.read_exact(&mut buf).map_err(|err| {
            let msg = format!("Failed to read the exact number of bytes.");
            crate::error::Error::IO(err, Some(msg))
        })?;
        asset::decode_asset::<Box<T>>(
            &buf,
            self.endian_type,
            Some(resource_info.resource_type.clone()),
        )
    }

    pub fn check_assets(&mut self) -> Result<()> {
        for (url, resource_info) in self.artifact_file_header.resource_map.clone() {
            log::trace!("url: {}, type: {:?}", url, resource_info.resource_type);
            let offset = resource_info.offset;
            let length = resource_info.length;
            let _ = self
                .buf_reader
                .seek(SeekFrom::Start(self.payload_offset + offset))
                .map_err(|err| {
                    crate::error::Error::IO(
                        err,
                        Some(format!("Failed to seek {}", self.payload_offset + offset)),
                    )
                })?;
            let mut buf: Vec<u8> = vec![0; length as usize];
            let _ = self.buf_reader.read_exact(&mut buf).map_err(|err| {
                let msg = format!("Failed to read the exact number of bytes.");
                crate::error::Error::IO(err, Some(msg))
            })?;

            if resource_info.resource_type.kind() == ASSET_KIND {
                let asset = self.asset(&url, None)?;
                let message = format!(
                    "{:?}, {:?}",
                    asset.as_ref().resource_type(),
                    resource_info.resource_type
                );
                assert!(
                    asset.as_ref().resource_type() == resource_info.resource_type,
                    "{}",
                    message
                );
            } else if resource_info.resource_type.kind() == CONTENT_ASSET_KIND {
                let content = self.content(&url, None)?;
                let message = format!(
                    "{:?}, {:?}",
                    content.as_ref().resource_type(),
                    resource_info.resource_type
                );
                assert!(
                    content.as_ref().resource_type() == resource_info.resource_type,
                    "{}",
                    message
                );
            } else {
                unimplemented!();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::ArtifactFileHeader;

    #[test]
    fn test_case_artifact() {
        let artifact = ArtifactFileHeader::default();
        let encoded: Vec<u8> = crate::bincode_legacy::serialize(&artifact, None).unwrap();
        let _decoded: ArtifactFileHeader =
            crate::bincode_legacy::deserialize(&encoded[..], None).unwrap();
    }
}
