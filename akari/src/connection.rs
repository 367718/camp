use std::io;

use crate::{
    sys,
    WinHttpHandle, Session,
};

pub(crate) struct Connection<'s> {
    pub(crate) handle: WinHttpHandle,
    #[allow(dead_code)]
    session: &'s Session,
}

impl<'s> Connection<'s> {
    
    pub(crate) fn open(session: &'s Session, host: &str, port: u16) -> io::Result<Self> {
        let host_wide = chikuwa::win_string(host)?;
        
        // SAFETY:
        // 1. `session.handle.as_raw()` is a valid `HINTERNET` session handle whose lifetime is guaranteed by the borrowed `&Session`
        // 2. `host_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `dw_reserved` as `0` is required by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpConnect(
                session.handle.as_raw(),
                host_wide.as_ptr(),
                port,
                0, // reserved
            )
            
        };
        
        // SAFETY: if non-null, `result` is a valid, open connection handle uniquely owned
        let handle = unsafe { WinHttpHandle::from_raw(result)? };
        
        // `session` is kept alive to prevent its `handle` getting closed
        Ok(Self { handle, session })
    }
    
}
