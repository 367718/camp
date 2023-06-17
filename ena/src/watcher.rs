use std::{
    ffi::OsString,
    io::{ Error, ErrorKind },
    os::{
        raw::*,
        windows::{
            ffi::OsStringExt,
            io::{ OwnedHandle, HandleOrInvalid, AsRawHandle },
            raw::HANDLE,
        },
    },
    path::{ Path, PathBuf },
    ptr,
    slice,
    sync::{ Arc, Weak },
    time::Duration,
    thread::{ self, JoinHandle },
};

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        pub fn CreateFileW(
            lpfilename: *const c_ushort,
            dwdesiredaccess: c_ulong,
            dwsharemode: c_ulong,
            lpsecurityattributes: *const c_void, // SECURITY_ATTRIBUTES
            dwcreationdisposition: c_ulong,
            dwflagsandattributes: c_ulong,
            htemplatefile: HANDLE,
        ) -> HANDLE;
        
        // https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw
        pub fn ReadDirectoryChangesW(
            hdirectory: HANDLE,
            lpbuffer: *mut c_void,
            nbufferlength: c_ulong,
            bwatchsubtree: c_int,
            dwnotifyfilter: c_ulong,
            lpbytesreturned: *mut c_ulong,
            lpoverlapped: *mut c_void, // OVERLAPPED
            lpcompletionroutine: *mut c_void, // LPOVERLAPPED_COMPLETION_ROUTINE
        ) -> c_int;
        
        // https://docs.microsoft.com/en-us/windows/win32/fileio/cancelioex-func
        pub fn CancelIoEx(
            hfile: HANDLE,
            lpoverlapped: *const c_void, // OVERLAPPED
        ) -> c_int;
        
    }
    
    #[repr(C)]
    #[allow(non_snake_case)]
    // https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-file_notify_information
    pub struct FILE_NOTIFY_INFORMATION {
        pub NextEntryOffset: c_ulong,
        pub Action: c_ulong,
        pub FileNameLength: c_ulong,
        pub FileName: [c_ushort; 1],
    }
    
}

const EVENT_BUFFER_SIZE: c_ulong = 24 * 1024;

pub struct FilesWatcher {
    weak_handle: Weak<OwnedHandle>,
    join_handle: Option<JoinHandle<()>>,
}

pub enum FilesWatcherEvent {
    FileAdded(PathBuf),
    FileRemoved(PathBuf),
    Interrupted(Error),
}

impl FilesWatcher {
    
    // ---------- constructors ----------
    
    
    pub fn mount<N: Fn(FilesWatcherEvent) + Send + 'static>(root_path: &Path, notify: N) -> Result<Self, Error> {
        let owned_handle = unsafe {
            
            let result = ffi::CreateFileW(
                chikuwa::WinString::from(root_path).as_ptr(),
                1, // FILE_LIST_DIRECTORY
                0x0000_0001 | 0x0000_0002 | 0x0000_0004, // FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE
                ptr::null(),
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
                
                let mut buffer = [0 as c_char; EVENT_BUFFER_SIZE as usize];
                let mut bytes = 0 as c_ulong;
                
                let result = unsafe {
                    
                    ffi::ReadDirectoryChangesW(
                        directory_handle.as_raw_handle(),
                        buffer.as_mut_ptr().cast::<c_void>(),
                        EVENT_BUFFER_SIZE,
                        1,
                        0x0000_0001 | 0x0000_0002, // FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_DIR_NAME
                        &mut bytes,
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
                    
                    let filename: &[c_ushort] = unsafe {
                        
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
                        
                        current_offset.offset(isize::try_from(current_entry.NextEntryOffset).unwrap())
                        
                    };
                    
                }
                
            }
            
        })
    }
    
}

impl Drop for FilesWatcher {
    
    fn drop(&mut self) {
        let Some(join_handle) = self.join_handle.take() else {
            return;
        };
        
        let Some(directory_handle) = self.weak_handle.upgrade() else {
            return;
        };
        
        thread::spawn(move || {
            
            while ! join_handle.is_finished() {
                
                unsafe {
                    
                    ffi::CancelIoEx(
                        directory_handle.as_raw_handle(),
                        ptr::null(),
                    )
                    
                };
                
                thread::sleep(Duration::from_millis(0));
                
            }
            
        });
    }
    
}
