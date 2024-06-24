use super::ffi;

#[derive(Clone, Copy, PartialEq)]
pub enum HandleSource {
    Session,
    Connection,
    Request,
}

pub struct Handle {
    inner: ffi::HINTERNET,
    source: HandleSource,
}

impl Handle {
    
    pub fn new(handle: ffi::HINTERNET, source: HandleSource) -> Self {
        Self {
            inner: handle,
            source,
        }
    }
    
    pub fn as_raw(&self) -> ffi::HINTERNET {
        self.inner
    }
    
    pub fn source(&self) -> HandleSource {
        self.source
    }
    
}

impl Drop for Handle {
    
    fn drop(&mut self) {
        ffi::close_handle(self).ok();
    }
    
}
