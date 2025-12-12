#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct ShaderDescription {
    pub shader_path: std::path::PathBuf,
    pub include_dirs: Vec<std::path::PathBuf>,
    pub definitions: Vec<String>,
}

pub fn pre_process(
    shader_path: &std::path::Path,
    include_dirs: impl Iterator<Item = impl AsRef<std::path::Path>>,
    definitions: impl Iterator<Item = impl AsRef<str>>,
) -> crate::error::Result<String> {
    #[cfg(all(feature = "editor", not(feature = "force_builtin_processor")))]
    if rs_foundation::is_program_in_path("cl.exe") {
        return pre_process_cl(shader_path, include_dirs, definitions);
    } else if rs_foundation::is_program_in_path("clang.exe") {
        return pre_process_clang(shader_path, include_dirs, definitions);
    }
    return pre_process_builtin(shader_path, include_dirs, definitions);
}

#[cfg(all(feature = "editor", not(feature = "force_builtin_processor")))]
fn pre_process_clang(
    shader_path: &std::path::Path,
    include_dirs: impl Iterator<Item = impl AsRef<std::path::Path>>,
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
        let include_dir = path_slash::PathBufExt::to_slash_lossy(&include_dir);
        if include_dir.is_empty() {
            return Err(crate::error::Error::ProcessFail(Some(String::from(
                "Empty include path",
            ))));
        }
        clang.arg(format!("-I{}", include_dir));
    }
    for definition in definitions {
        if definition.as_ref().is_empty() {
            return Err(crate::error::Error::ProcessFail(Some(String::from(
                "Empty macro definition",
            ))));
        }
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

#[cfg(all(feature = "editor", not(feature = "force_builtin_processor")))]
fn pre_process_cl(
    shader_path: &std::path::Path,
    include_dirs: impl Iterator<Item = impl AsRef<std::path::Path>>,
    definitions: impl Iterator<Item = impl AsRef<str>>,
) -> crate::error::Result<String> {
    let shader_path = dunce::canonicalize(shader_path).map_err(|err| {
        crate::error::Error::IO(err, Some(format!("{:?} is not exist.", shader_path)))
    })?;
    let mut cl = std::process::Command::new("cl.exe");
    cl.arg("/C");
    cl.arg("/EP");
    for include_dir in include_dirs {
        let include_dir = include_dir.as_ref();
        let include_dir = dunce::canonicalize(include_dir).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("{:?} is not exist.", include_dir)))
        })?;
        let include_dir = path_slash::PathBufExt::to_slash_lossy(&include_dir);
        if include_dir.is_empty() {
            return Err(crate::error::Error::ProcessFail(Some(String::from(
                "Empty include path",
            ))));
        }
        cl.arg(format!("/I\"{}\"", include_dir));
    }
    for definition in definitions {
        if definition.as_ref().is_empty() {
            return Err(crate::error::Error::ProcessFail(Some(String::from(
                "Empty macro definition",
            ))));
        }
        cl.arg(format!("/D{}", definition.as_ref()));
    }
    let path_arg = shader_path.to_str().ok_or(crate::error::Error::IO(
        std::io::ErrorKind::Other.into(),
        None,
    ))?;
    cl.arg(path_arg);
    let output = cl.output();
    let output = output.map_err(|err| crate::error::Error::IO(err, None))?;

    #[cfg(feature = "detect_encoding")]
    {
        let mut encoding_detector = chardetng::EncodingDetector::new();
        if !output.stdout.is_empty() {
            encoding_detector.feed(&output.stdout, true);
        } else if !output.stderr.is_empty() {
            encoding_detector.feed(&output.stderr, true);
        }
        let guess_encoding = encoding_detector.guess(None, true);
        let stderr = guess_encoding.decode(&output.stderr).0;
        let stdout = guess_encoding.decode(&output.stdout).0;
        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Err(crate::error::Error::ProcessFail(Some(stderr.to_string())))
        }
    }
    #[cfg(not(feature = "detect_encoding"))]
    {
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
}

fn parse_definitions_strict<I, S>(
    definitions: I,
) -> Result<std::collections::HashMap<String, String>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut map = std::collections::HashMap::new();

    for (idx, def) in definitions.into_iter().enumerate() {
        let s = def.as_ref();

        let Some(eq_pos) = s.find('=') else {
            map.insert(s.to_string(), "".to_string());
            continue;
            // return Err(format!(
            //     "Invalid definition at index {idx}: missing '=' -> {s:?}"
            // ));
        };
        if s[eq_pos + 1..].contains('=') {
            return Err(format!(
                "Invalid definition at index {idx}: multiple '=' -> {s:?}"
            ));
        }

        let key = &s[..eq_pos];
        let value = &s[eq_pos + 1..];

        if key.is_empty() {
            return Err(format!(
                "Invalid definition at index {idx}: empty key -> {s:?}"
            ));
        }
        // if value.is_empty() {
        //     return Err(format!("Invalid definition at index {idx}: empty value -> {s:?}"));
        // }
        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}

fn pre_process_builtin(
    shader_path: &std::path::Path,
    include_dirs: impl Iterator<Item = impl AsRef<std::path::Path>>,
    definitions: impl Iterator<Item = impl AsRef<str>>,
) -> crate::error::Result<String> {
    let defines = parse_definitions_strict(definitions)
        .map_err(|err| crate::error::Error::ProcessFail(Some(err)))?;
    let mut include_dirs: std::collections::HashSet<String> = include_dirs
        .filter_map(|p| p.as_ref().to_str().map(|s| s.to_string()))
        .collect();
    let parent = shader_path.parent().expect("Valid parent path");
    include_dirs.insert(parent.to_string_lossy().to_string());
    let mut processor = rs_cprep::processor::Preprocessor::new(include_dirs, defines);
    processor
        .process_file(shader_path)
        .map_err(|err| crate::error::Error::ProcessFail(Some(format!("{err}"))))
}
