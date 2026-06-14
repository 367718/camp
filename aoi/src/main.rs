use std::{
    error::Error,
    fs::File,
    io::{ self, Read, Write },
    os::raw::*,
};

use ayano::{
    Server, Request,
    StatusCode, ContentType, CacheControl,
};

unsafe extern "system" {
    
    // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
    fn WaitNamedPipeW(
        lpNamedPipeName: *const c_ushort,
        nTimeOut: c_ulong,
    ) -> c_int;
    
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const INDEX: &[u8] = include_bytes!("../rsc/index.html");
const STYLES: &[u8] = include_bytes!("../rsc/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/scripts.js");

const COMMAND_PLAY: &[u8] = b"cycle pause\n";
const COMMAND_MINUSCHAPTER: &[u8] = b"cycle chapter down\n";
const COMMAND_PLUSCHAPTER: &[u8] = b"cycle chapter up\n";
const COMMAND_MINUSPLAYLIST: &[u8] = b"playlist-prev\n";
const COMMAND_PLUSPLAYLIST: &[u8] = b"playlist-next\n";
const COMMAND_MINUS5: &[u8] = b"seek -5\n";
const COMMAND_PLUS5: &[u8] = b"seek 5\n";
const COMMAND_MINUS75: &[u8] = b"seek -75\n";
const COMMAND_PLUS75: &[u8] = b"seek 75\n";
const COMMAND_FULLSCREEN: &[u8] = b"cycle fullscreen\n";
const COMMAND_SUBTITLES: &[u8] = b"cycle sub\n";
const COMMAND_TITLE: &[u8] = b"show-text ${media-title} 5000\n";
const COMMAND_TIME: &[u8] = b"show-text \"${playback-time} (${time-remaining})\" 5000\n";
const COMMAND_QUIT: &[u8] = b"quit\n";

const PIPE_MAX_WAIT_AS_MILLIS: c_ulong = 5_000;

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    io::stdout().flush().ok();
    
    let _ = io::stdin().read(&mut [0]).ok();
}

fn process() -> Result<(), Box<dyn Error>> {
    let address = rin::get::<&str>(b"address")?;
    let mut server = Server::bind(address, 10, 10)?;
    
    println!();
    println!("Listening on http://{}", address);
    
    loop {
        
        let Ok(mut request) = server.accept() else {
            continue;
        };
        
        if let Err(error) = handle_request(&mut request) {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
        
    }
}

fn handle_request(request: &mut Request) -> Result<(), Box<dyn Error>> {
    let endpoint = request.endpoint()
        .ok_or("Invalid request")?;
    
    // -------------------- index --------------------
    
    if endpoint == b"GET /" {
        
        request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
            .and_then(|mut response| response.write_all(INDEX))?;
        
        return Ok(());
        
    }
    
    // -------------------- styles --------------------
    
    if endpoint == b"GET /styles.css" {
        
        request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
            .and_then(|mut response| response.write_all(STYLES))?;
        
        return Ok(());
        
    }
    
    // -------------------- scripts --------------------
    
    if endpoint == b"GET /scripts.js" {
        
        request.start_response(StatusCode::Ok, ContentType::Javascript, CacheControl::Static)
            .and_then(|mut response| response.write_all(SCRIPTS))?;
        
        return Ok(());
        
    }
    
    // -------------------- command --------------------
    
    if let Some(command) = get_command(endpoint) {
        
        let pipe = rin::get::<&str>(b"pipe")?;
        write_to_named_pipe(pipe, command)?;
        request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
        
        return Ok(());
        
    }
    
    // -------------------- not found --------------------
    
    Err("Endpoint not found".into())
}

fn get_command(endpoint: &[u8]) -> Option<&'static [u8]> {
    let result: &[u8] = match endpoint {
        b"POST /play" => COMMAND_PLAY,
        b"POST /minuschapter" => COMMAND_MINUSCHAPTER,
        b"POST /pluschapter" => COMMAND_PLUSCHAPTER,
        b"POST /minusplaylist" => COMMAND_MINUSPLAYLIST,
        b"POST /plusplaylist" => COMMAND_PLUSPLAYLIST,
        b"POST /minus5" => COMMAND_MINUS5,
        b"POST /plus5" => COMMAND_PLUS5,
        b"POST /minus75" => COMMAND_MINUS75,
        b"POST /plus75" => COMMAND_PLUS75,
        b"POST /fullscreen" => COMMAND_FULLSCREEN,
        b"POST /subtitles" => COMMAND_SUBTITLES,
        b"POST /title" => COMMAND_TITLE,
        b"POST /time" => COMMAND_TIME,
        b"POST /quit" => COMMAND_QUIT,
        _ => return None,
    };
    
    Some(result)
}

fn write_to_named_pipe(path: &str, data: &[u8]) -> io::Result<()> {
    unsafe {
        
        let result = WaitNamedPipeW(
            chikuwa::win_string(path).as_ptr(),
            PIPE_MAX_WAIT_AS_MILLIS,
        );
        
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        
    }
    
    let mut pipe = File::options()
        .write(true)
        .open(path)?;
    
    pipe.write_all(data)
}
