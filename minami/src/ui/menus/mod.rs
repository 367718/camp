mod files;
mod watchlist;
mod preferences;

use files::Files;
use watchlist::Watchlist;
use preferences::Preferences;

pub struct Menus {
    pub files: Files,
    pub watchlist: Watchlist,
    pub preferences: Preferences,
}

impl Menus {
    
    pub fn new() -> Self {
        Self {
            files: Files::new(),
            watchlist: Watchlist::new(),
            preferences: Preferences::new(),
        }
    }
    
}
