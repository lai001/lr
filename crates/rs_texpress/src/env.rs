use crate::TextureFormatType;
use rs_core_minimal::{file_manager::get_deps_dir, path_ext::CanonicalizeSlashExt};
use std::path::{Path, PathBuf};

pub mod ktxcli_supported {
    use crate::TextureFormatType;

    pub fn ldr() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ASTC_4x4_ldr,
            TextureFormatType::ASTC_5x4_ldr,
            TextureFormatType::ASTC_5x5_ldr,
            TextureFormatType::ASTC_6x5_ldr,
            TextureFormatType::ASTC_6x6_ldr,
            TextureFormatType::ASTC_8x5_ldr,
            TextureFormatType::ASTC_8x6_ldr,
            TextureFormatType::ASTC_8x8_ldr,
            TextureFormatType::ASTC_10x5_ldr,
            TextureFormatType::ASTC_10x6_ldr,
            TextureFormatType::ASTC_10x8_ldr,
            TextureFormatType::ASTC_10x10_ldr,
            TextureFormatType::ASTC_12x10_ldr,
            TextureFormatType::ASTC_12x12_ldr,
        ]
    }

    pub fn hdr() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ASTC_4x4_hdr,
            TextureFormatType::ASTC_5x4_hdr,
            TextureFormatType::ASTC_5x5_hdr,
            TextureFormatType::ASTC_6x5_hdr,
            TextureFormatType::ASTC_6x6_hdr,
            TextureFormatType::ASTC_8x5_hdr,
            TextureFormatType::ASTC_8x6_hdr,
            TextureFormatType::ASTC_8x8_hdr,
            TextureFormatType::ASTC_10x5_hdr,
            TextureFormatType::ASTC_10x6_hdr,
            TextureFormatType::ASTC_10x8_hdr,
            TextureFormatType::ASTC_10x10_hdr,
            TextureFormatType::ASTC_12x10_hdr,
            TextureFormatType::ASTC_12x12_hdr,
        ]
    }

    pub fn all() -> Vec<TextureFormatType> {
        let mut all = ldr();
        all.append(&mut hdr());
        all
    }

    pub fn dimension(texture_format_type: &TextureFormatType) -> String {
        assert!(all().contains(texture_format_type));
        match texture_format_type {
            TextureFormatType::ASTC_4x4_ldr => "4x4".to_string(),
            TextureFormatType::ASTC_5x4_ldr => "5x4".to_string(),
            TextureFormatType::ASTC_5x5_ldr => "5x5".to_string(),
            TextureFormatType::ASTC_6x5_ldr => "6x5".to_string(),
            TextureFormatType::ASTC_6x6_ldr => "6x6".to_string(),
            TextureFormatType::ASTC_8x5_ldr => "8x5".to_string(),
            TextureFormatType::ASTC_8x6_ldr => "8x6".to_string(),
            TextureFormatType::ASTC_8x8_ldr => "8x8".to_string(),
            TextureFormatType::ASTC_10x5_ldr => "10x5".to_string(),
            TextureFormatType::ASTC_10x6_ldr => "10x6".to_string(),
            TextureFormatType::ASTC_10x8_ldr => "10x8".to_string(),
            TextureFormatType::ASTC_10x10_ldr => "10x10".to_string(),
            TextureFormatType::ASTC_12x10_ldr => "12x10".to_string(),
            TextureFormatType::ASTC_12x12_ldr => "12x12".to_string(),
            TextureFormatType::ASTC_4x4_hdr => "4x4".to_string(),
            TextureFormatType::ASTC_5x4_hdr => "5x4".to_string(),
            TextureFormatType::ASTC_5x5_hdr => "5x5".to_string(),
            TextureFormatType::ASTC_6x5_hdr => "6x5".to_string(),
            TextureFormatType::ASTC_6x6_hdr => "6x6".to_string(),
            TextureFormatType::ASTC_8x5_hdr => "8x5".to_string(),
            TextureFormatType::ASTC_8x6_hdr => "8x6".to_string(),
            TextureFormatType::ASTC_8x8_hdr => "8x8".to_string(),
            TextureFormatType::ASTC_10x5_hdr => "10x5".to_string(),
            TextureFormatType::ASTC_10x6_hdr => "10x6".to_string(),
            TextureFormatType::ASTC_10x8_hdr => "10x8".to_string(),
            TextureFormatType::ASTC_10x10_hdr => "10x10".to_string(),
            TextureFormatType::ASTC_12x10_hdr => "12x10".to_string(),
            TextureFormatType::ASTC_12x12_hdr => "12x12".to_string(),
            _ => {
                panic!();
            }
        }
    }

    pub fn is_ldr(texture_format_type: &TextureFormatType) -> bool {
        assert!(all().contains(texture_format_type));
        match texture_format_type {
            TextureFormatType::ASTC_4x4_ldr => true,
            TextureFormatType::ASTC_5x4_ldr => true,
            TextureFormatType::ASTC_5x5_ldr => true,
            TextureFormatType::ASTC_6x5_ldr => true,
            TextureFormatType::ASTC_6x6_ldr => true,
            TextureFormatType::ASTC_8x5_ldr => true,
            TextureFormatType::ASTC_8x6_ldr => true,
            TextureFormatType::ASTC_8x8_ldr => true,
            TextureFormatType::ASTC_10x5_ldr => true,
            TextureFormatType::ASTC_10x6_ldr => true,
            TextureFormatType::ASTC_10x8_ldr => true,
            TextureFormatType::ASTC_10x10_ldr => true,
            TextureFormatType::ASTC_12x10_ldr => true,
            TextureFormatType::ASTC_12x12_ldr => true,
            TextureFormatType::ASTC_4x4_hdr => false,
            TextureFormatType::ASTC_5x4_hdr => false,
            TextureFormatType::ASTC_5x5_hdr => false,
            TextureFormatType::ASTC_6x5_hdr => false,
            TextureFormatType::ASTC_6x6_hdr => false,
            TextureFormatType::ASTC_8x5_hdr => false,
            TextureFormatType::ASTC_8x6_hdr => false,
            TextureFormatType::ASTC_8x8_hdr => false,
            TextureFormatType::ASTC_10x5_hdr => false,
            TextureFormatType::ASTC_10x6_hdr => false,
            TextureFormatType::ASTC_10x8_hdr => false,
            TextureFormatType::ASTC_10x10_hdr => false,
            TextureFormatType::ASTC_12x10_hdr => false,
            TextureFormatType::ASTC_12x12_hdr => false,
            _ => {
                panic!();
            }
        }
    }
}

