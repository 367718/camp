use std::{
    collections::HashMap,
    env,
    path::{ Path, PathBuf },
    time::Duration,
};

use crate:: { Window, Media };

pub(crate) const WINDOW_MAXIMIZED_ARG: &str = "--maximized";
pub(crate) const WINDOW_WIDTH_ARG: &str = "--width";
pub(crate) const WINDOW_HEIGHT_ARG: &str = "--height";
pub(crate) const WINDOW_X_ARG: &str = "--x";
pub(crate) const WINDOW_Y_ARG: &str = "--y";

pub(crate) const MEDIA_PLAYER_ARG: &str = "--player";
pub(crate) const MEDIA_ICONIFY_ARG: &str = "--iconify";
pub(crate) const MEDIA_FLAG_ARG: &str = "--flag";
pub(crate) const MEDIA_TIMEOUT_ARG: &str = "--timeout";
pub(crate) const MEDIA_AUTOSELECT_ARG: &str = "--autoselect";
pub(crate) const MEDIA_LOOKUP_ARG: &str = "--lookup";
pub(crate) const MEDIA_BIND_ARG: &str = "--bind";

pub(crate) const PATHS_FILES_ARG: &str = "--files";
pub(crate) const PATHS_DOWNLOADS_ARG: &str = "--downloads";
pub(crate) const PATHS_PIPE_ARG: &str = "--pipe";
pub(crate) const PATHS_DATABASE_ARG: &str = "--database";

pub struct Args {
    
    // window
    
    window_maximized: Option<bool>,
    window_width: Option<i32>,
    window_height: Option<i32>,
    window_x: Option<i32>,
    window_y: Option<i32>,
    
    // media
    
    media_player: Option<String>,
    media_iconify: Option<bool>,
    media_flag: Option<String>,
    media_timeout: Option<Duration>,
    media_autoselect: Option<bool>,
    media_lookup: Option<String>,
    media_bind: Option<String>,
    
    // paths
    
    paths_files: Option<PathBuf>,
    paths_downloads: Option<PathBuf>,
    paths_pipe: Option<PathBuf>,
    paths_database: Option<PathBuf>,
    
    // free
    
    free_flags: Vec<String>,
    free_pairs: HashMap<String, String>,
    
}

impl Args {
    
