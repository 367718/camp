mod args;
mod config;

mod window;
mod media;
mod paths;

use std::{
    error::Error,
    path::Path,
    time::Duration,
};

pub use args::Args;
pub use config::Config;

use window::Window;
use media::Media;
use paths::Paths;

pub struct Params {
    args: Args,
    config: Config,
}

impl Params {
    
    // ---------- constructors ----------
    
    
    pub fn new(args: Args, config: Config) -> Self {
        Self { args, config }
    }
    
    
    // ---------- helpers ----------
    
    
    fn arg_or_config<T>(arg: bool, value_arg: Option<T>, value_config: T) -> T {
        if arg {
            if let Some(value) = value_arg {
                return value;
            }
        }
        
        value_config
    }
    
}

// args

impl Params {
    
    // ---------- accessors ----------
    
    
    // window
    
    pub fn args_window_maximized(&self) -> Option<bool> {
        self.args.window_maximized()
    }
    
    pub fn args_window_width(&self) -> Option<i32> {
        self.args.window_width()
    }
    
    pub fn args_window_height(&self) -> Option<i32> {
        self.args.window_height()
    }
    
    pub fn args_window_x(&self) -> Option<i32> {
        self.args.window_x()
    }
    
    pub fn args_window_y(&self) -> Option<i32> {
        self.args.window_y()
    }
    
    // media
    
    pub fn args_media_player(&self) -> Option<&str> {
        self.args.media_player()
    }
    
    pub fn args_media_iconify(&self) -> Option<bool> {
        self.args.media_iconify()
    }
    
    pub fn args_media_flag(&self) -> Option<&str> {
        self.args.media_flag()
    }
    
    pub fn args_media_timeout(&self) -> Option<Duration> {
        self.args.media_timeout()
    }
    
    pub fn args_media_autoselect(&self) -> Option<bool> {
        self.args.media_autoselect()
    }
    
    pub fn args_media_lookup(&self) -> Option<&str> {
        self.args.media_lookup()
    }
    
    pub fn args_media_bind(&self) -> Option<&str> {
        self.args.media_bind()
    }
    
    // paths
    
    pub fn args_paths_files(&self) -> Option<&Path> {
        self.args.paths_files()
    }
    
    pub fn args_paths_downloads(&self) -> Option<&Path> {
        self.args.paths_downloads()
    }
    
    pub fn args_paths_pipe(&self) -> Option<&Path> {
        self.args.paths_pipe()
    }
    
    pub fn args_paths_database(&self) -> Option<&Path> {
        self.args.paths_database()
    }
    
    // free
    
    pub fn args_free_flag(&self, key: &str) -> bool {
        self.args.free_flag(key)
    }
    
    pub fn args_free_value(&self, key: &str) -> Option<&str> {
        self.args.free_value(key)
    }
    
}

// config

impl Params {
    
    // ---------- mutators ----------
    
    
    pub fn config_save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        self.config.save(path)
    }
    
}

// window

impl Params {
    
    // ---------- accessors ----------
    
    
    pub fn window_maximized(&self, arg: bool) -> bool {
        Self::arg_or_config(arg, self.args_window_maximized(), self.config.window_maximized())
    }
    
    pub fn window_width(&self, arg: bool) -> i32 {
        Self::arg_or_config(arg, self.args_window_width(), self.config.window_width())
    }
    
    pub fn window_height(&self, arg: bool) -> i32 {
        Self::arg_or_config(arg, self.args_window_height(), self.config.window_height())
    }
    
    pub fn window_x(&self, arg: bool) -> i32 {
        Self::arg_or_config(arg, self.args_window_x(), self.config.window_x())
    }
    
    pub fn window_y(&self, arg: bool) -> i32 {
        Self::arg_or_config(arg, self.args_window_y(), self.config.window_y())
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn window_set_maximized(&mut self, maximized: bool) -> bool {
        self.config.window_set_maximized(maximized)
    }
    
    pub fn window_set_width(&mut self, width: i32) -> Result<bool, Box<dyn Error>> {
        self.config.window_set_width(width)
    }
    
    pub fn window_set_height(&mut self, height: i32) -> Result<bool, Box<dyn Error>> {
        self.config.window_set_height(height)
    }
    
    pub fn window_set_x(&mut self, x: i32) -> Result<bool, Box<dyn Error>> {
        self.config.window_set_x(x)
    }
    
    pub fn window_set_y(&mut self, y: i32) -> Result<bool, Box<dyn Error>> {
        self.config.window_set_y(y)
    }
    
}

// media

impl Params {
    
    // ---------- accessors ----------
    
    
    pub fn media_player(&self, arg: bool) -> &str {
        Self::arg_or_config(arg, self.args_media_player(), self.config.media_player())
    }
    
    pub fn media_iconify(&self, arg: bool) -> bool {
        Self::arg_or_config(arg, self.args_media_iconify(), self.config.media_iconify())
    }
    
    pub fn media_flag(&self, arg: bool) -> &str {
        Self::arg_or_config(arg, self.args_media_flag(), self.config.media_flag())
    }
    