pub mod compressonatorcli_supported {
    use crate::TextureFormatType;

    pub fn all() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ARGB_8888,
            TextureFormatType::ARGB_16F,
            TextureFormatType::ARGB_32F,
            TextureFormatType::RGBA_1010102,
            TextureFormatType::ATC_RGB,
            TextureFormatType::ATC_RGBA_Explicit,
            TextureFormatType::ATC_RGBA_Interpolated,
            TextureFormatType::ATI1N,
            TextureFormatType::ATI2N,
            TextureFormatType::ATI2N_XY,
            TextureFormatType::ATI2N_DXT5,
            TextureFormatType::BC1,
            TextureFormatType::BC2,
            TextureFormatType::BC3,
            TextureFormatType::BC4,
            TextureFormatType::BC4_S,
            TextureFormatType::BC5,
            TextureFormatType::BC5_S,
            TextureFormatType::BC6H,
            TextureFormatType::BC7,
            TextureFormatType::DXT1,
            TextureFormatType::DXT3,
            TextureFormatType::DXT5,
            TextureFormatType::DXT5_xGBR,
            TextureFormatType::DXT5_RxBG,
            TextureFormatType::DXT5_RBxG,
            TextureFormatType::DXT5_xRBG,
            TextureFormatType::DXT5_RGxB,
            TextureFormatType::DXT5_xGxR,
            TextureFormatType::ETC_RGB,
            TextureFormatType::ETC2_RGB,
            TextureFormatType::ETC2_RGBA,
            TextureFormatType::ETC2_RGBA1,
            TextureFormatType::BRLG,
        ]
    }

    pub fn to_arg(texture_format_type: &TextureFormatType) -> String {
        assert!(all().contains(texture_format_type));
        format!("{:?}", texture_format_type)
    }

    pub fn ktx2_uncompressed() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ARGB_8888,
            TextureFormatType::ARGB_16F,
            TextureFormatType::ARGB_32F,
        ]
    }

    pub fn dds_uncompressed() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ARGB_8888,
            TextureFormatType::ARGB_16F,
            TextureFormatType::ARGB_32F,
        ]
    }

    pub fn ktx2_compressed() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::BC1,
            TextureFormatType::BC2,
            TextureFormatType::BC3,
            TextureFormatType::BC4,
            TextureFormatType::BC4_S,
            TextureFormatType::BC5,
            TextureFormatType::BC5_S,
            TextureFormatType::BC6H,
            TextureFormatType::BC7,
            TextureFormatType::DXT1,
            TextureFormatType::DXT3,
            TextureFormatType::DXT5,
            TextureFormatType::ETC_RGB,
            TextureFormatType::ETC2_RGB,
            TextureFormatType::ETC2_RGBA,
            TextureFormatType::ETC2_RGBA1,
        ]
    }

    pub fn dds_compressed() -> Vec<TextureFormatType> {
        vec![
            TextureFormatType::ATC_RGB,
            TextureFormatType::ATC_RGBA_Explicit,
            TextureFormatType::ATC_RGBA_Interpolated,
            TextureFormatType::ATI1N,
            TextureFormatType::ATI2N,
            TextureFormatType::ATI2N_XY,
            TextureFormatType::ATI2N_DXT5,
            TextureFormatType::BC1,
            TextureFormatType::BC2,
            TextureFormatType::BC3,
            TextureFormatType::BC4,
            TextureFormatType::BC4_S,
            TextureFormatType::BC5,
            TextureFormatType::BC5_S,
            TextureFormatType::BC6H,
            TextureFormatType::BC7,
            TextureFormatType::DXT1,
            TextureFormatType::DXT3,
            TextureFormatType::DXT5,
            TextureFormatType::DXT5_xGBR,
            TextureFormatType::DXT5_RxBG,
            TextureFormatType::DXT5_RBxG,
            TextureFormatType::DXT5_xRBG,
            TextureFormatType::DXT5_RGxB,
            TextureFormatType::DXT5_xGxR,
            TextureFormatType::ETC_RGB,
            TextureFormatType::ETC2_RGB,
            TextureFormatType::ETC2_RGBA,
            TextureFormatType::ETC2_RGBA1,
        ]
    }
}

