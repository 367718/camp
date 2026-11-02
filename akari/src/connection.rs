use std::io;

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Session,
};

pub struct Connection {
    pub handle: WinHttpHandle,
}

impl Connection {
    
    pub fn open(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let wide_host = chikuwa::win_string(host)?;
        
        // SAFETY:
        // 1. `session.handle.as_raw()` is a valid `HINTERNET` session handle whose lifetime is guaranteed by the borrowed `&Session`
        // 2. `wide_host.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `dw_reserved` as `0` is required by the WinHTTP API
        // 4. on failure, `null` is returned and the proper error is constructed and propagated
        // 5. on success, the returned handle is wrapped in `WinHttpHandle`, which calls `WinHttpCloseHandle` on drop
        let handle = unsafe {
            
            let result = sys::WinHttpConnect(
                session.handle.as_raw(),
                wide_host.as_ptr(),
                port,
                0, // reserved
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        Ok(Self { handle })
    }
    
}
