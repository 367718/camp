use std::{
    io,
    ptr::NonNull,
};

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
        // 1. `session.handle.as_raw()` is a valid `HINTERNET` session handle whose lifetime is guaranteed by the borrowed `&Session`
        // 2. `host.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `dw_reserved` as `0` is required by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpConnect(
                session.handle.as_raw(),
                host.as_ptr(),
                port,
                0, // reserved
            )
            
        };
        
        let handle = match NonNull::new(result) {
            // SAFETY: `result` is a uniquely owned, valid, open connection handle
            Some(result) => unsafe { WinHttpHandle::from_raw(result) },
            None => return Err(last_winhttp_error()),
        };
        
        // `session` is kept alive to prevent its `handle` getting closed
        Ok(Self { handle, session })
    }
    
}
