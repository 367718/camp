mod general;
mod files;
mod watchlist;
mod preferences;

use super::{
    WINDOW_SPACING, FIELDS_SPACING,
    SECTIONS_LISTBOX_ROW_WIDTH, SECTIONS_LISTBOX_ROW_HEIGHT,
    Menus,
};

use general::General;
use files::Files;
use watchlist::Watchlist;
use preferences::Preferences;

pub struct Window {
    pub general: General,
    pub files: Files,
    pub watchlist: Watchlist,
    pub preferences: Preferences,
}

impl Window {
    
    pub fn new(menus: &Menus) -> Self {
        let general = General::new(menus);
        let files = Files::new(&general);
        let watchlist = Watchlist::new(&general);
        let preferences = Preferences::new(&general);
        
        Self {
            general,
            files,
            watchlist,
            preferences,
        }
    }
    
}