use std::{
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use crate::{ HttpHandle, Response };

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    fn WinHttpReceiveResponse(
        h_request: RawHandle,
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
}

pub struct Request {
    pub handle: HttpHandle,
}

impl Request {
    
    pub fn receive_response(self) -> io::Result<Response> {
        unsafe {
            
            let result = WinHttpReceiveResponse(
                self.handle.as_raw(),
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Response {
            handle: self.handle,
        })
    }
    
}
