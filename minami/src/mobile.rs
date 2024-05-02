use std::{
    error::Error,
    io::Write,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/mobile/index.html");
const STYLES: &[u8] = include_bytes!("../rsc/mobile/styles.css");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
        .and_then(|mut response| response.write_all(STYLES))?;
    
    Ok(())
}
