// https://github.dev/dotnet/runtime/blob/0912e94a6c81f4743946831137ba6c3ec3f1a179/src/native/corehost/error_codes.h#L8
#[repr(u32)]
#[allow(non_camel_case_types)]
#[rustfmt::skip]
#[derive(Debug, PartialEq)]
pub enum StatusCode {
    // Success
    Success                             = 0,
    Success_HostAlreadyInitialized      = 0x00000001,
    Success_DifferentRuntimeProperties  = 0x00000002,

    // Failure
    InvalidArgFailure                   = 0x80008081,
    CoreHostLibLoadFailure              = 0x80008082,
    CoreHostLibMissingFailure           = 0x80008083,
    CoreHostEntryPointFailure           = 0x80008084,
    CoreHostCurHostFindFailure          = 0x80008085,
    // unused                           = 0x80008086,
    CoreClrResolveFailure               = 0x80008087,
    CoreClrBindFailure                  = 0x80008088,
    CoreClrInitFailure                  = 0x80008089,
    CoreClrExeFailure                   = 0x8000808a,
    ResolverInitFailure                 = 0x8000808b,
    ResolverResolveFailure              = 0x8000808c,
    LibHostCurExeFindFailure            = 0x8000808d,
    LibHostInitFailure                  = 0x8000808e,
    // unused                           = 0x8000808f,
    LibHostExecModeFailure              = 0x80008090,
    LibHostSdkFindFailure               = 0x80008091,
    LibHostInvalidArgs                  = 0x80008092,
    InvalidConfigFile                   = 0x80008093,
    AppArgNotRunnable                   = 0x80008094,
    AppHostExeNotBoundFailure           = 0x80008095,
    FrameworkMissingFailure             = 0x80008096,
    HostApiFailed                       = 0x80008097,
    HostApiBufferTooSmall               = 0x80008098,
    LibHostUnknownCommand               = 0x80008099,
    LibHostAppRootFindFailure           = 0x8000809a,
    SdkResolverResolveFailure           = 0x8000809b,
    FrameworkCompatFailure              = 0x8000809c,
    FrameworkCompatRetry                = 0x8000809d,
    // unused                           = 0x8000809e,
    BundleExtractionFailure             = 0x8000809f,
    BundleExtractionIOError             = 0x800080a0,
    LibHostDuplicateProperty            = 0x800080a1,
    HostApiUnsupportedVersion           = 0x800080a2,
    HostInvalidState                    = 0x800080a3,
    HostPropertyNotFound                = 0x800080a4,
    CoreHostIncompatibleConfig          = 0x800080a5,
    HostApiUnsupportedScenario          = 0x800080a6,
    HostFeatureDisabled                 = 0x800080a7,
}

impl TryFrom<u32> for StatusCode {
    type Error = String;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        Ok(unsafe { std::mem::transmute(value) })
    }
}

#[derive(Debug)]
pub enum Error {
    Host(StatusCode),
    Null,
    IO(std::io::Error, Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_ref())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
