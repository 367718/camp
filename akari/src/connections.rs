use std::{
    collections::{
        HashMap,
        hash_map::{ Entry, DefaultHasher },
    },
    hash::Hasher,
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
};

use crate::{ HttpHandle, Session };

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    fn WinHttpConnect(
        h_session: RawHandle,
        pswz_server_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        n_server_port: c_ushort, // INTERNET_PORT -> WORD
        dw_reserved: c_ulong,
    ) -> RawHandle;
    
}

pub struct Connections {
    inner: HashMap<u64, Connection>,
}

pub struct Connection {
    pub handle: HttpHandle,
}

impl Connections {
    
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    
    pub fn open_or_reuse(&mut self, session: &Session, host: &str, port: u16) -> io::Result<&Connection> {
        let mut hasher = DefaultHasher::new();
        hasher.write(host.as_bytes());
        hasher.write_u16(port);
        let key = hasher.finish();
        
        let connection = match self.inner.entry(key) {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => vacant.insert(Connection::new(session, host, port)?),
        };
        
        Ok(connection)
    }
    
}

impl Connection {
    
    fn new(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let handle = unsafe {
            
            let result = WinHttpConnect(
                session.handle.as_raw(),
                chikuwa::win_string(host).as_ptr(),
                port,
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            HttpHandle::new(result)
            
        };
        
        Ok(Self {
            handle,
        })
    }
    
}