    // ---------- constructors ----------
    
    
    pub fn new(data: Option<Vec<String>>, flags: &[&str], pairs: &[&str]) -> Self {
        let mut cmdargs = data.unwrap_or_else(|| env::args().collect());
        
        Self {
            
            // window
            
            window_maximized: Self::remove_value(&mut cmdargs, WINDOW_MAXIMIZED_ARG)
                .and_then(|value| match value {
                    value if value.eq_ignore_ascii_case("yes") => Some(true),
                    value if value.eq_ignore_ascii_case("no") => Some(false),
                    _ => None,
                }),
            
            window_width: Self::remove_value(&mut cmdargs, WINDOW_WIDTH_ARG)
                .and_then(|value| value.parse().ok())
                .filter(|&value| Window::validate_dimension(value).is_ok()),
            
            window_height: Self::remove_value(&mut cmdargs, WINDOW_HEIGHT_ARG)
                .and_then(|value| value.parse().ok())
                .filter(|&value| Window::validate_dimension(value).is_ok()),
            
            window_x: Self::remove_value(&mut cmdargs, WINDOW_X_ARG)
                .and_then(|value| value.parse().ok())
                .filter(|&value| Window::validate_coordinate(value).is_ok()),
            
            window_y: Self::remove_value(&mut cmdargs, WINDOW_Y_ARG)
                .and_then(|value| value.parse().ok())
                .filter(|&value| Window::validate_coordinate(value).is_ok()),
            
            // media
            
            media_player: Self::remove_value(&mut cmdargs, MEDIA_PLAYER_ARG)
                .filter(|value| Media::validate_player(value).is_ok()),
            
            media_iconify: Self::remove_value(&mut cmdargs, MEDIA_ICONIFY_ARG)
                .and_then(|value| match value {
                    value if value.eq_ignore_ascii_case("yes") => Some(true),
                    value if value.eq_ignore_ascii_case("no") => Some(false),
                    _ => None,
                }),
            
            media_flag: Self::remove_value(&mut cmdargs, MEDIA_FLAG_ARG)
                .filter(|value| Media::validate_flag(value).is_ok()),
            
            media_timeout: Self::remove_value(&mut cmdargs, MEDIA_TIMEOUT_ARG)
                .and_then(|value| value.parse().ok())
                .map(Duration::from_secs),
            
            media_autoselect: Self::remove_value(&mut cmdargs, MEDIA_AUTOSELECT_ARG)
                .and_then(|value| match value {
                    value if value.eq_ignore_ascii_case("yes") => Some(true),
                    value if value.eq_ignore_ascii_case("no") => Some(false),
                    _ => None,
                }),
            
            media_lookup: Self::remove_value(&mut cmdargs, MEDIA_LOOKUP_ARG)
                .filter(|value| Media::validate_lookup(value).is_ok()),
            
            media_bind: Self::remove_value(&mut cmdargs, MEDIA_BIND_ARG)
                .filter(|value| Media::validate_bind(value).is_ok()),
            
            // paths
            
            paths_files: Self::remove_value(&mut cmdargs, PATHS_FILES_ARG)
                .map(PathBuf::from),
            
            paths_downloads: Self::remove_value(&mut cmdargs, PATHS_DOWNLOADS_ARG)
                .map(PathBuf::from),
            
            paths_pipe: Self::remove_value(&mut cmdargs, PATHS_PIPE_ARG)
                .map(PathBuf::from),
            
            paths_database: Self::remove_value(&mut cmdargs, PATHS_DATABASE_ARG)
                .map(PathBuf::from),
            
            // free
            
            free_flags: flags.iter()
                .filter(|key| key.starts_with("--"))
                .filter_map(|key| Self::remove_flag(&mut cmdargs, key))
                .collect(),
            
            free_pairs: pairs.iter()
                .filter(|key| key.starts_with("--"))
                .filter_map(|key| Self::remove_pair(&mut cmdargs, key))
                .collect(),
            
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn free_flag(&self, search: &str) -> bool {
        self.free_flags.iter().any(|key| key.eq_ignore_ascii_case(search))
    }
    
    pub fn free_value(&self, search: &str) -> Option<&str> {
        for (key, value) in &self.free_pairs {
            if key.eq_ignore_ascii_case(search) {
                return Some(value.as_str());
            }
        }
        
        None
    }
    
    
    // ---------- helpers ----------
    
    
    fn remove_value(cmdargs: &mut Vec<String>, key: &str) -> Option<String> {
        for (index, window) in cmdargs.windows(2).enumerate() {
            if window[0].eq_ignore_ascii_case(key) && ! window[1].starts_with("--") {
                
                // remove key
                cmdargs.remove(index);
                
                // remove value
                return Some(cmdargs.remove(index));
                
            }
        }
        
        None
    }
    
    fn remove_pair(cmdargs: &mut Vec<String>, key: &str) -> Option<(String, String)> {
        for (index, window) in cmdargs.windows(2).enumerate() {
            if window[0].eq_ignore_ascii_case(key) && ! window[1].starts_with("--") {
                
                return Some((
                    // remove key
                    cmdargs.remove(index),
                    // remove value
                    cmdargs.remove(index),
                ));
                
            }
        }
        
        None
    }
    
    fn remove_flag(cmdargs: &mut Vec<String>, key: &str) -> Option<String> {
        for (index, flag) in cmdargs.iter().enumerate() {
            if flag.eq_ignore_ascii_case(key) {
                
                return Some(cmdargs.remove(index));
                
            }
        }
        
        None
    }
    
}

// window

impl Args {
    
    // ---------- accessors ----------
    
    
    pub fn window_maximized(&self) -> Option<bool> {
        self.window_maximized
    }
    
    pub fn window_width(&self) -> Option<i32> {
        self.window_width
    }
    
    pub fn window_height(&self) -> Option<i32> {
        self.window_height
    }
    
    pub fn window_x(&self) -> Option<i32> {
        self.window_x
    }
    
    pub fn window_y(&self) -> Option<i32> {
        self.window_y
    }
    
}

// media

impl Args {
    
    // ---------- accessors ----------
    
    
    pub fn media_player(&self) -> Option<&str> {
        self.media_player.as_deref()
    }
    
    pub fn media_iconify(&self) -> Option<bool> {
        self.media_iconify
    }
    
    pub fn media_flag(&self) -> Option<&str> {
        self.media_flag.as_deref()
    }
    
    pub fn media_timeout(&self) -> Option<Duration> {
        self.media_timeout
    }
    
    pub fn media_autoselect(&self) -> Option<bool> {
        self.media_autoselect
    }
    
    pub fn media_lookup(&self) -> Option<&str> {
        self.media_lookup.as_deref()
    }
    
    pub fn media_bind(&self) -> Option<&str> {
        self.media_bind.as_deref()
    }
    
}

// paths

impl Args {
    
    // ---------- accessors ----------
    
    
    pub fn paths_files(&self) -> Option<&Path> {
        self.paths_files.as_deref()
    }
    
    pub fn paths_downloads(&self) -> Option<&Path> {
        self.paths_downloads.as_deref()
    }
    
    pub fn paths_pipe(&self) -> Option<&Path> {
        self.paths_pipe.as_deref()
    }
    
    pub fn paths_database(&self) -> Option<&Path> {
        self.paths_database.as_deref()
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn empty() {
        // setup
        
        let data = None;
        let flags = &[];
        let pairs = &[];
        
        // operation
        
        let output = Args::new(data, flags, pairs);
        
        // control
        
        assert_eq!(output.window_maximized(), None);
        assert_eq!(output.window_width(), None);
        assert_eq!(output.window_height(), None);
        assert_eq!(output.window_x(), None);
        assert_eq!(output.window_y(), None);
        
        assert_eq!(output.media_player(), None);
        assert_eq!(output.media_iconify(), None);
        assert_eq!(output.media_flag(), None);
        assert_eq!(output.media_timeout(), None);
        assert_eq!(output.media_autoselect(), None);
        assert_eq!(output.media_lookup(), None);
        assert_eq!(output.media_bind(), None);
        
        assert_eq!(output.paths_files(), None);
        assert_eq!(output.paths_downloads(), None);
        assert_eq!(output.paths_pipe(), None);
        assert_eq!(output.paths_database(), None);
        
        assert_eq!(output.free_flag("--help"), false);
        assert_eq!(output.free_value("--unknown"), None);
    }
    
    #[test]
    fn full() {
        // setup
        
        let data = Some(Vec::from([
            
            String::from("--HelP"),
            String::from("no"),
            String::from("--test"),
            String::from("--unknown"),
            String::from("it is"),
            
            String::from(WINDOW_MAXIMIZED_ARG),
            String::from("no"),
            String::from(WINDOW_WIDTH_ARG),
            String::from("100"),
            String::from(WINDOW_HEIGHT_ARG),
            String::from("200"),
            String::from(WINDOW_X_ARG),
            String::from("10"),
            String::from(WINDOW_Y_ARG),
            String::from("25"),
            
            String::from(MEDIA_PLAYER_ARG),
            String::from("mpv"),
            String::from(MEDIA_ICONIFY_ARG).to_uppercase(),
            String::from("yes"),
            String::from(MEDIA_FLAG_ARG),
            String::from("user.test.flag"),
            String::from(MEDIA_TIMEOUT_ARG),
            String::from("25"),
            String::from(MEDIA_AUTOSELECT_ARG),
            String::from("yes"),
            String::from(MEDIA_LOOKUP_ARG),
            String::from("https://example.com/search?q=%s"),
            String::from(MEDIA_BIND_ARG),
            String::from("0.0.0.0:0000"),
            
            String::from(PATHS_FILES_ARG),
            String::from("/home/me/files"),
            String::from(PATHS_DOWNLOADS_ARG),
            String::from("/home/me/downloads"),
            String::from(PATHS_PIPE_ARG),
            String::from("/example/pipe"),
            String::from(PATHS_DATABASE_ARG),
            String::from("/home/me/.local/app/app.db"),
            
        ]));
        
        let flags = &[
            "--help",
        ];
        
        let pairs = &[
            "--unknown"
        ];
        
        // operation
        
        let output = Args::new(data, flags, pairs);
        
        // control
        
        assert_eq!(output.window_maximized(), Some(false));
        assert_eq!(output.window_width(), Some(100));
        assert_eq!(output.window_height(), Some(200));
        assert_eq!(output.window_x(), Some(10));
        assert_eq!(output.window_y(), Some(25));
        
        assert_eq!(output.media_player(), Some("mpv"));
        assert_eq!(output.media_iconify(), Some(true));
        assert_eq!(output.media_flag(), Some("user.test.flag"));
        assert_eq!(output.media_timeout(), Some(Duration::from_secs(25)));
        assert_eq!(output.media_autoselect(), Some(true));
        assert_eq!(output.media_lookup(), Some("https://example.com/search?q=%s"));
        assert_eq!(output.media_bind(), Some("0.0.0.0:0000"));
        
        assert_eq!(output.paths_files(), Some(Path::new("/home/me/files")));
        assert_eq!(output.paths_downloads(), Some(Path::new("/home/me/downloads")));
        assert_eq!(output.paths_pipe(), Some(Path::new("/example/pipe")));
        assert_eq!(output.paths_database(), Some(Path::new("/home/me/.local/app/app.db")));
        
        assert_eq!(output.free_flag("--help"), true);
        assert_eq!(output.free_flag("--test"), false);
        assert_eq!(output.free_value("--unknown"), Some("it is"));
    }
    
    #[test]
    fn unordered() {
        // setup
        
        let data = Some(Vec::from([
            
            String::from("--HelP"),
            String::from("yes"),
            
            String::from(PATHS_FILES_ARG),
            String::from("/home/me/files"),
            
            String::from(WINDOW_MAXIMIZED_ARG),
            String::from("no"),
            String::from(WINDOW_HEIGHT_ARG),
            String::from("200"),
            String::from(WINDOW_X_ARG),
            String::from("10"),
            String::from(WINDOW_Y_ARG),
            String::from("25"),
            
            String::from(MEDIA_PLAYER_ARG),
            String::from("mpv"),
            String::from(MEDIA_ICONIFY_ARG),
            String::from("yes"),
            String::from(MEDIA_TIMEOUT_ARG),
            String::from("25"),
            String::from(MEDIA_AUTOSELECT_ARG),
            String::from("yes"),
            String::from(MEDIA_LOOKUP_ARG),
            String::from("https://example.com/search?q=%s"),
            String::from(MEDIA_BIND_ARG),
            String::from("0.0.0.0:0000"),
            
            String::from("--unknown"),
            String::from("no"),
            
            String::from(MEDIA_FLAG_ARG),
            String::from("user.test.flag"),
            
            String::from(PATHS_DOWNLOADS_ARG),
            String::from("/home/me/downloads"),
            String::from(PATHS_PIPE_ARG),
            String::from("/example/pipe"),
            String::from(PATHS_DATABASE_ARG),
            String::from("/home/me/.local/app/app.db"),
            
            String::from(WINDOW_WIDTH_ARG),
            String::from("100"),
            
        ]));
        
        let flags = &[
            "--help",
            "--unknown",
        ];
        
        let pairs = &[];
        
        // operation
        
        let output = Args::new(data, flags, pairs);
        
        // control
        
        assert_eq!(output.window_maximized(), Some(false));
        assert_eq!(output.window_width(), Some(100));
        assert_eq!(output.window_height(), Some(200));
        assert_eq!(output.window_x(), Some(10));
        assert_eq!(output.window_y(), Some(25));
        
        assert_eq!(output.media_player(), Some("mpv"));
        assert_eq!(output.media_iconify(), Some(true));
        assert_eq!(output.media_flag(), Some("user.test.flag"));
        assert_eq!(output.media_timeout(), Some(Duration::from_secs(25)));
        assert_eq!(output.media_autoselect(), Some(true));
        assert_eq!(output.media_lookup(), Some("https://example.com/search?q=%s"));
        assert_eq!(output.media_bind(), Some("0.0.0.0:0000"));
        
        assert_eq!(output.paths_files(), Some(Path::new("/home/me/files")));
        assert_eq!(output.paths_downloads(), Some(Path::new("/home/me/downloads")));
        assert_eq!(output.paths_pipe(), Some(Path::new("/example/pipe")));
        assert_eq!(output.paths_database(), Some(Path::new("/home/me/.local/app/app.db")));
        
        assert_eq!(output.free_flag("--help"), true);
        assert_eq!(output.free_flag("--unknown"), true);
        assert_eq!(output.free_value("--unknown"), None);
    }
    
    #[test]
    fn invalid() {
        // setup
        
        let data = Some(Vec::from([
            
            String::from("test"),
            String::from("-placeholder"),
            
            String::from(WINDOW_MAXIMIZED_ARG),
            String::from("maybe"),
            String::from(WINDOW_WIDTH_ARG),
            String::from("0"),
            String::from(WINDOW_HEIGHT_ARG),
            String::from("0"),
            String::from(WINDOW_X_ARG),
            String::from("-10"),
            String::from(WINDOW_Y_ARG),
            String::from("-10"),
            
            String::from(MEDIA_PLAYER_ARG),
            String::from("--mpv"),
            String::from(MEDIA_ICONIFY_ARG),
            String::from("maybe"),
            String::from(MEDIA_FLAG_ARG),
            String::from(""),
            String::from(MEDIA_TIMEOUT_ARG),
            String::from("-1"),
            String::from(MEDIA_AUTOSELECT_ARG),
            String::from("maybe"),
            String::from(MEDIA_LOOKUP_ARG),
            String::from(""),
            String::from(MEDIA_BIND_ARG),
            String::from(""),
            
            String::from(PATHS_DOWNLOADS_ARG),
            String::from(PATHS_PIPE_ARG),
            String::from(PATHS_DATABASE_ARG),
            
        ]));
        
        let flags = &[
            "--new",
            "test",
            "-placeholder",
        ];
        
        let pairs = &[
            "-placeholder",
        ];
        
        // operation
        
        let output = Args::new(data, flags, pairs);
        
        // control
        
        assert_eq!(output.window_maximized(), None);
        assert_eq!(output.window_width(), None);
        assert_eq!(output.window_height(), None);
        assert_eq!(output.window_x(), None);
        assert_eq!(output.window_y(), None);
        
        assert_eq!(output.media_iconify(), None);
        assert_eq!(output.media_flag(), None);
        assert_eq!(output.media_timeout(), None);
        assert_eq!(output.media_autoselect(), None);
        assert_eq!(output.media_lookup(), None);
        assert_eq!(output.media_bind(), None);
        
        assert_eq!(output.paths_files(), None);
        assert_eq!(output.paths_downloads(), None);
        assert_eq!(output.paths_pipe(), None);
        assert_eq!(output.paths_database(), None);
        
        assert_eq!(output.free_flag("--help"), false);
        assert_eq!(output.free_flag("test"), false);
        assert_eq!(output.free_flag("-placeholder"), false);
        assert_eq!(output.free_value("-placeholder"), None);
    }
    
    #[test]
    fn partial() {
        // setup
        
        let data = Some(Vec::from([
            
            String::from("--unknown"),
            String::from("no"),
            
            String::from(WINDOW_WIDTH_ARG),
            String::from("100"),
            String::from(WINDOW_X_ARG),
            String::from("10"),
            String::from(WINDOW_Y_ARG),
            String::from("25"),
            
            String::from(MEDIA_PLAYER_ARG),
            String::from("mpv"),
            String::from(MEDIA_TIMEOUT_ARG),
            String::from("30"),
            String::from(MEDIA_LOOKUP_ARG),
            String::from(MEDIA_LOOKUP_ARG),
            String::from("https://example.com/search?q=%s"),
            
            String::from(PATHS_FILES_ARG),
            String::from("/home/me/files"),
            String::from(PATHS_DOWNLOADS_ARG),
            String::from("/home/me/Downloads"),
            
        ]));
        
        let flags = &[
            "--unknown",
        ];
        
        let pairs = &[];
        
        // operation
        
        let output = Args::new(data, flags, pairs);
        
        // control
        
        assert_eq!(output.window_maximized(), None);
        assert_eq!(output.window_width(), Some(100));
        assert_eq!(output.window_height(), None);
        assert_eq!(output.window_x(), Some(10));
        assert_eq!(output.window_y(), Some(25));
        
        assert_eq!(output.media_player(), Some("mpv"));
        assert_eq!(output.media_iconify(), None);
        assert_eq!(output.media_flag(), None);
        assert_eq!(output.media_timeout(), Some(Duration::from_secs(30)));
        assert_eq!(output.media_autoselect(), None);
        assert_eq!(output.media_lookup(), Some("https://example.com/search?q=%s"));
        assert_eq!(output.media_bind(), None);
        
        assert_eq!(output.paths_files(), Some(Path::new("/home/me/files")));
        assert_eq!(output.paths_downloads(), Some(Path::new("/home/me/Downloads")));
        assert_eq!(output.paths_pipe(), None);
        assert_eq!(output.paths_database(), None);
        
        assert_eq!(output.free_flag("--help"), false);
        assert_eq!(output.free_flag("--unknown"), true);
        assert_eq!(output.free_value("--help"), None);
        assert_eq!(output.free_value("--unknown"), None);
    }
    
}