    pub fn media_timeout(&self, arg: bool) -> Duration {
        Self::arg_or_config(arg, self.args_media_timeout(), self.config.media_timeout())
    }
    
    pub fn media_autoselect(&self, arg: bool) -> bool {
        Self::arg_or_config(arg, self.args_media_autoselect(), self.config.media_autoselect())
    }
    
    pub fn media_lookup(&self, arg: bool) -> &str {
        Self::arg_or_config(arg, self.args_media_lookup(), self.config.media_lookup())
    }
    
    pub fn media_bind(&self, arg: bool) -> &str {
        Self::arg_or_config(arg, self.args_media_bind(), self.config.media_bind())
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn media_set_player<S: AsRef<str>>(&mut self, player: S) -> Result<bool, Box<dyn Error>> {
        self.config.media_set_player(player)
    }
    
    pub fn media_set_iconify(&mut self, iconify: bool) -> bool {
        self.config.media_set_iconify(iconify)
    }
    
    pub fn media_set_flag<S: AsRef<str>>(&mut self, flag: S) -> Result<bool, Box<dyn Error>> {
        self.config.media_set_flag(flag)
    }
    
    pub fn media_set_timeout(&mut self, timeout: Duration) -> Result<bool, Box<dyn Error>> {
        self.config.media_set_timeout(timeout)
    }
    
    pub fn media_set_autoselect(&mut self, autoselect: bool) -> bool {
        self.config.media_set_autoselect(autoselect)
    }
    
    pub fn media_set_lookup<S: AsRef<str>>(&mut self, lookup: S) -> Result<bool, Box<dyn Error>> {
        self.config.media_set_lookup(lookup)
    }
    
    pub fn media_set_bind<S: AsRef<str>>(&mut self, bind: S) -> Result<bool, Box<dyn Error>> {
        self.config.media_set_bind(bind)
    }
    
}

// paths

impl Params {
    
    // ---------- accessors ----------
    
    
    pub fn paths_files(&self, arg: bool) -> &Path {
        Self::arg_or_config(arg, self.args_paths_files(), self.config.paths_files())
    }
    
    pub fn paths_downloads(&self, arg: bool) -> &Path {
        Self::arg_or_config(arg, self.args_paths_downloads(), self.config.paths_downloads())
    }
    
    pub fn paths_pipe(&self, arg: bool) -> &Path {
        Self::arg_or_config(arg, self.args_paths_pipe(), self.config.paths_pipe())
    }
    
    pub fn paths_database(&self, arg: bool) -> &Path {
        Self::arg_or_config(arg, self.args_paths_database(), self.config.paths_database())
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn paths_set_files<P: AsRef<Path>>(&mut self, files: P) -> Result<bool, Box<dyn Error>> {
        self.config.paths_set_files(files)
    }
    
    pub fn paths_set_downloads<P: AsRef<Path>>(&mut self, downloads: P) -> Result<bool, Box<dyn Error>> {
        self.config.paths_set_downloads(downloads)
    }
    
    pub fn paths_set_pipe<P: AsRef<Path>>(&mut self, pipe: P) -> Result<bool, Box<dyn Error>> {
        self.config.paths_set_pipe(pipe)
    }
    
    pub fn paths_set_database<P: AsRef<Path>>(&mut self, database: P) -> Result<bool, Box<dyn Error>> {
        self.config.paths_set_database(database)
    }
    
}

#[cfg(test)]
mod lib {
    
    use super::*;
    
    use std::{
        path::Path,
        time::Duration,
    };
    
    #[test]
    fn window() {
        // setup
        
        let data = Some(Vec::from([
            String::from(args::WINDOW_MAXIMIZED_ARG),
            String::from("yes"),
            String::from(args::WINDOW_WIDTH_ARG),
            String::from("100"),
            String::from(args::WINDOW_HEIGHT_ARG),
            String::from("200"),
            String::from(args::WINDOW_X_ARG),
            String::from("10"),
            String::from(args::WINDOW_Y_ARG),
            String::from("30"),
        ]));
        
        let flags = &[];
        let pairs = &[];
        
        let args = Args::new(data, flags, pairs);
        
        let cfgpath = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path();
        
        let config = Config::new(&cfgpath).unwrap();
        
        // operation
        
        let output = Params::new(args, config);
        
        // control
        
        assert_eq!(output.window_maximized(false), Window::DEFAULT_MAXIMIZED);
        assert_eq!(output.window_maximized(true), true);
        assert_ne!(output.window_maximized(true), Window::DEFAULT_MAXIMIZED);
        
        assert_eq!(output.window_width(false), Window::DEFAULT_WIDTH);
        assert_eq!(output.window_width(true), 100);
        assert_ne!(output.window_width(true), Window::DEFAULT_WIDTH);
        
        assert_eq!(output.window_height(false), Window::DEFAULT_HEIGHT);
        assert_eq!(output.window_height(true), 200);
        assert_ne!(output.window_height(true), Window::DEFAULT_HEIGHT);
        
        assert_eq!(output.window_x(false), Window::DEFAULT_X);
        assert_eq!(output.window_x(true), 10);
        assert_ne!(output.window_x(true), Window::DEFAULT_X);
        
        assert_eq!(output.window_y(false), Window::DEFAULT_Y);
        assert_eq!(output.window_y(true), 30);
        assert_ne!(output.window_y(true), Window::DEFAULT_Y);
    }
    
