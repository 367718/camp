use std::{
    io,
    ptr::{ self, NonNull },
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection, Response,
};

// INVARIANT: drop order required by the WinHTTP API, do not rearrange
// handle -> connection
pub struct Request<'s> {
    handle: WinHttpHandle,
    #[allow(dead_code)]
    connection: Connection<'s>,
}

impl<'s> Request<'s> {
    
    pub(crate) fn open(connection: Connection<'s>, path: &[u16]) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        // enables TLS usage
        let flags = sys::WINHTTP_FLAG_SECURE;
        
        // SAFETY:
        // 1. `connection.handle.as_raw()` is a valid `HINTERNET` connection handle whose lifetime is guaranteed by the owned `Connection`
        // 2. `path.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. passing `null` for `pwsz_verb`, `pwsz_version`, `pwsz_referrer`, and `ppwsz_accept_types` selects standard WinHTTP API defaults
        // 4. `flags` is a valid option defined by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(), // default http verb: GET
                path.as_ptr(), // automatically escaped by WinHTTP as long as `WINHTTP_FLAG_ESCAPE_DISABLE` is not set
                ptr::null(), // HTTP/1.1 by default, overridden by HTTP/2 set in `session`
                ptr::null(), // no referer
                ptr::null(), // no accept header
                flags,
            )
            
        };
        
        let handle = match NonNull::new(result) {
            // SAFETY: `result` is a uniquely owned, valid, open request handle
            Some(result) => unsafe { WinHttpHandle::from_raw(result) },
            None => return Err(last_winhttp_error()),
        };
        
        // `connection` is kept alive to prevent its `handle` getting closed
        Ok(Self { handle, connection })
    }
    
    pub(crate) fn send(self) -> io::Result<Response<'s>> {
        // -------------------- send request --------------------
        
        // SAFETY:
        // 1. `self.handle.as_raw()` is a valid `HINTERNET` request handle that was verified to be non-null and has not been closed yet
        // 2. `null` for `lpsz_headers` and `0` for `dw_headers_length` are valid if no additional headers are to be sent
        // 3. `null` for `lp_optional` and `0` for `dw_optional_length` are valid if no body is to be sent
        // 4. `dw_total_length` as `0` is valid as no body is to be sent
        // 5. `dw_context` as `0` is valid and signals that no context is associated with the request
        let result = unsafe {
            
            sys::WinHttpSendRequest(
                self.handle.as_raw(),
                ptr::null(), // no additional headers
                0, // no additional headers
                ptr::null(), // no body
                0, // no body
                0, // no body
                0, // no context
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- receive response --------------------
        
        // SAFETY:
        // 1. `self.handle.as_raw()` is a valid `HINTERNET` request handle on which `WinHttpSendRequest` has successfully completed
        // 2. `lp_reserved` as `null` is required by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpReceiveResponse(
                self.handle.as_raw(),
                ptr::null(), // reserved
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // `connection` is kept alive to prevent its `handle` getting closed
        Response::new(self.handle, self.connection)
    }
    
}
