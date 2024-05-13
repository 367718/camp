use std::{
    error::Error,
    io::Write,
};

use ayano::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/general/index.html");
const FAVICON: &[u8] = include_bytes!("../rsc/general/favicon.ico");
const STYLES: &[u8] = include_bytes!("../rsc/general/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/general/scripts.js");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn favicon(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Icon, CacheControl::Static)
        .and_then(|mut response| response.write_all(FAVICON))?;
    
    Ok(())
}

pub fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
        .and_then(|mut response| response.write_all(STYLES))?;
    
    Ok(())
}

pub fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Javascript, CacheControl::Static)
        .and_then(|mut response| response.write_all(SCRIPTS))?;
    
    Ok(())
}
