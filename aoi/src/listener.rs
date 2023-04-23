use std::{
    ffi::c_void,
    io::{ Error, ErrorKind },
    mem,
    net::TcpStream,
    os::{
        raw::*,
        windows::{
            io::FromRawSocket,
            raw::SOCKET,
        },
    },
    ptr,
    sync::{ Arc, Weak },
};

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsastartup
        pub fn WSAStartup(
            wversionrequested: c_ushort,
            lpwsadata: *mut c_void, // WSADATA
        ) -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsacleanup
        pub fn WSACleanup() -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsastringtoaddressw
        pub fn WSAStringToAddressW(
            addressstring: *const c_ushort,
            addressfamily: c_int,
            lpprotocolinfo: *const c_void, // WSAPROTOCOL_INFOW
            lpaddress: *mut Sockaddr,
            lpaddresslength: *mut c_int,
        ) -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsasocketw
        pub fn WSASocketW(
            af: c_int,
            _type: c_int,
            protocol: c_int,
            lpprotocolinfo: *const c_void, // WSAPROTOCOL_INFOW
            g: c_uint,
            dwflags: c_ulong,
        ) -> SOCKET;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsagetlasterror
        pub fn WSAGetLastError() -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
        pub fn bind(
            s: SOCKET,
            name: *const Sockaddr,
            namelen: c_int,
        ) -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-listen
        pub fn listen(
            s: SOCKET,
            backlog: c_int,
        ) -> c_int;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept
        pub fn accept(
            s: SOCKET,
            addr: *mut Sockaddr,
            addrlen: *mut c_int,
        ) -> SOCKET;
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-closesocket
        pub fn closesocket(
            s: SOCKET,
        ) -> c_int;
        
    }
    
    #[repr(C)]
    // https://learn.microsoft.com/en-us/windows/win32/api/winsock/ns-winsock-sockaddr
    pub struct Sockaddr {
        pub sa_family: c_ushort,
        pub sa_data: [c_char; 14],
    }
    
}

pub struct Listener {
    socket: Weak<SOCKET>,
}

pub struct ListenerStopper {
    socket: Option<Arc<SOCKET>>,
}

impl Listener {
    
    // ---------- constructors ----------
    
    
    pub fn new(bind_address: &str) -> Result<(Self, ListenerStopper), Error> {
        // ---------- startup ----------
        
        let startup = unsafe {
            
            let mut imp_details = mem::zeroed();
            
            ffi::WSAStartup(
                0x202, // 2.2
                &mut imp_details,
            )
            
        };
        
        if startup != 0 {
            return Err(Error::from_raw_os_error(startup));
        }
        
        // ---------- address ----------
        
        let encoded_address: Vec<c_ushort> = bind_address.encode_utf16()
            .chain(Some(0))
            .collect();
        
        let mut address = unsafe {
            
            mem::zeroed::<ffi::Sockaddr>()
            
        };
        
        let conversion = unsafe {
            
            let mut address_length = mem::size_of_val(&address) as c_int;
            
            ffi::WSAStringToAddressW(
                encoded_address.as_ptr(),
                2, // AF_INET
                ptr::null(),
                &mut address,
                &mut address_length,
            )
            
        };
        
        if conversion != 0 {
            
            let error = unsafe {
                
                ffi::WSAGetLastError()
                
            };
            
            close_and_cleanup(None);
            
            return Err(Error::from_raw_os_error(error));
            
        }
        
        // ---------- socket ----------
        
        let socket = unsafe {
            
            ffi::WSASocketW(
                2, // AF_INET
                1, // SOCK_STREAM
                6, // IPPROTO_TCP
                ptr::null_mut(),
                0x01, // SG_UNCONSTRAINED_GROUP
                0x01 | 0x80, // WSA_FLAG_OVERLAPPED, WSA_FLAG_NO_HANDLE_INHERIT
            )
            
        };
        
        if socket == ! 0 as SOCKET {
            
            close_and_cleanup(None);
            
            return Err(Error::last_os_error());
            
        }
        
        // ---------- bind ----------
        
        let bind = unsafe {
            
            ffi::bind(
                socket,
                &address,
                mem::size_of_val(&address) as c_int,
            )
            
        };
        
        if bind != 0 {
            
            let error = unsafe {
                
                ffi::WSAGetLastError()
                
            };
            
            close_and_cleanup(Some(socket));
            
            return Err(Error::from_raw_os_error(error));
            
        }
        
        // ---------- listen ----------
        
        let listen = unsafe {
            
            ffi::listen(
                socket,
                128,
            )
            
        };
        
        if listen == -1 {
            
            let error = unsafe {
                
                ffi::WSAGetLastError()
                
            };
            
            close_and_cleanup(Some(socket));
            
            return Err(Error::from_raw_os_error(error));
            
        }
        
        // ---------- return ----------
        
        let socket = Arc::new(socket);
        
        let listener = Self {
            socket: Arc::downgrade(&socket),
        };
        
        let stopper = ListenerStopper {
            socket: Some(socket),
        };
        
        Ok((listener, stopper))
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn accept(&self) -> Result<TcpStream, Error> {
        let Some(socket) = self.socket.upgrade() else {
            return Err(Error::new(ErrorKind::Other, "The socket has been closed"));
        };
        
        let accept = unsafe {
            
            ffi::accept(
                *socket,
                ptr::null_mut(),
                ptr::null_mut(),
            )
            
        };
        
        if accept == ! 0 as SOCKET {
            
            let error = unsafe {
                
                ffi::WSAGetLastError()
                
            };
            
            return Err(Error::from_raw_os_error(error));
            
        }
        
        let stream = unsafe {
            
            TcpStream::from_raw_socket(accept)
            
        };
        
        Ok(stream)
    }
    
}

impl ListenerStopper {
    
    pub fn stop(&mut self) {
        if let Some(socket) = self.socket.take() {
            close_and_cleanup(Some(*socket));
        }
    }
    
}

impl Drop for ListenerStopper {
    
    fn drop(&mut self) {
        self.stop();
    }
    
}

fn close_and_cleanup(socket: Option<SOCKET>) {
    
    if let Some(socket) = socket {
        
        unsafe {
            
            ffi::closesocket(
                socket,
            )
            
        };
        
    }
    
    unsafe {
        
        ffi::WSACleanup()
        
    };
    
}
