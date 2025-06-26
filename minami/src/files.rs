use std::{
    error::Error,
    ffi::OsStr,
    io::Write,
    path::MAIN_SEPARATOR_STR,
    process::{ Command, Stdio },
};

use ayano::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/files/index.html");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    // -------------------- operation --------------------
    
    let list = ena::Files::walk(root)?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in list {
        
        // skip entries whose file_name cannot be represented in UTF-8
        let Some(file_name) = entry.file_name().to_str() else {
            continue;
        };
        
        // skip entries whose container cannot be represented in UTF-8
        let Some(container) = entry.container().to_str() else {
            continue;
        };
        
        write!(&mut response, "<a data-value='{}'>", u8::from(! entry.is_marked(flag).unwrap_or(false)))?;
        
        if ! container.is_empty() {
            response.write_all(b"<span>")?;
            
            chikuwa::escape_html(container.as_bytes())
                .try_for_each(|escaped| response.write_all(escaped))?;
            
            response.write_all(MAIN_SEPARATOR_STR.as_bytes())?;
            
            response.write_all(b"</span>")?;
        }
        
        chikuwa::escape_html(file_name.as_bytes())
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get(b"root")?;
    let player = rin::get(b"player")?;
    
    let form_data = request.form_data()
        .ok_or("Could not extract form data")?;
    
    let matchers = form_data.get(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).map(OsStr::new).ok())
        .collect::<Vec<&OsStr>>();
    
    let mut selected = ena::Files::walk(root)?
        .filter(|entry| matchers.iter().any(|matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("No relevant file found".into());
    }
    
    // -------------------- operation --------------------
    
    Command::new(player)
        .args(selected)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn mark(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    let form_data = request.form_data()
        .ok_or("Could not extract form data")?;
    
    let matchers = form_data.get(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).map(OsStr::new).ok())
        .collect::<Vec<&OsStr>>();
    
    let mut selected = ena::Files::walk(root)?
        .filter(|entry| matchers.iter().any(|matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("No relevant file found".into());
    }
    
    // -------------------- operation --------------------
    
    selected.try_for_each(|entry| entry.toggle_mark(flag))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn folder(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get(b"root")?;
    
    let form_data = request.form_data()
        .ok_or("Could not extract form data")?;
    
    let matchers = form_data.get(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).map(OsStr::new).ok())
        .collect::<Vec<&OsStr>>();
    
    let mut selected = ena::Files::walk(root)?
        .filter(|entry| matchers.iter().any(|matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("No relevant file found".into());
    }
    
    let folder = match form_data.get(b"input").next() {
        Some(input) => str::from_utf8(input).map_err(|_| "Invalid input")?,
        None => "",
    };
    
    // -------------------- operation --------------------
    
    selected.try_for_each(|entry| entry.move_to_folder(folder))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get(b"root")?;
    
    let form_data = request.form_data()
        .ok_or("Could not extract form data")?;
    
    let matchers = form_data.get(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).map(OsStr::new).ok())
        .collect::<Vec<&OsStr>>();
    
    let mut selected = ena::Files::walk(root)?
        .filter(|entry| matchers.iter().any(|matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("No relevant file found".into());
    }
    
    // -------------------- operation --------------------
    
    selected.try_for_each(ena::FilesEntry::delete)?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
