mod stores;
mod menus;
mod window;
mod dialogs;

use std::env;

use gtk::{
    gdk,
    prelude::*,
};

use crate::{ FilesSection, WatchlistSection };

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
    clipboards: Clipboards,
}

pub struct Widgets {
    pub stores: Stores,
    pub menus: Menus,
    pub window: Window,
    pub dialogs: Dialogs,
}

struct Clipboards {
    main: gtk::Clipboard,
    primary: gtk::Clipboard,
}

impl Ui {
    
    // ---------- constructors ----------
    
    
    pub fn new(stylesheet_arg: Option<&str>) -> Self {
        // ---------- widgets ----------
        
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
            
            // command-line argument
            Some(arg) => {
                
                provider.load_from_path(arg).ok();
                
            },
            
            // executable-adjacent
            None => {
                
                let exec = env::current_exe()
                    .unwrap()
                    .with_file_name("stylesheet.css");
                
                if let Some(exec) = exec.to_str() {
                    provider.load_from_path(exec).ok();
                }
                
            },
            
        }
        
        gtk::StyleContext::add_provider_for_screen(
            &gtk::prelude::GtkWindowExt::screen(&widgets.window.general.window).unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        // ---------- clipboards ----------
        
        // primary allows for middle-click paste while the application is running
        
        let clipboards = Clipboards {
            main: widgets.window.general.window.clipboard(&gdk::Atom::intern("CLIPBOARD")),
            primary: widgets.window.general.window.clipboard(&gdk::Atom::intern("PRIMARY")),
        };
        
        // ---------- struct ----------
        
        Self {
            widgets,
            clipboards,
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn widgets(&self) -> &Widgets {
        &self.widgets
    }
    
    pub fn clipboards_set_text(&self, text: &str) {
        self.clipboards.main.set_text(text);
        self.clipboards.primary.set_text(text);
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
