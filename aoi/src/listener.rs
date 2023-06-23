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
    
    pub const WS_VERSION: c_ushort = 0x202; // 2.2
    
    pub const AF_INET: c_int = 2;
    
    pub const INVALID_SOCKET: SOCKET = SOCKET::MAX;
    pub const SOCKET_ERROR: c_int = -1;
    
    pub const SOCK_STREAM: c_int = 1;
    pub const IPPROTO_TCP: c_int = 6;
    pub const SG_UNCONSTRAINED_GROUP: c_uint = 0x01;
    pub const WSA_FLAG_OVERLAPPED: c_ulong = 0x01;
    pub const WSA_FLAG_NO_HANDLE_INHERIT: c_ulong = 0x80;
    
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
        
        unsafe {
            
            let mut imp_details = mem::zeroed();
            
            let result = ffi::WSAStartup(
                ffi::WS_VERSION,
                &mut imp_details,
            );
            
            if result != 0 {
                return Err(Error::from_raw_os_error(result));
            }
            
        }
        
        // ---------- address ----------
        
        let address = unsafe {
            
            // value stored to prevent crash in optimized build
            let encoded_address = chikuwa::WinString::from(bind_address);
            
            let mut store = mem::zeroed::<ffi::Sockaddr>();
            let mut store_length = mem::size_of_val(&store) as c_int;
            
            let result = ffi::WSAStringToAddressW(
                encoded_address.as_ptr(),
                ffi::AF_INET,
                ptr::null(),
                &mut store,
                &mut store_length,
            );
            
            if result == ffi::SOCKET_ERROR {
                close_and_cleanup(None);
                return Err(Error::from_raw_os_error(ffi::WSAGetLastError()));
            }
            
            store
            
        };
        
        // ---------- socket ----------
        
        let socket = unsafe {
            
            let result = ffi::WSASocketW(
                ffi::AF_INET,
                ffi::SOCK_STREAM,
                ffi::IPPROTO_TCP,
                ptr::null_mut(),
                ffi::SG_UNCONSTRAINED_GROUP,
                ffi::WSA_FLAG_OVERLAPPED | ffi::WSA_FLAG_NO_HANDLE_INHERIT,
            );
            
            if result == ffi::INVALID_SOCKET {
                close_and_cleanup(None);
                return Err(Error::from_raw_os_error(ffi::WSAGetLastError()));
            }
            
            result
            
        };
        
        // ---------- bind ----------
        
        unsafe {
            
            let result = ffi::bind(
                socket,
                &address,
                mem::size_of_val(&address) as c_int,
            );
            
            if result == ffi::SOCKET_ERROR {
                close_and_cleanup(Some(socket));
                return Err(Error::from_raw_os_error(ffi::WSAGetLastError()));
            }
            
        };
        
        // ---------- listen ----------
        
        unsafe {
            
            let result = ffi::listen(
                socket,
                128,
            );
            
            if result == ffi::SOCKET_ERROR {
                close_and_cleanup(Some(socket));
                return Err(Error::from_raw_os_error(ffi::WSAGetLastError()));
            }
            
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
        
        let stream = unsafe {
            
            let result = ffi::accept(
                *socket,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            if result == ffi::INVALID_SOCKET {
                return Err(Error::from_raw_os_error(ffi::WSAGetLastError()));
            }
            
            TcpStream::from_raw_socket(result)
            
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
