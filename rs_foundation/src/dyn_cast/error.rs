use std::fmt;

#[derive(Debug, Clone)]
pub struct DynCastError {
    pub requested: &'static str,
    pub supported: &'static [&'static str],
}

impl fmt::Display for DynCastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "dyn_cast: '{}' is not in the supported type list {:?}",
            self.requested, self.supported,
        )
    }
}

impl std::error::Error for DynCastError {}
