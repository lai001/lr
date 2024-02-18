use crate::error::Result;
use crate::level::Level;
use crate::{
    asset::{self, Asset},
    file_header::{
        self, FileHeader, ARTIFACT_FILE_MAGIC_NUMBERS, HEADER_LENGTH_SIZE, IDENTIFICATION_SIZE,
    },
    image::Image,
    resource_info::ResourceInfo,
    resource_type::EResourceType,
    shader_source_code::ShaderSourceCode,
    static_mesh::StaticMesh,
    EEndianType,
};
use rs_core_minimal::settings::Settings;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{
    collections::HashMap,
    io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArtifactFileHeader {
    pub settings: Settings,
    pub resource_map: std::collections::HashMap<url::Url, ResourceInfo>,
}

pub struct ResourceEncodeTask<R>
where
    R: Seek + Read,
{
    pub url: url::Url,
    pub resource_type: EResourceType,
    pub reader: R,
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
            resource_type: task.resource_type,
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
        std::io::copy(&mut task.reader, &mut buf_writer)
            .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to copy data."))))?;
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
    T: Asset,
{
    let mut tasks: Vec<ResourceEncodeTask<Cursor<Vec<u8>>>> = Vec::new();
    for asset in assets {
        let asset_encoded_data =
            asset::encode_asset(asset.get_resource_type(), endian_type, asset)?;
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

    pub fn encode<T>(&mut self, asset: &T)
    where
        T: Asset,
    {
        let asset_encoded_data =
            asset::encode_asset(asset.get_resource_type(), self.endian_type, asset).unwrap();
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

    pub fn get_resource<T>(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<T>
    where
        T: Asset,
    {
        let resource_info = self.artifact_file_header.resource_map.get(url).ok_or(
            crate::error::Error::NotFound(Some(format!("Resource does not contain {}.", url))),
        )?;
        if Some(resource_info.resource_type) != expected_resource_type {
            return Err(crate::error::Error::ResourceTypeNotMatch);
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
        asset::decode_asset::<T>(&buf, self.endian_type, Some(resource_info.resource_type))
    }

    pub fn check_assets(&mut self) -> Result<()> {
        for (_, resource_info) in &self.artifact_file_header.resource_map {
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
            match resource_info.resource_type {
                EResourceType::Image => {
                    let asset = asset::decode_asset::<Image>(
                        &buf,
                        self.endian_type,
                        Some(resource_info.resource_type),
                    )?;
                    log::trace!(
                        "url: {}, type: {:?}",
                        asset.url,
                        resource_info.resource_type
                    );
                }
                EResourceType::StaticMesh => {
                    let asset = asset::decode_asset::<StaticMesh>(
                        &buf,
                        self.endian_type,
                        Some(resource_info.resource_type),
                    )?;
                    log::trace!(
                        "url: {}, type: {:?}",
                        asset.url,
                        resource_info.resource_type
                    );
                }
                EResourceType::ShaderSourceCode => {
                    let asset = asset::decode_asset::<ShaderSourceCode>(
                        &buf,
                        self.endian_type,
                        Some(resource_info.resource_type),
                    )?;
                    log::trace!(
                        "url: {}, type: {:?}",
                        asset.url,
                        resource_info.resource_type
                    );
                }
                EResourceType::Level => {
                    let asset = asset::decode_asset::<Level>(
                        &buf,
                        self.endian_type,
                        Some(resource_info.resource_type),
                    )?;
                    log::trace!(
                        "url: {}, type: {:?}",
                        asset.url,
                        resource_info.resource_type
                    );
                }
                _ => {}
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
        let encoded: Vec<u8> = bincode::serialize(&artifact).unwrap();
        let decoded: ArtifactFileHeader = bincode::deserialize(&encoded[..]).unwrap();
    }
}
