mod stores;
mod menus;
mod window;
mod dialogs;

use gtk::{
    gdk,
    prelude::*,
};

use crate::{
    STYLESHEET,
    FilesSection, WatchlistSection,
};

use stores::Stores;
use window::Window;
use dialogs::Dialogs;
use menus::Menus;

const WINDOW_SPACING: i32 = 6;
const DIALOGS_SPACING: i32 = 15;
const FIELDS_SPACING: i32 = 15;

const SECTIONS_LISTBOX_ROW_WIDTH: i32 = 110;
const SECTIONS_LISTBOX_ROW_HEIGHT: i32 = 40;

pub struct Ui {
    widgets: Widgets,
    clipboard: gtk::Clipboard,
}

pub struct Widgets {
    pub stores: Stores,
    pub menus: Menus,
    pub window: Window,
    pub dialogs: Dialogs,
}

impl Ui {
    
    // ---------- constructors ----------
    
    
    pub fn new(stylesheet_arg: Option<&str>) -> Self {
        let stores =  Stores::new();
        let menus = Menus::new();
        let window = Window::new(&menus);
        let dialogs = Dialogs::new(&window);
        
        let widgets = Widgets {
            stores,
            menus,
            window,
            dialogs,
        };
        
        let provider = gtk::CssProvider::new();
        
        match stylesheet_arg {
            Some(arg) => provider.load_from_path(arg).ok(),
            None => provider.load_from_data(STYLESHEET).ok(),
        };
        
        gtk::StyleContext::add_provider_for_screen(
            &gtk::prelude::GtkWindowExt::screen(&widgets.window.general.window).unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        let clipboard = widgets.window.general.window.clipboard(&gdk::Atom::intern("CLIPBOARD"));
        
        Self {
            widgets,
            clipboard,
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn widgets(&self) -> &Widgets {
        &self.widgets
    }
    
    pub fn clipboard_set_text(&self, text: &str) {
        self.clipboard.set_text(text);
    }
    
    pub fn dialogs_error_show(&self, message: &str) {
        self.widgets.dialogs.general.error.message_label.set_text(message);
        
        let error_dialog = &self.widgets.dialogs.general.error.dialog;
        
        error_dialog.run();
        error_dialog.unrealize();
        error_dialog.hide();
    }
    
    pub fn files_current_treeview(&self) -> Option<&gtk::TreeView> {
        if let Some(selected) = self.widgets.window.files.listbox.selected_row() {
            
            let name = selected.widget_name();
            
            if name == FilesSection::New.display() {
                return Some(&self.widgets.window.files.new_treeview);
            }
            
            if name == FilesSection::Watched.display() {
                return Some(&self.widgets.window.files.watched_treeview)
            }
            
        }
        
        None
    }
    
    pub fn watchlist_current_treeview(&self) -> Option<&gtk::TreeView> {
        if let Some(selected) = self.widgets.window.watchlist.listbox.selected_row() {
            
            let name = selected.widget_name();
            
            if name == WatchlistSection::Watching.display() {
                return Some(&self.widgets.window.watchlist.watching_treeview);
            }
            
            if name == WatchlistSection::OnHold.display() {
                return Some(&self.widgets.window.watchlist.on_hold_treeview);
            }
            
            if name == WatchlistSection::PlanToWatch.display() {
                return Some(&self.widgets.window.watchlist.plan_to_watch_treeview);
            }
            
            if name == WatchlistSection::Completed.display() {
                return Some(&self.widgets.window.watchlist.completed_treeview);
            }
            
        }
        
        None
    }
    
}
