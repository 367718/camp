use std::{
    error::Error,
    io::Write,
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/rules/index.html");

pub enum RulesEndpoint {
    Index,
    Entries,
    Insert,
    Update,
    Delete,
}

impl RulesEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/rules") => Some(Self::Index),
            (b"GET", b"/rules/entries") => Some(Self::Entries),
            (b"POST", b"/rules/insert") => Some(Self::Insert),
            (b"POST", b"/rules/update") => Some(Self::Update),
            (b"POST", b"/rules/delete") => Some(Self::Delete),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Entries => entries(&mut request),
            Self::Insert => insert(&mut request),
            Self::Update => update(&mut request),
            Self::Delete => delete(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- list --------------------
    
    let rules = chiaki::List::load("rules")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &rules {
        
        write!(&mut response, "<a data-value='{}'>", entry.value)?;
        
        chikuwa::HtmlEscaper::from(entry.tag)
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- matcher --------------------
    
    let matcher = request.param(b"input")
        .next()
        .ok_or("Matcher not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("rules")
        .and_then(|mut list| list.insert(matcher, 0))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- matcher and progress --------------------
    
    let matcher = request.param(b"tag")
        .next()
        .ok_or("Matcher not provided")?;
    
    let progress = request.param(b"input")
        .next()
        .and_then(|progress| str::from_utf8(progress).ok())
        .and_then(|progress| progress.parse().ok())
        .ok_or("Progress not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("rules")
        .and_then(|mut list| list.update(matcher, progress))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- matcher --------------------
    
    let matcher = request.param(b"tag")
        .next()
        .ok_or("Matcher not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("rules")
        .and_then(|mut list| list.delete(matcher))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}
