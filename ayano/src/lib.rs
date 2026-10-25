#[link(name = "httpapi")]
unsafe extern "system" {}

mod sys;
mod server;
mod form_field;
mod request;
mod response;

use std::os::raw::*;

pub use server::Server;
pub use form_field::FormField;
pub use request::Request;
pub use response::Response;

pub enum StatusCode {
    Ok,
    Error,
    BadRequest,
    NotFound,
    ContentTooLarge,
    RequestHeaderFieldsTooLarge,
    VersionNotSupported,
}

pub enum ContentType {
    Plain,
    Html,
    Css,
    Javascript,
}

pub enum CacheControl {
    Static,
    Dynamic,
}

impl StatusCode {
    
    const fn into_int(self) -> c_ushort {
        match self {
            Self::Ok => 200,
            Self::Error => 500,
            Self::BadRequest => 400,
            Self::NotFound => 404,
            Self::ContentTooLarge => 413,
            Self::RequestHeaderFieldsTooLarge => 431,
            Self::VersionNotSupported => 505,
        }
    }
    
}

impl ContentType {
    
    const fn into_header(self) -> &'static [u8] {
        match self {
            Self::Plain => b"text/plain; charset=utf-8",
            Self::Html => b"text/html; charset=utf-8",
            Self::Css => b"text/css; charset=utf-8",
            Self::Javascript => b"text/javascript; charset=utf-8",
        }
    }
    
}

impl CacheControl {
    
    const fn into_header(self) -> &'static [u8] {
        match self {
            Self::Static => b"max-age=15552000, immutable",
            Self::Dynamic => b"no-cache, no-store",
        }
    }
    
}
