use std::{
    fs::{ OpenOptions, File },
    io::{ self, Write, Error, ErrorKind },
    os::raw::*,
};

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

const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

pub struct Pipe<'n> {
    name: &'n str,
    connection: Option<File>,
}

impl<'n> Pipe<'n> {
    
    pub fn new(name: &'n str) -> Self {
        Self {
            name,
            connection: None,
        }
    }
    
    fn connect(&mut self) -> io::Result<()> {
        unsafe {
            
            let result = ffi::WaitNamedPipeW(
                chikuwa::WinString::from(self.name).as_ptr(),
                PIPE_MAX_WAIT,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        let connection = OpenOptions::new()
            .write(true)
            .open(self.name)?;
        
        self.connection = Some(connection);
        
        Ok(())
    }
    
}

impl<'n> Write for Pipe<'n> {
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        
        if self.connection.is_none() {
            self.connect()?;
        }
        
        let mut retried = false;
        
        loop {
            
            match self.connection.as_mut().unwrap().write(buf) {
                
                Ok(bytes) => break Ok(bytes),
                
                Err(error) => {
                    
                    // retry connection at least once
                    if ! retried && error.kind() == ErrorKind::BrokenPipe {
                        self.connect()?;
                        retried = true;
                        continue;
                    }
                    
                    break Err(error);
                    
                }
            }
            
        }
        
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.connection.as_mut()
            .ok_or(Error::other("The pipe connection is closed"))?
            .flush()
    }
    
}
