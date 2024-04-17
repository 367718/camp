use std::{
    error::Error,
    io::Write,
    path::Path,
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
    
    let files = ena::Files::new(Path::new(root))?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in files {
        
        write!(&mut response, "<a data-value='{}'>", u8::from(! entry.is_marked(flag)))?;
        
        let (filename, container) = entry.components(root);
        
        if let Some(container) = container {
            response.write_all(b"<span>")?;
            
            chikuwa::HtmlEscaper::from(container.as_bytes())
                .try_for_each(|escaped| response.write_all(escaped))?;
            
            response.write_all(b"</span>")?;
        }
        
        chikuwa::HtmlEscaper::from(filename.as_bytes())
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
    
    let mut files = ena::Files::new(Path::new(root))?
        .filter(|file| request.param(b"tag").any(|tag| file.relative(root).as_bytes() == tag))
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
    
    let mut files = ena::Files::new(Path::new(root))?
        .filter(|file| request.param(b"tag").any(|tag| file.relative(root).as_bytes() == tag))
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
    
    let mut files = ena::Files::new(Path::new(root))?
        .filter(|file| request.param(b"tag").any(|tag| file.relative(root).as_bytes() == tag))
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- foldername --------------------
    
    let foldername = match request.param(b"input").next() {
        Some(input) => str::from_utf8(input).map_err(|_| "Invalid foldername")?,
        None => "",
    };
    
    // -------------------- operation --------------------
    
    files.try_for_each(|file| file.move_to_folder(root, foldername))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    let root = rin::get(b"root")?;
    
    // -------------------- files --------------------
    
    let mut files = ena::Files::new(Path::new(root))?
        .filter(|file| request.param(b"tag").any(|tag| file.relative(root).as_bytes() == tag))
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
