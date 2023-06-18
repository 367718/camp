use std::{
    ops::Deref,
    os::{
        raw::*,
        windows::ffi::OsStrExt,
    },
    path::Path,
};

pub struct WinString(Vec<c_ushort>);

impl From<&str> for WinString {
    
    fn from(base: &str) -> Self {
        Self(base.encode_utf16()
            .chain(Some(0))
            .collect())
    }
    
}

impl From<&Path> for WinString {
    
    fn from(base: &Path) -> Self {
        Self(base.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect())
    }
    
}

impl Deref for WinString {
    
    type Target = Vec<c_ushort>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
    
}
