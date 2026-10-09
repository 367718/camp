use std::{
    io::{ self, Error, ErrorKind, Read },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use super::HttpHandle;

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    fn WinHttpReadData(
        h_request: RawHandle,
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
}

pub struct Response {
    pub(crate) handle: HttpHandle,
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        let bytes = c_ulong::try_from(buf.len())
            .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
        
        unsafe {
            
            let result = WinHttpReadData(
                self.handle.as_raw(),
                ptr::from_mut(buf).cast::<c_void>(),
                bytes,
                &raw mut amount_read,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}
