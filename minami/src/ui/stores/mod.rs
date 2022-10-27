mod general;
mod preferences;
mod files;
mod watchlist;

use general::Stores as GeneralStores;
use preferences::Stores as PreferencesStores;
use files::Stores as FilesStores;
use watchlist::Stores as WatchlistStores;

pub struct Stores {
    pub general: GeneralStores,
    pub preferences: PreferencesStores,
    pub files: FilesStores,
    pub watchlist: WatchlistStores,
}

impl Stores {
    
    pub fn new() -> Self {
        Self {
            general: GeneralStores::new(),
            preferences: PreferencesStores::new(),
            files: FilesStores::new(),
            watchlist: WatchlistStores::new(),
        }
    }
    
}
