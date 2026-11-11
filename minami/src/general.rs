use std::{
    error::Error,
    io::Write,
};

use ayano::{ Request, ContentType, CacheControl };

const STYLES: &[u8] = include_bytes!("../rsc/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/scripts.js");

pub fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(200, ContentType::Css, CacheControl::Static)
        .and_then(|mut response| response.write_all(STYLES))?;
    
    Ok(())
}

pub fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(200, ContentType::Javascript, CacheControl::Static)
        .and_then(|mut response| response.write_all(SCRIPTS))?;
    
    Ok(())
}
