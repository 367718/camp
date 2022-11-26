use std::{
    error::Error,
    fs::File,
    io::{ Write, BufWriter },
    time::Duration,
};

use crate::Config;

pub struct Media {
    player: Box<str>,
    iconify: bool,
    flag: Box<str>,
    timeout: Duration,
    autoselect: bool,
    lookup: Box<str>,
    bind: Box<str>,
}

pub enum PlayerError {
    Empty,
    Linebreak,
}

pub enum FlagError {
    Empty,
    Linebreak,
}

pub enum TimeoutError {
    Zero,
    Greater,
}

pub enum LookupError {
    Empty,
    Linebreak,
}

pub enum BindError {
    Empty,
    Linebreak,
}

impl Media {
    
    pub const DEFAULT_PLAYER: &'static str = "mpv";
    pub const DEFAULT_ICONIFY: bool = false;
    pub const DEFAULT_FLAG: &'static str = "user.app.flag";
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
    pub const DEFAULT_AUTOSELECT: bool = false;
    pub const DEFAULT_LOOKUP: &'static str = "https://example.com/search?q=%s";
    pub const DEFAULT_BIND: &'static str = "127.0.0.1:7777";
    
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            player: Self::DEFAULT_PLAYER.to_owned().into_boxed_str(),
            iconify: Self::DEFAULT_ICONIFY,
            flag: Self::DEFAULT_FLAG.to_owned().into_boxed_str(),
            timeout: Self::DEFAULT_TIMEOUT,
            autoselect: Self::DEFAULT_AUTOSELECT,
            lookup: Self::DEFAULT_LOOKUP.to_owned().into_boxed_str(),
            bind: Self::DEFAULT_BIND.to_owned().into_boxed_str(),
        }
    }
    
    pub fn serialize(&self, writer: &mut BufWriter<&File>) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "media.player = {}", self.player)?;
        writeln!(writer, "media.iconify = {}", self.iconify)?;
        writeln!(writer, "media.flag = {}", self.flag.as_ref())?;
        writeln!(writer, "media.timeout = {}", self.timeout.as_secs())?;
        writeln!(writer, "media.autoselect = {}", self.autoselect)?;
        writeln!(writer, "media.lookup = {}", self.lookup)?;
        writeln!(writer, "media.bind = {}", self.bind)?;
        
        Ok(())
    }
    
    pub fn deserialize(content: &[u8]) -> Result<(Self, bool), Box<dyn Error>> {
        let mut corrected = false;
        
        let player = Config::get_value(content, b"media.player")?;
        let iconify = Config::get_value(content, b"media.iconify")?;
        let flag = Config::get_value(content, b"media.flag")?;
        let timeout = Config::get_value(content, b"media.timeout")?;
        let autoselect = Config::get_value(content, b"media.autoselect")?;
        let lookup = Config::get_value(content, b"media.lookup")?;
        let bind = Config::get_value(content, b"media.bind")?;
        
        let mut media = Media {
            player: player.to_owned().into_boxed_str(),
            iconify: iconify == "true",
            flag: flag.to_owned().into_boxed_str(),
            timeout: Duration::from_secs(timeout.parse().unwrap_or(0)),
            autoselect: autoselect == "true",
            lookup: lookup.to_owned().into_boxed_str(),
            bind: bind.to_owned().into_boxed_str(),
        };
        
        // player
        if Self::validate_player(&media.player).is_err() {
            media.player = Self::DEFAULT_PLAYER.to_owned().into_boxed_str();
            corrected = true;
        }
        
        // flag
        if Self::validate_flag(&media.flag).is_err() {
            media.flag = Self::DEFAULT_FLAG.to_owned().into_boxed_str();
            corrected = true;
        }
        
        // timeout
        if Self::validate_timeout(media.timeout).is_err() {
            media.timeout = Self::DEFAULT_TIMEOUT;
            corrected = true;
        }
        
        // lookup
        if Self::validate_lookup(&media.lookup).is_err() {
            media.lookup = Self::DEFAULT_LOOKUP.to_owned().into_boxed_str();
            corrected = true;
        }
        
        // bind
        if Self::validate_bind(&media.bind).is_err() {
            media.bind = Self::DEFAULT_BIND.to_owned().into_boxed_str();
            corrected = true;
        }
        
        Ok((media, corrected))
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn player(&self) -> &str {
        self.player.as_ref()
    }
    
    pub fn iconify(&self) -> bool {
        self.iconify
    }
    
    pub fn flag(&self) -> &str {
        &self.flag
    }
    
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    
    pub fn autoselect(&self) -> bool {
        self.autoselect
    }
    
    pub fn lookup(&self) -> &str {
        self.lookup.as_ref()
    }
    
    pub fn bind(&self) -> &str {
        self.bind.as_ref()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn set_player<S: AsRef<str>>(&mut self, player: S) -> Result<bool, Box<dyn Error>> {
        let player = player.as_ref();
        
        if self.player.as_ref() == player {
            return Ok(false);
        }
        
        Self::check_player(player)?;
        
        self.player = player.to_owned().into_boxed_str();
        
        Ok(true)
    }
    
    pub fn set_iconify(&mut self, iconify: bool) -> bool {
        if self.iconify == iconify {
            return false;
        }
        
        self.iconify = iconify;
        
        true
    }
    
    pub fn set_flag<S: AsRef<str>>(&mut self, flag: S) -> Result<bool, Box<dyn Error>> {
        let flag = flag.as_ref();
        
        if self.flag.as_ref() == flag {
            return Ok(false);
        }
        
        Self::check_flag(flag)?;
        
        self.flag = flag.to_owned().into_boxed_str();
        
        Ok(true)
    }
    
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<bool, Box<dyn Error>> {
        if self.timeout == timeout {
            return Ok(false);
        }
        
        Self::check_timeout(timeout)?;
        
        self.timeout = timeout;
        
        Ok(true)
    }
    
    pub fn set_autoselect(&mut self, autoselect: bool) -> bool {
        if self.autoselect == autoselect {
            return false;
        }
        
        self.autoselect = autoselect;
        
        true
    }
    
    pub fn set_lookup<S: AsRef<str>>(&mut self, lookup: S) -> Result<bool, Box<dyn Error>> {
        let lookup = lookup.as_ref();
        
        if self.lookup.as_ref() == lookup {
            return Ok(false);
        }
        
        Self::check_lookup(lookup)?;
        
        self.lookup = lookup.to_owned().into_boxed_str();
        
        Ok(true)
    }
    
    pub fn set_bind<S: AsRef<str>>(&mut self, bind: S) -> Result<bool, Box<dyn Error>> {
        let bind = bind.as_ref();
        
        if self.bind.as_ref() == bind {
            return Ok(false);
        }
        
        Self::check_bind(bind)?;
        
        self.bind = bind.to_owned().into_boxed_str();
        
        Ok(true)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_player(player: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_player(player) {
            match error {
                PlayerError::Empty => return Err("Player: cannot be empty".into()),
                PlayerError::Linebreak => return Err("Player: cannot contain linebreaks".into()),
            }
        }
        
        Ok(())
    }
    
    fn check_flag(flag: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_flag(flag) {
            match error {
                FlagError::Empty => return Err("Flag: cannot be empty".into()),
                FlagError::Linebreak => return Err("Flag: cannot contain linebreaks".into()),
            }
        }
        
        Ok(())
    }
    
    fn check_timeout(timeout: Duration) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_timeout(timeout) {
            match error {
                TimeoutError::Zero => return Err("Timeout: cannot be zero".into()),
                TimeoutError::Greater => return Err("Timeout: cannot be greater than 86_400".into()),
            }
        }
        
        Ok(())
    }
    
    fn check_lookup(lookup: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_lookup(lookup) {
            match error {
                LookupError::Empty => return Err("Lookup: cannot be empty".into()),
                LookupError::Linebreak => return Err("Lookup: cannot contain linebreaks".into()),
            }
        }
        
        Ok(())
    }
    
    fn check_bind(bind: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_bind(bind) {
            match error {
                BindError::Empty => return Err("Bind: cannot be empty".into()),
                BindError::Linebreak => return Err("Bind: cannot contain linebreaks".into()),
            }
        }
        
        Ok(())
    }
    
    pub fn validate_player(player: &str) -> Result<(), PlayerError> {
        if player.is_empty() {
            return Err(PlayerError::Empty);
        }
        
        if player.contains('\n') {
            return Err(PlayerError::Linebreak);
        }
        
        Ok(())
    }
    
    pub fn validate_flag(flag: &str) -> Result<(), FlagError> {
        if flag.is_empty() {
            return Err(FlagError::Empty);
        }
        
        if flag.contains('\n') {
            return Err(FlagError::Linebreak);
        }
        
        Ok(())
    }
    
    pub fn validate_timeout(timeout: Duration) -> Result<Duration, TimeoutError> {
        if timeout.is_zero() {
            return Err(TimeoutError::Zero);
        }
        
        // 24 hours in seconds
        if timeout.as_secs() > 86_400 {
            return Err(TimeoutError::Greater);
        }
        
        Ok(timeout)
    }
    
    pub fn validate_lookup(lookup: &str) -> Result<(), LookupError> {
        if lookup.is_empty() {
            return Err(LookupError::Empty);
        }
        
        if lookup.contains('\n') {
            return Err(LookupError::Linebreak);
        }
        
        Ok(())
    }
    
    pub fn validate_bind(bind: &str) -> Result<(), BindError> {
        if bind.is_empty() {
            return Err(BindError::Empty);
        }
        
        if bind.contains('\n') {
            return Err(BindError::Linebreak);
        }
        
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod player {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let player = Media::DEFAULT_PLAYER;
            
            // operation
            
            let output = Media::validate_player(player);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_player("vlc");
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(media.player(), "vlc");
            
            assert_ne!(media.player(), Media::DEFAULT_PLAYER);
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_player("");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.player(), Media::DEFAULT_PLAYER);
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_player("\nmpv");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.player(), Media::DEFAULT_PLAYER);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_player(Media::DEFAULT_PLAYER);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(media.player(), Media::DEFAULT_PLAYER);
        }
        
    }
    
    mod iconify {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_iconify(true);
            
            // control
            
            assert_eq!(output, true);
            
            assert_eq!(media.iconify(), true);
            
            assert_ne!(media.iconify(), Media::DEFAULT_ICONIFY);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_iconify(Media::DEFAULT_ICONIFY);
            
            // control
            
            assert_eq!(output, false);
            
            assert_eq!(media.iconify(), Media::DEFAULT_ICONIFY);
        }
        
    }
    
    mod flag {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let flag = Media::DEFAULT_FLAG;
            
            // operation
            
            let output = Media::validate_flag(flag);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_flag("user.test.flag");
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(media.flag(), "user.test.flag");
            
            assert_ne!(media.flag(), Media::DEFAULT_FLAG);
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_flag("");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.flag(), Media::DEFAULT_FLAG);
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_flag("user.app\n.flag");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.flag(), Media::DEFAULT_FLAG);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_flag(Media::DEFAULT_FLAG);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(media.flag(), Media::DEFAULT_FLAG);
        }
        
    }
    
    mod timeout {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let timeout = Media::DEFAULT_TIMEOUT;
            
            // operation
            
            let output = Media::validate_timeout(timeout);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_timeout(Duration::from_secs(5));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(media.timeout(), Duration::from_secs(5));
            
            assert_ne!(media.timeout(), Media::DEFAULT_TIMEOUT);
        }
        
        #[test]
        fn invalid_too_small() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_timeout(Duration::from_secs(0));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.timeout(), Media::DEFAULT_TIMEOUT);
        }
        
        #[test]
        fn invalid_too_big() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_timeout(Duration::from_secs(90_000));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.timeout(), Media::DEFAULT_TIMEOUT);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_timeout(Duration::from_secs(15));
            
            // control
            
            assert!(output.is_ok());
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(media.timeout(), Media::DEFAULT_TIMEOUT);
        }
        
    }
    
    mod autoselect {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_autoselect(true);
            
            // control
            
            assert_eq!(output, true);
            
            assert_eq!(media.autoselect(), true);
            
            assert_ne!(media.autoselect(), Media::DEFAULT_AUTOSELECT);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_autoselect(Media::DEFAULT_AUTOSELECT);
            
            // control
            
            assert_eq!(output, false);
            
            assert_eq!(media.autoselect(), Media::DEFAULT_AUTOSELECT);
        }
        
    }
    
    mod lookup {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let lookup = Media::DEFAULT_LOOKUP;
            
            // operation
            
            let output = Media::validate_lookup(lookup);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_lookup("https://testing.com/search?q=%s");
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(media.lookup(), "https://testing.com/search?q=%s");
            
            assert_ne!(media.lookup(), Media::DEFAULT_LOOKUP);
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_lookup("");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.lookup(), Media::DEFAULT_LOOKUP);
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_lookup("https://example.com/search\n?q=%s");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.lookup(), Media::DEFAULT_LOOKUP);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_lookup(Media::DEFAULT_LOOKUP);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(media.lookup(), Media::DEFAULT_LOOKUP);
        }
        
    }
    
    mod bind {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let bind = Media::DEFAULT_BIND;
            
            // operation
            
            let output = Media::validate_bind(bind);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_bind("192.168.0.1:7777");
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(media.bind(), "192.168.0.1:7777");
            
            assert_ne!(media.bind(), Media::DEFAULT_BIND);
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_bind("");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.bind(), Media::DEFAULT_BIND);
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_bind("127.0.\n0.1:7777");
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(media.bind(), Media::DEFAULT_BIND);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut media = Media::new();
            
            // operation
            
            let output = media.set_bind(Media::DEFAULT_BIND);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(media.bind(), Media::DEFAULT_BIND);
        }
        
    }
    
}
