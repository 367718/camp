use std::{
    ops::Deref,
    os::raw::*,
};

pub struct WinString(Vec<c_ushort>);

impl From<&str> for WinString {
    
    fn from(base: &str) -> Self {
        Self(base.encode_utf16()
            .chain(Some(0))
            .collect())
    }
    
}

impl Deref for WinString {
    
    type Target = [c_ushort];
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
    
}