#[derive(Debug)]
pub struct TexpressEnv {
    compressonatorcli_path: PathBuf,
    ktx_path: PathBuf,
    ktx2check_path: PathBuf,
    ktxinfo_path: PathBuf,
    toktx_path: PathBuf,
}

impl TexpressEnv {
    fn find_program(deps_dir: &Path, pattern: &str) -> crate::error::Result<PathBuf> {
        let pattern = deps_dir.join(pattern);
        let pattern = pattern
            .as_os_str()
            .try_into()
            .map_err(|err| crate::error::Error::Utf8Error(err))?;
        let path = glob::glob(pattern)
            .map_err(|err| crate::error::Error::PatternError(err, None))?
            .next()
            .ok_or(crate::error::Error::Other(format!(
                "No paths that match the given pattern, {}",
                pattern
            )))?
            .map_err(|err| crate::error::Error::GlobError(err, None))?;
        let path = path
            .canonicalize_slash()
            .map_err(|err| crate::error::Error::IO(err, None))?;
        Ok(path)
    }

    fn new() -> crate::error::Result<TexpressEnv> {
        let deps_dir = get_deps_dir();
        let deps_dir = deps_dir
            .canonicalize()
            .map_err(|err| crate::error::Error::IO(err, None))?;
        assert!(deps_dir.is_dir());
        let compressonatorcli_path =
            Self::find_program(&deps_dir, "compressonatorcli-*-win64/compressonatorcli.exe")?;
        let ktx2check_path =
            Self::find_program(&deps_dir, "KTX-Software-*-Windows-x64/bin/ktx2check.exe")?;
        let ktx_path = Self::find_program(&deps_dir, "KTX-Software-*-Windows-x64/bin/ktx.exe")?;
        let ktxinfo_path =
            Self::find_program(&deps_dir, "KTX-Software-*-Windows-x64/bin/ktxinfo.exe")?;
        let toktx_path = Self::find_program(&deps_dir, "KTX-Software-*-Windows-x64/bin/toktx.exe")?;
        Ok(TexpressEnv {
            compressonatorcli_path,
            ktx_path,
            ktx2check_path,
            ktxinfo_path,
            toktx_path,
        })
    }

