use std::{
    io::{Error, ErrorKind},
    mem,
    os::raw::*,
    slice,
};

use crate::{ last_winhttp_error, sys };

pub(crate) struct Url {
    pub(crate) host: Vec<u16>,
    pub(crate) port: u16,
    pub(crate) path: Vec<u16>,
}

const SLASH: u16 = 47; // /
const QUESTION_MARK: u16 = 63; // ?
const HASH: u16 = 35; // #

impl TryFrom<&str> for Url {
    
    type Error = Error;
    
    fn try_from(url: &str) -> Result<Self, Self::Error> {
        // -------------------- components --------------------
        
        let url_wide = chikuwa::win_string(url)?;
        
        // SAFETY: all-zero bit patterns are valid for structs that consist entirely of integer primitives and raw pointers
        let mut components: sys::UrlComponents = unsafe { mem::zeroed() };
        
        // size of `UrlComponents` in bytes, as required by the WinHTTP API
        components.dw_struct_size = c_ulong::try_from(size_of_val(&components)).expect("Unexpected truncation");
        
        // pointer mode: WinHTTP sets pointers into `url_wide` instead of performing memory allocations or string copies
        components.dw_host_name_length = 1; // populate `lpsz_host_name` and `dw_host_name_length`
        components.dw_url_path_length = 1; // populate `lpsz_url_path` and `dw_url_path_length`
        components.dw_extra_info_length = 1; // populate `lpsz_extra_info` and `dw_extra_info_length`
        
        // SAFETY:
        // 1. `url_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `dw_url_length` as `0` is valid and signals that `pwsz_url` is null-terminated
        // 3. `dw_flags` as `0` is valid and signals that no special flags are to be used to control the operation
        // 4. `&raw mut components` points to a live, properly aligned `UrlComponents` struct
        let result = unsafe {
            
            sys::WinHttpCrackUrl(
                url_wide.as_ptr(),
                0, // no explicit length
                0, // no flag
                &raw mut components,
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- scheme --------------------
        
        // only HTTPS connections are supported
        if components.n_scheme != sys::INTERNET_SCHEME_HTTPS {
            return Err(Error::new(ErrorKind::InvalidInput, "Unsupported URL scheme"));
        }
        
        // -------------------- host --------------------
        
        if components.lpsz_host_name.is_null() || components.dw_host_name_length == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Host cannot be empty"));
        }
        
        // SAFETY:
        // 1. `components.lpsz_host_name` was verified to be non-null and `dw_host_name_length` is greater than 0
        // 2. `lpsz_host_name` is guaranteed to point to a valid, contiguous slice of `dw_host_name_length` UTF-16 code units located within `url_wide`
        let host_raw = unsafe {
            
            slice::from_raw_parts(
                components.lpsz_host_name,
                components.dw_host_name_length as usize,
            )
            
        };
        
        let mut host = Vec::with_capacity(host_raw.len() + 1);
        host.extend_from_slice(host_raw);
        host.push(0);
        
        // -------------------- port --------------------
        
        // will be set to HTTPS default port (443) if not specified
        let port = components.n_port;
        
        // -------------------- path --------------------
        
        let path_raw = if ! components.lpsz_url_path.is_null() && components.dw_url_path_length > 0 {
            
            // SAFETY:
            // 1. `components.lpsz_url_path` was verified to be non-null and `dw_url_path_length` is greater than 0
            // 2. `lpsz_url_path` is guaranteed to point to a valid, contiguous slice of `dw_url_path_length` UTF-16 code units located within `url_wide`
            unsafe {
                
                slice::from_raw_parts(
                    components.lpsz_url_path,
                    components.dw_url_path_length as usize,
                )
                
            }
            
        } else {
            
            &[]
            
        };
        
        let extra_raw = if ! components.lpsz_extra_info.is_null() && components.dw_extra_info_length > 0 {
            
            // SAFETY:
            // 1. `components.lpsz_extra_info` was verified to be non-null and `dw_extra_info_length` is greater than 0
            // 2. `lpsz_extra_info` is guaranteed to point to a valid, contiguous slice of `dw_extra_info_length` UTF-16 code units located within `url_wide`
            unsafe {
                
                slice::from_raw_parts(
                    components.lpsz_extra_info,
                    components.dw_extra_info_length as usize,
                )
                
            }
            
        } else {
            
            &[]
            
        };
        
        // retain "?query", strip "#fragment" (which must never be sent to the server)
        // "#fragment" either exists by itself or comes after "?query" (if "?query" is present)
        let extra_without_fragment = if extra_raw.starts_with(&[QUESTION_MARK]) {
            
            let end = extra_raw
                .iter()
                .position(|&code_unit| code_unit == HASH)
                .unwrap_or(extra_raw.len());
            
            &extra_raw[..end]
            
        } else {
            
            &[]
            
        };
        
        // strip initial "/slash", if present, to simplify path construction
        let path_without_slash = path_raw
            .strip_prefix(&[SLASH])
            .unwrap_or(path_raw);
        
        let mut path = Vec::with_capacity(path_without_slash.len() + extra_without_fragment.len() + 2);
        path.push(SLASH);
        path.extend_from_slice(path_without_slash);
        path.extend_from_slice(extra_without_fragment);
        path.push(0);
        
        // -------------------- response --------------------
        
        Ok(Self { host, port, path })
    }
    
}

#[cfg(test)]
mod tests {
    
