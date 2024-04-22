use std::{
    error::Error,
    ffi::OsStr,
    io::Write,
    path::MAIN_SEPARATOR_STR,
    process::{ Command, Stdio },
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/files/index.html");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    // -------------------- list --------------------
    
    let files = ena::Files::new(root)?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in files {
        
        // skip entries whose file_name cannot be represented in UTF-8
        let Some(file_name) = entry.file_name().to_str() else {
            continue;
        };
        
        // skip entries whose container cannot be represented in UTF-8
        let Some(container) = entry.container(root).to_str() else {
            continue;
        };
        
        write!(&mut response, "<a data-value='{}'>", u8::from(! entry.is_marked(flag)))?;
        
        if ! container.is_empty() {
            response.write_all(b"<span>")?;
            
            chikuwa::HtmlEscaper::from(container.as_bytes())
                .try_for_each(|escaped| response.write_all(escaped))?;
            
            response.write_all(MAIN_SEPARATOR_STR.as_bytes())?;
            
            response.write_all(b"</span>")?;
        }
        
        chikuwa::HtmlEscaper::from(file_name.as_bytes())
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    let player = rin::get(b"player")?;
    
    // -------------------- files --------------------
    
    let mut files = ena::Files::new(root)?
        .filter(|file| is_file_selected(request, root, file))
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    Command::new(player)
        .args(files)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

pub fn mark(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    // -------------------- files --------------------
    
    let mut files = ena::Files::new(root)?
        .filter(|file| is_file_selected(request, root, file))
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    files.try_for_each(|file| file.toggle_mark(flag))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

pub fn folder(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    
    // -------------------- files --------------------
    
    let mut files = ena::Files::new(root)?
        .filter(|file| is_file_selected(request, root, file))
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- folder --------------------
    
    let folder = match request.param(b"input").next() {
        Some(input) => str::from_utf8(input).map_err(|_| "Invalid folder")?,
        None => "",
    };
    
    // -------------------- operation --------------------
    
    files.try_for_each(|file| file.move_to_folder(root, folder))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    
    // -------------------- files --------------------
    
    let mut files = ena::Files::new(root)?
        .filter(|file| is_file_selected(request, root, file))
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    files.try_for_each(ena::FilesEntry::delete)?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn is_file_selected(request: &Request, root: &str, file: &ena::FilesEntry) -> bool {
    request.param(b"tag")
        .map(|tag| OsStr::new(str::from_utf8(tag).unwrap_or("")))
        .any(|tag| file.relative(root) == tag)
}
