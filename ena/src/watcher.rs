use std::{
    ffi::{ c_void, OsString },
    io::{ Error, ErrorKind },
    os::windows::{
        ffi::{ OsStrExt, OsStringExt },
        io::{ OwnedHandle, HandleOrInvalid, AsRawHandle },
    },
    path::{ Path, PathBuf },
    ptr,
    slice,
    sync::{ Arc, Weak },
    time::Duration,
    thread::{ self, JoinHandle },
};

const EVENT_BUFFER_SIZE: u32 = 24 * 1024;

pub struct FilesWatcher {
    weak_handle: Weak<OwnedHandle>,
    join_handle: Option<JoinHandle<()>>,
}

pub enum FilesWatcherEvent {
    FileAdded(PathBuf),
    FileRemoved(PathBuf),
    Interrupted(Error),
}

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        pub fn CreateFileW(
            lpFileName: *const u16, // LPCWSTR -> *const WCHAR (wchar_t)
            dwDesiredAccess: u32, // DWORD (c_ulong)
            dwShareMode: u32, // DWORD (c_ulong)
            lpSecurityAttributes: *mut c_void, // LPSECURITY_ATTRIBUTES -> *mut SECURITY_ATTRIBUTES
            dwCreationDisposition: u32, // DWORD (c_ulong)
            dwFlagsAndAttributes: u32, // DWORD (c_ulong)
            hTemplateFile: *mut c_void, // HANDLE
        ) -> *mut c_void; // HANDLE
        
        // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw
        pub fn ReadDirectoryChangesW(
            hDirectory: *mut c_void, // HANDLE
            lpBuffer: *mut c_void, // LPVOID
            nBufferLength: u32, // DWORD (c_ulong)
            bWatchSubtree: i32, // BOOL (c_int)
            dwNotifyFilter: u32, // DWORD (c_ulong)
            lpBytesReturned: *mut u32, // LPDWORD -> *mut DWORD (c_ulong)
            lpOverlapped: *mut c_void, // LPOVERLAPPED -> *mut OVERLAPPED
            lpCompletionRoutine: *mut c_void, // LPOVERLAPPED_COMPLETION_ROUTINE
        ) -> i32; // BOOL (c_int)
        
        // https://docs.microsoft.com/en-us/windows/win32/fileio/cancelioex-func
        pub fn CancelIoEx(
            hFile: *mut c_void, // HANDLE
            lpOverlapped: *mut c_void, // LPOVERLAPPED
        ) -> i32; // BOOL (c_int)
        
    }
    
    #[repr(C)]
    #[allow(non_snake_case)]
    // https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-file_notify_information
    pub struct FILE_NOTIFY_INFORMATION {
        pub NextEntryOffset: u32, // DWORD (c_ulong)
        pub Action: u32, // DWORD (c_ulong)
        pub FileNameLength: u32, // DWORD (c_ulong)
        pub FileName: [u16; 1], // WCHAR (wchar_t)
    }
    
}

impl FilesWatcher {
    
    // ---------- constructors ----------
    
    
    pub fn mount<N: Fn(FilesWatcherEvent) + Send + 'static>(root_path: &Path, notify: N) -> Result<Self, Error> {
        let encoded_path: Vec<u16> = root_path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let owned_handle = unsafe {
            
            let result = ffi::CreateFileW(
                encoded_path.as_ptr(),
                1, // FILE_LIST_DIRECTORY
                0x0000_0001 | 0x0000_0002 | 0x0000_0004, // FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE
                ptr::null_mut(),
                3, // OPEN_EXISTING
                0x0200_0000, // FILE_FLAG_BACKUP_SEMANTICS
                ptr::null_mut(),
            );
            
            OwnedHandle::try_from(HandleOrInvalid::from_raw_handle(result))
                .map_err(|_| Error::last_os_error())
            
        }?;
        
        let directory_handle = Arc::new(owned_handle);
        let weak_handle = Arc::downgrade(&directory_handle);
        let join_handle = Some(Self::generate_events(directory_handle, root_path.to_owned(), notify)?);
        
        Ok(Self {
            weak_handle,
            join_handle,
        })
    }
    
    
    // ---------- helpers ----------
    
    
    fn generate_events<N: Fn(FilesWatcherEvent) + Send + 'static>(directory_handle: Arc<OwnedHandle>, root_path: PathBuf, notify: N) -> Result<JoinHandle<()>, Error> {
        thread::Builder::new().spawn(move || {
            
            loop {
                
                let mut buffer = [0u8; EVENT_BUFFER_SIZE as usize];
                let mut bytes = 0u32;
                
                let result = unsafe {
                    
                    ffi::ReadDirectoryChangesW(
                        directory_handle.as_raw_handle(),
                        buffer.as_mut_ptr().cast::<c_void>(),
                        EVENT_BUFFER_SIZE,
                        1,
                        0x0000_0001 | 0x0000_0002, // FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_DIR_NAME
                        ptr::addr_of_mut!(bytes),
                        ptr::null_mut(),
                        ptr::null_mut(),
                    )
                    
                };
                
                if result == 0 {
                    
                    let error = Error::last_os_error();
                    
                    // produced by CancelSynchronousIo
                    if error.kind() != ErrorKind::TimedOut {
                        notify(FilesWatcherEvent::Interrupted(error));
                    }
                    
                    break;
                    
                }
                
                // this would mean that the buffer wasn't big enough to fit every observed event, so data loss is inevitable
                if bytes == 0 {
                    
                    let error = Error::new(ErrorKind::Other, "Could not process all generated events");
                    notify(FilesWatcherEvent::Interrupted(error));
                    
                    break;
                    
                }
                
                let mut current_offset = buffer.as_ptr();
                
                loop {
                    
                    let current_entry = unsafe {
                        
                        &*current_offset.cast::<ffi::FILE_NOTIFY_INFORMATION>()
                        
                    };
                    
                    let length = current_entry.FileNameLength as usize / 2;
                    
                    let filename: &[u16] = unsafe {
                        
                        slice::from_raw_parts(current_entry.FileName.as_ptr(), length)
                        
                    };
                    
                    let path = root_path.join(OsString::from_wide(filename));
                    
                    match current_entry.Action {
                        
                        // FILE_ACTION_ADDED, FILE_ACTION_RENAMED_NEW_NAME
                        0x0000_0001 | 0x0000_0005 => notify(FilesWatcherEvent::FileAdded(path)),
                        
                        // FILE_ACTION_REMOVED, FILE_ACTION_RENAMED_OLD_NAME
                        0x0000_0002 | 0x0000_0004 => notify(FilesWatcherEvent::FileRemoved(path)),
                        
                        _ => unreachable!(),
                        
                    }
                    
                    if current_entry.NextEntryOffset == 0 {
                        break;
                    }
                    
                    current_offset = unsafe {
                        
                        current_offset.offset(current_entry.NextEntryOffset as isize)
                        
                    };
                    
                }
                
            }
            
        })
    }
    
}

impl Drop for FilesWatcher {
    
    fn drop(&mut self) {
        let join_handle = match self.join_handle.take() {
            Some(join_handle) => join_handle,
            None => return,
        };
        
        let directory_handle = match self.weak_handle.upgrade() {
            Some(directory_handle) => directory_handle,
            None => return,
        };
        
        thread::spawn(move || {
            
            while ! join_handle.is_finished() {
                
                unsafe {
                    
                    ffi::CancelIoEx(
                        directory_handle.as_raw_handle(),
                        ptr::null_mut(),
                    )
                    
                };
                
                thread::sleep(Duration::from_millis(0));
                
            }
            
        });
    }
    
}