    // valid
    // ip
    // ip_with_port
    // with_querystring
    // with_querystring_and_fragment
    // with_fragment
    // with_port
    // no_host
    // invalid_scheme
    // uppercase_scheme
    // invalid_port
    // empty_port
    // no_slash
    // no_slash_with_querystring
    // no_slash_with_port_and_fragment
    
    use super::*;
    
    #[test]
    fn valid() {
        // setup
        
        let resource = "https://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn ip() {
        // setup
        
        let resource = "https://192.168.150.10/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("192.168.150.10").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn ip_with_port() {
        // setup
        
        let resource = "https://192.168.150.10:7777";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("192.168.150.10").unwrap());
        assert_eq!(output.port, 7777);
        assert_eq!(output.path, chikuwa::win_string("/").unwrap());
    }
    
    #[test]
    fn with_querystring() {
        // setup
        
        let resource = "https://example.com/test?placeholder=no&madeup=yes";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test?placeholder=no&madeup=yes").unwrap());
    }
    
    #[test]
    fn with_querystring_and_fragment() {
        // setup
        
        let resource = "https://example.com/test?placeholder=no&madeup=yes#fragment";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test?placeholder=no&madeup=yes").unwrap());
    }
    
    #[test]
    fn with_fragment() {
        // setup
        
        let resource = "https://example.com/test#fragment";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn with_port() {
        // setup
        
        let resource = "https://example.com:8080/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 8080);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn no_host() {
        // setup
        
        let resource = "https://";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn invalid_scheme() {
        // setup
        
        let resource = "http://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn uppercase_scheme() {
        // setup
        
        let resource = "HTTPS://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn invalid_port() {
        // setup
        
        let resource = "https://example.com:port/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn empty_port() {
        // setup
        
        let resource = "https://example.com:/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/test").unwrap());
    }
    
    #[test]
    fn no_slash() {
        // setup
        
        let resource = "https://example.com";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/").unwrap());
    }
    
    #[test]
    fn no_slash_with_querystring() {
        // setup
        
        let resource = "https://example.com?placeholder=no&madeup=yes";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/?placeholder=no&madeup=yes").unwrap());
    }
    
    #[test]
    fn no_slash_with_port_and_fragment() {
        // setup
        
        let resource = "https://example.com:443#frag";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, chikuwa::win_string("example.com").unwrap());
        assert_eq!(output.port, 443);
        assert_eq!(output.path, chikuwa::win_string("/").unwrap());
    }
    
}
