use std::{
    ffi::c_void,
    io::{ Error, ErrorKind },
    mem,
    net::TcpStream,
    os::windows::io::FromRawSocket,
    ptr,
    sync::{ Arc, Weak },
};

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsastartup
        pub fn WSAStartup(
            wVersionRequested: u16, // WORD -> c_ushort
            lpWSAData: *mut c_void, // LPWSADATA -> *mut WSADATA
        ) -> i32; // c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsacleanup
        pub fn WSACleanup() -> i32; // c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsastringtoaddressw
        pub fn WSAStringToAddressW(
            AddressString: *const u16, // LPWSTR -> *const WCHAR -> wchar_t
            AddressFamily: i32, // INT -> c_int
            lpProtocolInfo: *mut c_void, // LPWSAPROTOCOL_INFOW -> *mut WSAPROTOCL_INFOW
            lpAddress: *mut SOCKADDR, // LPSOCKADDR
            lpAddressLength: *mut i32, // LPINT *mut c_int
        ) -> i32; // INT -> c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsasocketw
        pub fn WSASocketW(
            af: i32, // c_int
            _type: i32, // c_int
            protocol: i32, // c_int
            lpProtocolInfo: *mut c_void, // LPWSAPROTOCOL_INFOW -> *mut WSAPROTOCOL_INFOW
            g: u32, // GROUP -> c_uint
            dwFlags: u32, // DWORD -> c_ulong
        ) -> usize; // SOCKET -> UINT_PTR
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsagetlasterror
        pub fn WSAGetLastError() -> i32; // c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
        pub fn bind(
            s: usize, // SOCKET -> UINT_PTR
            name: *const SOCKADDR,
            namelen: i32 // c_int
        ) -> i32; // c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-listen
        pub fn listen(
            s: usize, // SOCKET -> UINT_PTR
            backlog: i32, // c_int
        ) -> i32; // c_int
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept
        pub fn accept(
            s: usize, // SOCKET -> UINT_PTR
            addr: *mut SOCKADDR,
            addrlen: *mut i32, // c_int
        ) -> usize; // SOCKET -> UINT_PTR
        
        // https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-closesocket
        pub fn closesocket(
            s: usize, // SOCKET -> UINT_PTR
        ) -> i32; // c_int
        
    }
    
    #[repr(C)]
    // https://learn.microsoft.com/en-us/windows/win32/api/winsock/ns-winsock-sockaddr
    pub struct SOCKADDR {
        pub sa_family: u16, // ADDRESS_FAMILY -> USHORT -> c_ushort
        pub sa_data: [i8; 14], // CHAR -> c_char
    }
    
}

pub struct Listener {
    socket: Weak<usize>,
}

pub struct ListenerStopper {
    socket: Option<Arc<usize>>,
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
        
        let encoded_address: Vec<u16> = bind_address.encode_utf16()
            .chain(Some(0))
            .collect();
        
        let mut address = unsafe {
            
            mem::zeroed::<ffi::SOCKADDR>()
            
        };
        
        let mut address_length = mem::size_of_val(&address) as i32;
        
        let conversion = unsafe {
            
            ffi::WSAStringToAddressW(
                encoded_address.as_ptr(),
                2, // AF_INET
                ptr::null_mut(),
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
        
        if socket == ! 0usize {
            
            close_and_cleanup(None);
            
            return Err(Error::last_os_error());
            
        }
        
        // ---------- bind ----------
        
        let bind = unsafe {
            
            ffi::bind(
                socket,
                &address,
                mem::size_of_val(&address) as i32,
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
        let socket = match self.socket.upgrade() {
            Some(socket) => socket,
            None => return Err(Error::new(ErrorKind::Other, "The socket has been closed")),
        };
        
        let accept = unsafe {
            
            ffi::accept(
                *socket,
                ptr::null_mut(),
                ptr::null_mut(),
            )
            
        };
        
        if accept == ! 0usize {
            
            let error = unsafe {
                
                ffi::WSAGetLastError()
                
            };
            
            return Err(Error::from_raw_os_error(error));
            
        }
        
        let stream = unsafe {
            
            TcpStream::from_raw_socket(accept as u64)
            
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

fn close_and_cleanup(socket: Option<usize>) {
    
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
