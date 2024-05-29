use super::ffi;

pub struct Handle {
    inner: ffi::HINTERNET,
}

impl Handle {
    
    pub fn new(handle: ffi::HINTERNET) -> Self {
        Self {
            inner: handle,
        }
    }
    
    pub fn as_raw(&self) -> ffi::HINTERNET {
        self.inner
    }
    
}

impl Drop for Handle {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.inner);
            
        }
    }
    
}
