mod general;
mod preferences;
mod files;
mod watchlist;

use super::{
    DIALOGS_SPACING, FIELDS_SPACING,
    Window,
};

use general::Dialogs as GeneralDialogs;
use preferences::Dialogs as PreferencesDialogs;
use files::Dialogs as FilesDialogs;
use watchlist::Dialogs as WatchlistDialogs;

pub struct Dialogs {
    pub general: GeneralDialogs,
    pub preferences: PreferencesDialogs,
    pub files: FilesDialogs,
    pub watchlist: WatchlistDialogs,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            general: GeneralDialogs::new(window),
            preferences: PreferencesDialogs::new(window),
            files: FilesDialogs::new(window),
            watchlist: WatchlistDialogs::new(window),
        }
    }
    
}

fn build_main_box(orientation: gtk::Orientation) -> gtk::Box {
    gtk::builders::BoxBuilder::new()
    .visible(true)
    .orientation(orientation)
    .spacing(DIALOGS_SPACING)
    .build()
}
