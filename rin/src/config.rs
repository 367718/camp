use std::{
    error::Error,
    fs::{ self, File },
    io::Write,
    path::Path,
    str,
    time::Duration,
};

use crate::{ Window, Media, Paths };

pub struct Config {
    modified: bool,
    window: Window,
    media: Media,
    paths: Paths,
}

impl Config {
    
    // ---------- constructors ----------
    
    
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut config = Self {
            modified: true,
            window: Window::new(),
            media: Media::new(),
            paths: Paths::new(),
        };
        
        config.save(path)?;
        
        Ok(config)
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read(&path)?;
        
        let mut modified = false;
        
        // ----- window -----
        
        let window = {
            let (data, corrected) = Window::deserialize(&content)?;
            modified |= corrected;
            data
        };
        
        // ----- media -----
        
        let media = {
            let (data, corrected) = Media::deserialize(&content)?;
            modified |= corrected;
            data
        };
        
        // ----- paths -----
        
        let paths = {
            let (data, corrected) = Paths::deserialize(&content)?;
            modified |= corrected;
            data
        };
        
        Ok(Self {
            modified,
            window,
            media,
            paths,
        })
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        if self.modified {
            
            // attempt to perform the update atomically
            
            let tmp_path = chikuwa::EphemeralPath::builder().build();
            let mut tmp_file = File::create(&tmp_path)?;
            
            self.window.serialize(&mut tmp_file)?;
            self.media.serialize(&mut tmp_file)?;
            self.paths.serialize(&mut tmp_file)?;
            
            tmp_file.flush()?;
            
            fs::rename(&tmp_path, path)?;
            
            tmp_path.unmanage();
            
            self.modified = false;
            
        }
        
        Ok(())
    }
    
    fn modify(&mut self, result: bool) -> bool {
        self.modified |= result;
        result
    }
    
    
    // ---------- helpers ----------
    
    
    pub(crate) fn get_value<'a>(data: &'a [u8], key: &[u8]) -> Result<&'a str, Box<dyn Error>> {
        if let Some(line) = data.split(|&value| value == b'\n').find(|line| line.starts_with(key)) {
            if let [b' ', b'=', b' ', value @ ..] = &line[key.len()..] {
                
                let value = value.strip_suffix(&[b'\r'])
                    .unwrap_or(value);
                
                return Ok(str::from_utf8(value)?);
                
            }
        }
        
        let base = "Missing or invalid field: ";
        let field = str::from_utf8(key)?;
        
        let mut message = String::with_capacity(base.len() + field.len());
        
        message.push_str(base);
        message.push_str(field);
        
        Err(message.into())
    }
    
}

// window

impl Config {
    
    // ---------- acessors ----------
    
    
    pub fn window_maximized(&self) -> bool {
        self.window.maximized()
    }
    
    pub fn window_width(&self) -> i32 {
        self.window.width()
    }
    
    pub fn window_height(&self) -> i32 {
        self.window.height()
    }
    
    pub fn window_x(&self) -> i32 {
        self.window.x()
    }
    
    pub fn window_y(&self) -> i32 {
        self.window.y()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn window_set_maximized(&mut self, maximized: bool) -> bool {
        let result = self.window.set_maximized(maximized);
        self.modify(result)
    }
    
    pub fn window_set_width(&mut self, width: i32) -> Result<bool, Box<dyn Error>> {
        let result = self.window.set_width(width)?;
        Ok(self.modify(result))
    }
    
    pub fn window_set_height(&mut self, height: i32) -> Result<bool, Box<dyn Error>> {
        let result = self.window.set_height(height)?;
        Ok(self.modify(result))
    }
    
    pub fn window_set_x(&mut self, x: i32) -> Result<bool, Box<dyn Error>> {
        let result = self.window.set_x(x)?;
        Ok(self.modify(result))
    }
    
    pub fn window_set_y(&mut self, y: i32) -> Result<bool, Box<dyn Error>> {
        let result = self.window.set_y(y)?;
        Ok(self.modify(result))
    }
    
}

// media

impl Config {
    
    // ---------- accessors ----------
    
    
    pub fn media_player(&self) -> &str {
        self.media.player()
    }
    
    pub fn media_iconify(&self) -> bool {
        self.media.iconify()
    }
    
    pub fn media_flag(&self) -> &str {
        self.media.flag()
    }
    
    pub fn media_timeout(&self) -> Duration {
        self.media.timeout()
    }
    
    pub fn media_autoselect(&self) -> bool {
        self.media.autoselect()
    }
    
