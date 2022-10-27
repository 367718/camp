mod files;
mod watchlist;
mod preferences;

use files::Menus as FilesMenus;
use watchlist::Menus as WatchlistMenus;
use preferences::Menus as PreferencesMenus;

pub struct Menus {
    pub files: FilesMenus,
    pub watchlist: WatchlistMenus,
    pub preferences: PreferencesMenus,
}

impl Menus {
    
    pub fn new() -> Self {
        Self {
            files: FilesMenus::new(),
            watchlist: WatchlistMenus::new(),
            preferences: PreferencesMenus::new(),
        }
    }
    
}