    pub fn is_valid_ktx2_file(&self, path: impl AsRef<Path>) -> crate::error::Result<()> {
        let path = path.as_ref();
        let output = std::process::Command::new(&self.ktx2check_path)
            .arg(path)
            .output()
            .map_err(|err| {
                crate::error::Error::IO(err, Some(format!("Failed to execute command")))
            })?;
        let stderr = unsafe { String::from_utf8_unchecked(output.stderr) };
        let stdout = unsafe { String::from_utf8_unchecked(output.stdout) };
        if output.status.success() {
            Ok(())
        } else {
            Err(crate::error::Error::Other(format!(
                "out: {}\nerr: {}",
                stdout, stderr
            )))
        }
    }

    pub fn is_valid_dds_file(&self, path: impl AsRef<Path>) -> crate::error::Result<()> {
        let file = std::fs::File::open(path).map_err(|err| crate::error::Error::IO(err, None))?;
        let _ = image_dds::ddsfile::Dds::read(file)
            .map_err(|err| crate::error::Error::DdsfileError(err))?;
        Ok(())
    }

    pub fn convert(
        &self,
        compression_format_type: TextureFormatType,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        logfile: Option<impl AsRef<Path>>,
    ) -> crate::error::Result<String> {
        if compressonatorcli_supported::all().contains(&compression_format_type) {
            let extension = output_path
                .as_ref()
                .extension()
                .ok_or(crate::error::Error::IO(
                    std::io::ErrorKind::InvalidFilename.into(),
                    None,
                ))?;
            if !(extension == "dds" || extension == "ktx2") {
                return Err(crate::error::Error::IO(
                    std::io::ErrorKind::InvalidFilename.into(),
                    None,
                ));
            }
            let mut command = std::process::Command::new(&self.compressonatorcli_path);
            if let Some(logfile) = logfile {
                let logfile = logfile.as_ref();
                command.arg("-logfile");
                command.arg(logfile);
            }
            command.arg("-fd");
            command.arg(compressonatorcli_supported::to_arg(
                &compression_format_type,
            ));
            command.arg(input_path.as_ref());
            command.arg(output_path.as_ref());
            let output = command
                .output()
                .map_err(|err| crate::error::Error::IO(err, None))?;
            let stderr: String = String::from_utf8(output.stderr)
                .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
            let stdout: String = String::from_utf8(output.stdout)
                .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
            assert!(stderr.is_empty());
            if output.status.success() {
                Ok(stdout)
            } else {
                Err(crate::error::Error::Other(format!(
                    "out: {}\nerr: {}",
                    stdout, stderr
                )))
            }
        } else if ktxcli_supported::all().contains(&compression_format_type) {
            let extension = output_path
                .as_ref()
                .extension()
                .ok_or(crate::error::Error::IO(
                    std::io::ErrorKind::InvalidFilename.into(),
                    None,
                ))?;
            if !(extension == "ktx2") {
                return Err(crate::error::Error::IO(
                    std::io::ErrorKind::InvalidFilename.into(),
                    None,
                ));
            }
            let mut command = std::process::Command::new(&self.toktx_path);
            command.arg("--t2");
            command.arg("--encode");
            command.arg("astc");
            command.arg("--astc_blk_d");
            command.arg(ktxcli_supported::dimension(&compression_format_type));
            command.arg("--astc_mode");
            if ktxcli_supported::is_ldr(&compression_format_type) {
                command.arg("ldr");
            } else {
                command.arg("hdr");
            }
            command.arg("--astc_quality");
            command.arg("100");

            command.arg(output_path.as_ref());
            command.arg(input_path.as_ref());

            let output = command
                .output()
                .map_err(|err| crate::error::Error::IO(err, None))?;
            let stderr: String = String::from_utf8(output.stderr)
                .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
            let stdout: String = String::from_utf8(output.stdout)
                .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
            if !stderr.is_empty() {
                Err(crate::error::Error::Other(format!("err: {}", stderr)))
            } else if output.status.success() {
                assert!(stderr.is_empty());
                Ok(format!("{stdout}"))
            } else {
                Err(crate::error::Error::Other(format!("err: {}", stderr)))
            }
        } else {
            unimplemented!();
        }
    }

