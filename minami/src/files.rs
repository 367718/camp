use core::fmt::NumBuffer;

use std::{
    error::Error,
    io::Write,
    path::MAIN_SEPARATOR_STR,
    process::{ Command, Stdio },
};

use ayano::{ ServerRequest, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/files.html");

pub fn index(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let flag = rin::get::<&str>(b"flag")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    let mut numbuf = NumBuffer::new();
    
    for entry in &list {
        
        // skip entries whose relative path cannot be represented in utf8
        let Some(relative) = entry.relative().to_str() else {
            continue;
        };
        
        let (container, file_name) = relative.rsplit_once(MAIN_SEPARATOR_STR)
            .unwrap_or(("", relative));
        
        response.write_all(b"<a data-value='")?;
        
        // false => 0
        //  true => 1
        // if marked, flip boolean to use true as 0
        let value = u8::from(! entry.is_marked(flag).unwrap_or(false));
        
        response.write_all(value.format_into(&mut numbuf).as_bytes())?;
        response.write_all(b"'>")?;
        
        if ! container.is_empty() {
            response.write_all(b"<span>")?;
            chikuwa::escape_html(container.as_bytes(), &mut response)?;
            response.write_all(MAIN_SEPARATOR_STR.as_bytes())?;
            response.write_all(b"</span>")?;
        }
        
        chikuwa::escape_html(file_name.as_bytes(), &mut response)?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn play(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let player = rin::get::<&str>(b"player")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_data(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).ok())
        .collect::<Vec<&str>>();
    
    let mut selected = list.into_iter()
        .filter(|entry| matchers.iter().any(|&matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("File not found".into());
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

pub fn mark(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let flag = rin::get::<&str>(b"flag")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_data(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).ok())
        .collect::<Vec<&str>>();
    
    let mut selected = list.into_iter()
        .filter(|entry| matchers.iter().any(|&matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("File not found".into());
    }
    
    // -------------------- operation --------------------
    
    selected.try_for_each(|entry| entry.toggle_mark(flag))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn folder(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_data(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).ok())
        .collect::<Vec<&str>>();
    
    let mut selected = list.into_iter()
        .filter(|entry| matchers.iter().any(|&matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("File not found".into());
    }
    
    let folder = match request.form_data(b"input").next() {
        Some(input) => str::from_utf8(input).map_err(|_| "Invalid input")?,
        None => "",
    };
    
    // -------------------- operation --------------------
    
    selected.try_for_each(|entry| entry.move_to_folder(folder))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut ServerRequest) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_data(b"matcher")
        .filter_map(|matcher| str::from_utf8(matcher).ok())
        .collect::<Vec<&str>>();
    
    let mut selected = list.into_iter()
        .filter(|entry| matchers.iter().any(|&matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("File not found".into());
    }
    
    // -------------------- operation --------------------
    
    selected.try_for_each(ena::FilesEntry::delete)?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
