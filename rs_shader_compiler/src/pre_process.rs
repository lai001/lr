use std::path::PathBuf;
#[cfg(feature = "editor")]
mod editor_mod {
    pub use path_slash::PathBufExt;
    pub use std::path::Path;
}
#[cfg(feature = "editor")]
use editor_mod::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct ShaderDescription {
    pub shader_path: PathBuf,
    pub include_dirs: Vec<PathBuf>,
    pub definitions: Vec<String>,
}

#[cfg(feature = "editor")]
pub fn pre_process(
    shader_path: &Path,
    include_dirs: impl Iterator<Item = impl AsRef<Path>>,
    definitions: impl Iterator<Item = impl AsRef<str>>,
) -> crate::error::Result<String> {
    let shader_path = dunce::canonicalize(shader_path).map_err(|err| {
        crate::error::Error::IO(err, Some(format!("{:?} is not exist.", shader_path)))
    })?;
    let mut clang = std::process::Command::new("clang");
    clang.arg("-E");
    clang.arg("-P");
    clang.arg("-x");
    clang.arg("c");
    clang.arg("-std=c11");
    for include_dir in include_dirs {
        let include_dir = include_dir.as_ref();
        let include_dir = dunce::canonicalize(include_dir).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("{:?} is not exist.", include_dir)))
        })?;
        let include_dir = include_dir.to_slash_lossy();
        clang.arg(format!("-I{}", include_dir));
    }
    for definition in definitions {
        clang.arg(format!("-D{}", definition.as_ref()));
    }
    let path_arg = shader_path.to_str().ok_or(crate::error::Error::IO(
        std::io::ErrorKind::Other.into(),
        None,
    ))?;
    clang.arg(path_arg);
    let output = clang.output();
    let output = output.map_err(|err| crate::error::Error::IO(err, None))?;
    let stderr = String::from_utf8(output.stderr);
    let stdout = String::from_utf8(output.stdout);
    let stdout = stdout.map_err(|err| crate::error::Error::FromUtf8Error(err))?;
    let stderr = stderr.map_err(|err| crate::error::Error::FromUtf8Error(err))?;
    if output.status.success() {
        Ok(stdout.to_string())
    } else {
        Err(crate::error::Error::ProcessFail(Some(stderr)))
    }
}
