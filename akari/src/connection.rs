use std::io;

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Session,
};

pub(crate) struct Connection<'s> {
    pub(crate) handle: WinHttpHandle,
    #[allow(dead_code)]
    session: &'s Session,
}

impl<'s> Connection<'s> {
    
    pub(crate) fn open(session: &'s Session, host: &[u16], port: u16) -> io::Result<Self> {
        // SAFETY:
        // 1. `session.handle.inner()` is a valid `HINTERNET` session handle whose lifetime is guaranteed by the borrowed `&Session`
        // 2. `host.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `dw_reserved` as `0` is required by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpConnect(
                session.handle.inner(),
                host.as_ptr(),
                port,
                0, // reserved
            )
            
        };
        
        let handle = result
            // SAFETY: `raw` is a uniquely owned, valid, open connection handle
            .map(|raw| unsafe { WinHttpHandle::from_raw(raw) })
            .ok_or_else(last_winhttp_error)?;
        
        // `session` is kept alive to prevent its `handle` getting closed
        Ok(Self { handle, session })
    }
    
}