    pub fn media_lookup(&self) -> &str {
        self.media.lookup()
    }
    
    pub fn media_bind(&self) -> &str {
        self.media.bind()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn media_set_player<S: AsRef<str>>(&mut self, player: S) -> Result<bool, Box<dyn Error>> {
        let result = self.media.set_player(player)?;
        Ok(self.modify(result))
    }
    
    pub fn media_set_iconify(&mut self, iconify: bool) -> bool {
        let result = self.media.set_iconify(iconify);
        self.modify(result)
    }
    
    pub fn media_set_flag<S: AsRef<str>>(&mut self, flag: S) -> Result<bool, Box<dyn Error>> {
        let result = self.media.set_flag(flag)?;
        Ok(self.modify(result))
    }
    
    pub fn media_set_timeout(&mut self, timeout: Duration) -> Result<bool, Box<dyn Error>> {
        let result = self.media.set_timeout(timeout)?;
        Ok(self.modify(result))
    }
    
    pub fn media_set_autoselect(&mut self, autoselect: bool) -> bool {
        let result = self.media.set_autoselect(autoselect);
        self.modify(result)
    }
    
    pub fn media_set_lookup<S: AsRef<str>>(&mut self, lookup: S) -> Result<bool, Box<dyn Error>> {
        let result = self.media.set_lookup(lookup)?;
        Ok(self.modify(result))
    }
    
    pub fn media_set_bind<S: AsRef<str>>(&mut self, bind: S) -> Result<bool, Box<dyn Error>> {
        let result = self.media.set_bind(bind)?;
        Ok(self.modify(result))
    }
    
}

// paths

impl Config {
    
    // ---------- accessors ----------
    
    
    pub fn paths_files(&self) -> &Path {
        self.paths.files()
    }
    
    pub fn paths_downloads(&self) -> &Path {
        self.paths.downloads()
    }
    
    pub fn paths_pipe(&self) -> &Path {
        self.paths.pipe()
    }
    
