use std::{
    fs::{ OpenOptions, File },
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

pub struct NamedPipe {
    handle: File,
}

impl NamedPipe {
    
    pub fn connect(name: &str) -> io::Result<Self> {
        unsafe {
            
            let result = ffi::WaitNamedPipeW(
                WinString::from(name).as_ptr(),
                MAX_WAIT,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        let handle = OpenOptions::new()
            .write(true)
            .open(name)?;
        
        Ok(Self {
            handle,
        })
    }
    
}

impl Write for NamedPipe {
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // no timeout
        self.handle.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
    
}