    #[test]
    fn media() {
        // setup
        
        let data = Some(Vec::from([
            String::from(args::MEDIA_PLAYER_ARG),
            String::from("vlc"),
            String::from(args::MEDIA_ICONIFY_ARG),
            String::from("yes"),
            String::from(args::MEDIA_FLAG_ARG),
            String::from("placeholder"),
            String::from(args::MEDIA_TIMEOUT_ARG),
            String::from("30"),
            String::from(args::MEDIA_AUTOSELECT_ARG),
            String::from("yes"),
            String::from(args::MEDIA_LOOKUP_ARG),
            String::from("https://placeholder.com/search?q=%s"),
            String::from(args::MEDIA_BIND_ARG),
            String::from("10.0.0.1"),
        ]));
        
        let flags = &[];
        let pairs = &[];
        
        let args = Args::new(data, flags, pairs);
        
        let cfgpath = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path();
        
        let config = Config::new(&cfgpath).unwrap();
        
        // operation
        
        let output = Params::new(args, config);
        
        // control
        
        assert_eq!(output.media_player(false), Media::DEFAULT_PLAYER);
        assert_eq!(output.media_player(true), "vlc");
        assert_ne!(output.media_player(true), Media::DEFAULT_PLAYER);
        
        assert_eq!(output.media_iconify(false), Media::DEFAULT_ICONIFY);
        assert_eq!(output.media_iconify(true), true);
        assert_ne!(output.media_iconify(true), Media::DEFAULT_ICONIFY);
        
        assert_eq!(output.media_flag(false), Media::DEFAULT_FLAG);
        assert_eq!(output.media_flag(true), "placeholder");
        assert_ne!(output.media_flag(true), Media::DEFAULT_FLAG);
        
        assert_eq!(output.media_timeout(false), Media::DEFAULT_TIMEOUT);
        assert_eq!(output.media_timeout(true), Duration::from_secs(30));
        assert_ne!(output.media_timeout(true), Media::DEFAULT_TIMEOUT);
        
        assert_eq!(output.media_autoselect(false), Media::DEFAULT_AUTOSELECT);
        assert_eq!(output.media_autoselect(true), true);
        assert_ne!(output.media_autoselect(true), Media::DEFAULT_AUTOSELECT);
        
        assert_eq!(output.media_lookup(false), Media::DEFAULT_LOOKUP);
        assert_eq!(output.media_lookup(true), "https://placeholder.com/search?q=%s");
        assert_ne!(output.media_lookup(true), Media::DEFAULT_LOOKUP);
        
        assert_eq!(output.media_bind(false), Media::DEFAULT_BIND);
        assert_eq!(output.media_bind(true), "10.0.0.1");
        assert_ne!(output.media_bind(true), Media::DEFAULT_BIND);
    }
    
    #[test]
    fn paths() {
        // setup
        
        let data = Some(Vec::from([
            String::from(args::PATHS_FILES_ARG),
            String::from("/home/me/files"),
            String::from(args::PATHS_DOWNLOADS_ARG),
            String::from("/home/me/Downloads"),
            String::from(args::PATHS_PIPE_ARG),
            String::from("/testing/pipe"),
            String::from(args::PATHS_DATABASE_ARG),
            String::from("/testing/database"),
        ]));
        
        let flags = &[];
        let pairs = &[];
        
        let args = Args::new(data, flags, pairs);
        
        let cfgpath = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path();
        
        let config = Config::new(&cfgpath).unwrap();
        
        // operation
        
        let output = Params::new(args, config);
        
        // control
        
        assert_eq!(output.paths_files(false), Path::new(Paths::DEFAULT_FILES));
        assert_eq!(output.paths_files(true), Path::new("/home/me/files"));
        assert_ne!(output.paths_files(true), Path::new(Paths::DEFAULT_FILES));
        
        assert_eq!(output.paths_downloads(false), Path::new(Paths::DEFAULT_DOWNLOADS));
        assert_eq!(output.paths_downloads(true), Path::new("/home/me/Downloads"));
        assert_ne!(output.paths_downloads(true), Path::new(Paths::DEFAULT_DOWNLOADS));
        
        assert_eq!(output.paths_pipe(false), Path::new(Paths::DEFAULT_PIPE));
        assert_eq!(output.paths_pipe(true), Path::new("/testing/pipe"));
        assert_ne!(output.paths_pipe(true), Path::new(Paths::DEFAULT_PIPE));
        
        assert_eq!(output.paths_database(false), Path::new(Paths::DEFAULT_DATABASE));
        assert_eq!(output.paths_database(true), Path::new("/testing/database"));
        assert_ne!(output.paths_database(true), Path::new(Paths::DEFAULT_DATABASE));
    }
    
}
