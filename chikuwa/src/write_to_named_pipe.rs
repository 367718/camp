use std::{
    fs::OpenOptions,
    io::{ self, Write, Error },
    os::raw::*,
};

use super::WinString;

const MAX_WAIT: c_ulong = 5000; // milliseconds

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
        pub fn WaitNamedPipeW(
            lpNamedPipeName: *const c_ushort,
            nTimeOut: c_ulong,
        ) -> c_int;
        
    }
    
}

pub fn write_to_named_pipe(path: &str, data: &[u8]) -> io::Result<()> {
    unsafe {
        
        let result = ffi::WaitNamedPipeW(
            WinString::from(path).as_ptr(),
            MAX_WAIT,
        );
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
    }
    
    OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|mut pipe| pipe.write_all(data))
}