    pub fn ktxinfo_path(&self) -> &PathBuf {
        &self.ktxinfo_path
    }

    pub fn ktx_path(&self) -> &PathBuf {
        &self.ktx_path
    }

    pub fn global_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut TexpressEnv) -> R,
    {
        GLOBAL_TEXPRESS_ENV.with_borrow_mut(f)
    }
}

thread_local! {
    static GLOBAL_TEXPRESS_ENV: std::cell::RefCell<TexpressEnv>  = std::cell::RefCell::new(TexpressEnv::new().expect("Valid")) ;
}

#[cfg(test)]
mod test {
    use crate::env::{compressonatorcli_supported, ktxcli_supported, TexpressEnv};
    use rs_core_minimal::file_manager::get_current_exe_dir;
    use std::path::PathBuf;

    fn get_tmp_folder() -> PathBuf {
        let current_exe_dir = get_current_exe_dir().expect("Valid path");
        current_exe_dir.join(module_path!().replace("::", "_"))
    }

    fn create_tmp_folder() -> PathBuf {
        let path = get_tmp_folder();
        if path.exists() {
            path
        } else {
            std::fs::create_dir_all(path.clone()).expect(&format!("{:?}", path));
            path
        }
    }

    #[test]
    fn test_env() {
        let _ = TexpressEnv::new().expect("Valid");
    }

