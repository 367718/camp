use std::{
    error::Error,
    ffi::OsString,
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
    
    let root = rin::get::<&str>(b"root")?;
    let flag = rin::get::<&str>(b"flag")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let filter = request.query_string(b"filter")
        .next()
        .unwrap_or_default();
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &list {
        
        // skip entries whose relative path cannot be represented in UTF-8
        let Some(relative) = entry.relative().to_str() else {
            continue;
        };
        
        if ! filter.is_empty() && chikuwa::subslice_index(relative.as_bytes(), &filter).is_none() {
            continue;
        }
        
        let file_name = entry.file_name().to_str().unwrap();
        let container = entry.container().to_str().unwrap();
        
        // false => 0
        //  true => 1
        // if marked, flip boolean to use 0 as true
        let value = u8::from(! entry.is_marked(flag).unwrap_or(false));
        
        write!(&mut response, "<a data-value='{}'>", value)?;
        
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

pub fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let player = rin::get::<&str>(b"player")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_params(b"matcher")
        .filter_map(|matcher| String::from_utf8(matcher).ok())
        .map(OsString::from)
        .collect::<Vec<OsString>>();
    
    let mut selected = list.into_iter()
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
    
    let root = rin::get::<&str>(b"root")?;
    let flag = rin::get::<&str>(b"flag")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_params(b"matcher")
        .filter_map(|matcher| String::from_utf8(matcher).ok())
        .map(OsString::from)
        .collect::<Vec<OsString>>();
    
    let mut selected = list.into_iter()
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
    
    let root = rin::get::<&str>(b"root")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_params(b"matcher")
        .filter_map(|matcher| String::from_utf8(matcher).ok())
        .map(OsString::from)
        .collect::<Vec<OsString>>();
    
    let mut selected = list.into_iter()
        .filter(|entry| matchers.iter().any(|matcher| matcher == entry.relative()))
        .peekable();
    
    if selected.peek().is_none() {
        return Err("No relevant file found".into());
    }
    
    let folder = match request.form_params(b"input").next() {
        Some(input) => String::from_utf8(input).map_err(|_| "Invalid input")?,
        None => String::new(),
    };
    
    // -------------------- operation --------------------
    
    selected.try_for_each(|entry| entry.move_to_folder(&folder))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let root = rin::get::<&str>(b"root")?;
    let max_directory_depth = rin::get::<u64>(b"max_directory_depth")?;
    
    let list = ena::Files::new(root, max_directory_depth);
    
    let matchers = request.form_params(b"matcher")
        .filter_map(|matcher| String::from_utf8(matcher).ok())
        .map(OsString::from)
        .collect::<Vec<OsString>>();
    
    let mut selected = list.into_iter()
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
