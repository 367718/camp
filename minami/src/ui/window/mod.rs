mod general;
mod files;
mod watchlist;
mod preferences;

use gtk::prelude::*;

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

fn build_buttons_box() -> gtk::Box {
    gtk::Box::builder()
    .visible(true)
    .homogeneous(true)
    .spacing(WINDOW_SPACING)
    .halign(gtk::Align::Start)
    .orientation(gtk::Orientation::Horizontal)
    .build()
}

fn build_button(label: &str, action_name: &str, class: Option<&str>) -> gtk::Button {
    let button = gtk::Button::builder()
        .visible(true)
        .child(&{
            
            gtk::Label::builder()
            .visible(true)
            .label(label)
            .xalign(0.5)
            .width_chars(7)
            .build()
            
        })
        .action_name(action_name)
        .build();
    
    if let Some(class) = class {
        button.style_context().add_class(class);
    }
    
    button
}