    #[test]
    fn test_dds_compressed() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::RgbaImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_dds_compressed.dds");
        let input_path = tmp_folder.join("test_dds_compressed.png");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::Png)
            .expect("Saves the buffer to file");
        let logfile = tmp_folder.join("test_dds_compressed_process_results.txt");
        println!("{:?}", &logfile.as_os_str());
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::dds_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, Some(&logfile));
            if let Err(err) = out {
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn test_ktx2_compressed() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::RgbaImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_ktx2_compressed.ktx2");
        let input_path = tmp_folder.join("test_ktx2_compressed.png");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::Png)
            .expect("Saves the buffer to file");
        let logfile = tmp_folder.join("test_ktx2_compressed_process_results.txt");
        println!("{:?}", &logfile.as_os_str());
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::ktx2_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, Some(&logfile));
            if let Err(err) = out {
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn test_ktx2_uncompressed() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::Rgba32FImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_ktx2_uncompressed.ktx2");
        let input_path = tmp_folder.join("test_ktx2_uncompressed.exr");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::OpenExr)
            .expect("Saves the buffer to file");
        let logfile = tmp_folder.join("test_ktx2_uncompressed_process_results.txt");
        println!("{:?}", &logfile.as_os_str());
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::ktx2_uncompressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, Some(&logfile));
            if let Err(err) = out {
                panic!("{:?}, {}", ty, err);
            }
        }
    }

    #[test]
    fn test_dds_uncompressed() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::Rgb32FImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_dds_uncompressed.dds");
        let input_path = tmp_folder.join("test_dds_uncompressed.exr");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::OpenExr)
            .expect("Saves the buffer to file");
        let logfile = tmp_folder.join("test_dds_uncompressed_process_results.txt");
        println!("{:?}", &logfile.as_os_str());
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::dds_uncompressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, Some(&logfile));
            if let Err(err) = out {
                panic!("{:?}, {}", ty, err);
            }
        }
    }

    #[test]
    fn test_is_valid_ktx2_file() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::RgbaImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_is_valid_ktx2_file.ktx2");
        let input_path = tmp_folder.join("test_is_valid_ktx2_file.png");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::Png)
            .expect("Saves the buffer to file");
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::ktx2_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, None::<&std::path::Path>);
            if let Ok(_) = out {
                break;
            }
        }
        assert!(output_path.is_file());
        assert!(texpress_env.is_valid_ktx2_file(output_path).is_ok());

        let output_path = tmp_folder.join("test_is_valid_ktx2_file.dds");
        for ty in compressonatorcli_supported::dds_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, None::<&std::path::Path>);
            if let Ok(_) = out {
                break;
            }
        }

        assert!(output_path.is_file());
        assert!(texpress_env.is_valid_ktx2_file(output_path).is_err());
    }

    #[test]
    fn test_is_valid_dds_file() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let rgba_image = image::RgbaImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_is_valid_dds_file.dds");
        let input_path = tmp_folder.join("test_is_valid_dds_file.png");
        rgba_image
            .save_with_format(&input_path, image::ImageFormat::Png)
            .expect("Saves the buffer to file");
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in compressonatorcli_supported::dds_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, None::<&std::path::Path>);
            if let Ok(_) = out {
                break;
            }
        }
        assert!(output_path.is_file());
        assert!(texpress_env.is_valid_dds_file(output_path).is_ok());

        let output_path = tmp_folder.join("test_is_valid_dds_file.ktx2");
        for ty in compressonatorcli_supported::ktx2_compressed() {
            let out = texpress_env.convert(ty, &input_path, &output_path, None::<&std::path::Path>);
            if let Ok(_) = out {
                break;
            }
        }

        assert!(output_path.is_file());
        assert!(texpress_env.is_valid_dds_file(output_path).is_err());
    }

    #[test]
    fn test_file_extension() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let out = texpress_env.convert(
            crate::TextureFormatType::BC7,
            "",
            "abc",
            None::<&std::path::Path>,
        );
        assert!(out.is_err());
        if let Err(err) = out {
            match err {
                crate::error::Error::IO(error, _) => {
                    assert_eq!(error.kind(), std::io::ErrorKind::InvalidFilename);
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_astc_ktx2_compressed() {
        let texpress_env = TexpressEnv::new().expect("Valid");
        let image = image::RgbaImage::new(512, 512);
        let tmp_folder = create_tmp_folder();
        let output_path = tmp_folder.join("test_astc_ktx2_compressed.ktx2");
        let input_path = tmp_folder.join("test_astc_ktx2_compressed.png");
        image
            .save_with_format(&input_path, image::ImageFormat::Png)
            .expect("Saves the buffer to file");
        println!("{:?}", &input_path.as_os_str());
        println!("{:?}", &output_path.as_os_str());
        for ty in ktxcli_supported::ldr() {
            let out = texpress_env.convert(ty, &input_path, &output_path, None::<&std::path::Path>);
            if let Err(err) = out {
                panic!("{}", err);
            }
            assert!(texpress_env.is_valid_ktx2_file(&output_path).is_ok());
        }
    }
}
