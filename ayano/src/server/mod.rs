mod api;
mod session;
mod queue;
mod url_group;

use std::{
    io,
    os::raw::*,
};

use api::Api;
use session::Session;
use queue::Queue;
use url_group::UrlGroup;

use crate::Request;

// drop order required by http.sys, do not rearrange
pub struct Server {
    #[allow(dead_code)]
    url_group: UrlGroup,
    queue: Queue,
    #[allow(dead_code)]
    session: Session,
    #[allow(dead_code)]
    api: Api,
}

impl Server {
    
    pub fn bind(url: &str) -> io::Result<Self> {
        let api = Api::init()?;
        let session = Session::new()?;
        let queue = Queue::new()?;
        let url_group = UrlGroup::new(session.id, queue.handle, url)?;
        
        Ok(Self { url_group, queue, session, api })
    }
    
    pub fn accept(&self) -> io::Result<Request<'_>> {
        Request::new(self)
    }
    
    pub(crate) fn queue_handle(&self) -> *mut c_void {
        self.queue.handle
    }
    
}