    pub fn paths_database(&self) -> &Path {
        self.paths.database()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn paths_set_files<P: AsRef<Path>>(&mut self, files: P) -> Result<bool, Box<dyn Error>> {
        let result = self.paths.set_files(files)?;
        Ok(self.modify(result))
    }
    
    pub fn paths_set_downloads<P: AsRef<Path>>(&mut self, downloads: P) -> Result<bool, Box<dyn Error>> {
        let result = self.paths.set_downloads(downloads)?;
        Ok(self.modify(result))
    }
    
    pub fn paths_set_pipe<P: AsRef<Path>>(&mut self, pipe: P) -> Result<bool, Box<dyn Error>> {
        let result = self.paths.set_pipe(pipe)?;
        Ok(self.modify(result))
    }
    
    pub fn paths_set_database<P: AsRef<Path>>(&mut self, database: P) -> Result<bool, Box<dyn Error>> {
        let result = self.paths.set_database(database)?;
        Ok(self.modify(result))
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    use std::io::Write;
    
    #[test]
    fn new() {
        // setup
        
        let tmp_path = chikuwa::EphemeralPath::builder().build();
        
        // operation
        
        let output = Config::new(&tmp_path);
        
        // control
        
        assert!(output.is_ok());
    }
    
    mod load {
        
        use super::*;
        
        #[test]
        fn valid_lf() {
            // setup
            
            let tmp_path = chikuwa::EphemeralPath::builder().build();
            
            let mut data = Vec::new();
            
            writeln!(data, "window.maximized = true").unwrap();
            writeln!(data, "window.width = 750").unwrap();
            writeln!(data, "window.height = 900").unwrap();
            writeln!(data, "window.x = 175").unwrap();
            writeln!(data, "window.y = 125").unwrap();
            
            writeln!(data, "media.player = vlc").unwrap();
            writeln!(data, "media.iconify = true").unwrap();
            writeln!(data, "media.flag = user.test.flag").unwrap();
            writeln!(data, "media.timeout = 5").unwrap();
            writeln!(data, "media.autoselect = true").unwrap();
            writeln!(data, "media.lookup = https://placeholder.com/search?q=%s\n").unwrap();
            writeln!(data, "media.bind = 192.168.0.1:7777").unwrap();
            
            writeln!(data, "paths.files = /placeholder/files").unwrap();
            writeln!(data, "paths.downloads = /placeholder/downloads").unwrap();
            writeln!(data, "paths.pipe = //./pipe/placeholder").unwrap();
            writeln!(data, "paths.database = /placeholder/database").unwrap();
            
            fs::write(&tmp_path, &data).unwrap();
            
            // operation
            
            let output = Config::load(&tmp_path);
            
            // control
            
            assert!(output.is_ok());
            
            let config = output.unwrap();
            
            assert_eq!(config.window_maximized(), true);
            assert_eq!(config.window_width(), 750);
            assert_eq!(config.window_height(), 900);
            assert_eq!(config.window_x(), 175);
            assert_eq!(config.window_y(), 125);
            
            assert_eq!(config.media_player(), "vlc");
            assert_eq!(config.media_iconify(), true);
            assert_eq!(config.media_flag(), "user.test.flag");
            assert_eq!(config.media_timeout(), Duration::from_secs(5));
            assert_eq!(config.media_autoselect(), true);
            assert_eq!(config.media_lookup(), "https://placeholder.com/search?q=%s");
            
            assert_eq!(config.paths_files(), Path::new("/placeholder/files"));
            assert_eq!(config.paths_downloads(), Path::new("/placeholder/downloads"));
            assert_eq!(config.paths_pipe(), Path::new("//./pipe/placeholder"));
            assert_eq!(config.paths_database(), Path::new("/placeholder/database"));
            
            assert_ne!(config.window_maximized(), Window::DEFAULT_MAXIMIZED);
            assert_ne!(config.window_width(), Window::DEFAULT_WIDTH);
            assert_ne!(config.window_height(), Window::DEFAULT_HEIGHT);
            assert_ne!(config.window_x(), Window::DEFAULT_X);
            assert_ne!(config.window_y(), Window::DEFAULT_Y);
            
            assert_ne!(config.media_player(), Media::DEFAULT_PLAYER);
            assert_ne!(config.media_iconify(), Media::DEFAULT_ICONIFY);
            assert_ne!(config.media_flag(), Media::DEFAULT_FLAG);
            assert_ne!(config.media_timeout(), Media::DEFAULT_TIMEOUT);
            assert_ne!(config.media_autoselect(), Media::DEFAULT_AUTOSELECT);
            assert_ne!(config.media_lookup(), Media::DEFAULT_LOOKUP);
            assert_ne!(config.media_bind(), Media::DEFAULT_BIND);
            
            assert_ne!(config.paths_files(), Path::new(Paths::DEFAULT_FILES));
            assert_ne!(config.paths_downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
            assert_ne!(config.paths_pipe(), Path::new(Paths::DEFAULT_PIPE));
            assert_ne!(config.paths_database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn valid_crlf() {
            // setup
            
            let tmp_path = chikuwa::EphemeralPath::builder().build();
            
            let mut data = Vec::new();
            
            writeln!(data, "window.maximized = true\r").unwrap();
            writeln!(data, "window.width = 750\r").unwrap();
            writeln!(data, "window.height = 900\r").unwrap();
            writeln!(data, "window.x = 175\r").unwrap();
            writeln!(data, "window.y = 125\r").unwrap();
            
            writeln!(data, "media.player = vlc\r").unwrap();
            writeln!(data, "media.iconify = true\r").unwrap();
            writeln!(data, "media.flag = user.test.flag\r").unwrap();
            writeln!(data, "media.timeout = 5\r").unwrap();
            writeln!(data, "media.autoselect = true\r").unwrap();
            writeln!(data, "media.lookup = https://placeholder.com/search?q=%s\n\r").unwrap();
            writeln!(data, "media.bind = 192.168.0.1:7777\r").unwrap();
            
            writeln!(data, "paths.files = /placeholder/files\r").unwrap();
            writeln!(data, "paths.downloads = /placeholder/downloads\r").unwrap();
            writeln!(data, "paths.pipe = //./pipe/placeholder\r").unwrap();
            writeln!(data, "paths.database = /placeholder/database\r").unwrap();
            
            fs::write(&tmp_path, &data).unwrap();
            
            // operation
            
            let output = Config::load(&tmp_path);
            
            // control
            
            assert!(output.is_ok());
            
            let config = output.unwrap();
            
            assert_eq!(config.window_maximized(), true);
            assert_eq!(config.window_width(), 750);
            assert_eq!(config.window_height(), 900);
            assert_eq!(config.window_x(), 175);
            assert_eq!(config.window_y(), 125);
            
            assert_eq!(config.media_player(), "vlc");
            assert_eq!(config.media_iconify(), true);
            assert_eq!(config.media_flag(), "user.test.flag");
            assert_eq!(config.media_timeout(), Duration::from_secs(5));
            assert_eq!(config.media_autoselect(), true);
            assert_eq!(config.media_lookup(), "https://placeholder.com/search?q=%s");
            
            assert_eq!(config.paths_files(), Path::new("/placeholder/files"));
            assert_eq!(config.paths_downloads(), Path::new("/placeholder/downloads"));
            assert_eq!(config.paths_pipe(), Path::new("//./pipe/placeholder"));
            assert_eq!(config.paths_database(), Path::new("/placeholder/database"));
            
            assert_ne!(config.window_maximized(), Window::DEFAULT_MAXIMIZED);
            assert_ne!(config.window_width(), Window::DEFAULT_WIDTH);
            assert_ne!(config.window_height(), Window::DEFAULT_HEIGHT);
            assert_ne!(config.window_x(), Window::DEFAULT_X);
            assert_ne!(config.window_y(), Window::DEFAULT_Y);
            
            assert_ne!(config.media_player(), Media::DEFAULT_PLAYER);
            assert_ne!(config.media_iconify(), Media::DEFAULT_ICONIFY);
            assert_ne!(config.media_flag(), Media::DEFAULT_FLAG);
            assert_ne!(config.media_timeout(), Media::DEFAULT_TIMEOUT);
            assert_ne!(config.media_autoselect(), Media::DEFAULT_AUTOSELECT);
            assert_ne!(config.media_lookup(), Media::DEFAULT_LOOKUP);
            assert_ne!(config.media_bind(), Media::DEFAULT_BIND);
            
            assert_ne!(config.paths_files(), Path::new(Paths::DEFAULT_FILES));
            assert_ne!(config.paths_downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
            assert_ne!(config.paths_pipe(), Path::new(Paths::DEFAULT_PIPE));
            assert_ne!(config.paths_database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let tmp_path = chikuwa::EphemeralPath::builder().build();
            
            let mut data = Vec::new();
            
            writeln!(data, "window.maximized = false").unwrap();
            writeln!(data, "window.width = 0").unwrap();
            writeln!(data, "window.height = 0").unwrap();
            writeln!(data, "window.x = -1").unwrap();
            writeln!(data, "window.y = -1").unwrap();
            
            writeln!(data, "media.player = ").unwrap();
            writeln!(data, "media.iconify = false").unwrap();
            writeln!(data, "media.flag = ").unwrap();
            writeln!(data, "media.timeout = 0").unwrap();
            writeln!(data, "media.autoselect = false").unwrap();
            writeln!(data, "media.lookup = ").unwrap();
            writeln!(data, "media.bind = ").unwrap();
            
            writeln!(data, "paths.files = ").unwrap();
            writeln!(data, "paths.downloads = ").unwrap();
            writeln!(data, "paths.pipe = ").unwrap();
            writeln!(data, "paths.database = ").unwrap();
            
            fs::write(&tmp_path, &data).unwrap();
            
            // operation
            
            let output = Config::load(&tmp_path);
            
            // control
            
            assert!(output.is_ok());
            
            let config = output.unwrap();
            
            assert_eq!(config.window_maximized(), Window::DEFAULT_MAXIMIZED);
            assert_eq!(config.window_width(), Window::DEFAULT_WIDTH);
            assert_eq!(config.window_height(), Window::DEFAULT_HEIGHT);
            assert_eq!(config.window_x(), Window::DEFAULT_X);
            assert_eq!(config.window_y(), Window::DEFAULT_Y);
            
            assert_eq!(config.media_player(), Media::DEFAULT_PLAYER);
            assert_eq!(config.media_iconify(), Media::DEFAULT_ICONIFY);
            assert_eq!(config.media_flag(), Media::DEFAULT_FLAG);
            assert_eq!(config.media_timeout(), Media::DEFAULT_TIMEOUT);
            assert_eq!(config.media_autoselect(), Media::DEFAULT_AUTOSELECT);
            assert_eq!(config.media_lookup(), Media::DEFAULT_LOOKUP);
            
            assert_eq!(config.paths_files(), Path::new(Paths::DEFAULT_FILES));
            assert_eq!(config.paths_downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
            assert_eq!(config.paths_pipe(), Path::new(Paths::DEFAULT_PIPE));
            assert_eq!(config.paths_database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
    }
    
    mod save {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let tmp_path = chikuwa::EphemeralPath::builder().build();
            
            let mut config = Config::new(&tmp_path).unwrap();
            
            config.media_set_player("mpc-hc").unwrap();
            
            // operation
            
            let output = config.save(&tmp_path);
            
            // control
            
            assert!(output.is_ok());
            
            let config = Config::load(&tmp_path).unwrap();
            
            assert_eq!(config.media_player(), "mpc-hc");
        }
        
    }
    
}
