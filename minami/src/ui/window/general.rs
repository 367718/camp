use gtk::{
    gdk_pixbuf,
    gio,
    glib,
    pango,
    prelude::*,
};

use crate::{ APP_NAME, APP_ICON };

use super::{
    WINDOW_SPACING,
    Menus,
};

pub struct General {
    pub window: gtk::ApplicationWindow,
    
    pub search_entry: gtk::SearchEntry,
    pub search_completion: gtk::EntryCompletion,
    
    pub switchers_box: gtk::Box,
    pub sections_stack: gtk::Stack,
}

impl General {
    
    pub fn new(menus: &Menus) -> Self {
        
        /*
        
        vertical_box
            
            (files_bar_menu)
            (watchlist_bar_menu)
            (preferences_bar_menu)
            
            vertical_box
                
                { search_entry }
                    { search_completion }
                /search_entry
                
                horizontal_box
                    
                    { switchers_box }
                        (files)
                        (watchlist)
                        (preferences)
                    /switchers_box
                    
                    { sections_stack }
                        (files)
                        (watchlist)
                        (preferences)
                    /sections_stack
                    
                /horizontal_box
                
            /vertical_box
            
        /vertical_box
        
        */
        
        // ---------- window ----------
        
        let window = {
            
            gtk::builders::ApplicationWindowBuilder::new()
            .title(APP_NAME)
            .icon(&{
                
                let bytes = glib::Bytes::from_static(APP_ICON);
                let stream = gio::MemoryInputStream::from_bytes(&bytes);
                gdk_pixbuf::Pixbuf::from_stream(&stream, None::<&gio::Cancellable>).unwrap()
                
            })
            .build()
            
        };
        
        // ---------- root box ----------
        
        let root_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        window.add(&root_box);
        
        // ---------- menus ----------
        
        root_box.add(&menus.files.bar.menu);
        root_box.add(&menus.watchlist.bar.menu);
        root_box.add(&menus.preferences.bar.menu);
        
        // ---------- main box ----------
        
        let main_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .spacing(WINDOW_SPACING)
            .build()
            
        };
        
        root_box.add(&main_box);
        
        // ---------- search entry ----------
        
        let search_entry = {
            
            gtk::builders::SearchEntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .placeholder_text("Search files and watchlist")
            .build()
            
        };
        
        search_entry.style_context().add_class("completion");
        
        main_box.add(&search_entry);
        
        // ---------- search completion ----------
        
        let search_completion = Self::build_completion();
        
        search_entry.set_completion(Some(&search_completion));
        
        // ---------- sections box ----------
        
        let sections_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(WINDOW_SPACING)
            .build()
            
        };
        
        main_box.add(&sections_box);
        
        // ---------- switchers_box ----------
        
        let switchers_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .spacing(WINDOW_SPACING)
            .build()
            
        };
        
        sections_box.add(&switchers_box);
        
        // ---------- stack box ----------
        
        let stack_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        sections_box.add(&stack_box);
        
        // ---------- sections_stack ----------
        
        let sections_stack = {
            
            gtk::builders::StackBuilder::new()
            .visible(true)
            .transition_duration(0)
            .build()
            
        };
        
        stack_box.add(&sections_stack);
        
        // ---------- return ----------
        
        Self {
            window,
            
            search_entry,
            search_completion,
            
            switchers_box,
            sections_stack,
        }
        
    }
    
    fn build_completion() -> gtk::EntryCompletion {
        // ---------- completion ----------
        
        let completion = {
            
            gtk::builders::EntryCompletionBuilder::new()
                .text_column(2)
                .build()
            
        };
        
        // ---------- render ----------
        
        let cell = gtk::CellRendererText::new();
        cell.set_ellipsize(pango::EllipsizeMode::Middle);
        
        completion.pack_end(&cell, true);
        completion.add_attribute(&cell, "text", 2);
        
        // ---------- match function ----------
        
        // since model content will be set based on input, do not apply filter
        completion.set_match_func(|_, _, _| true);
        
        // ---------- return ----------
        
        completion
    }
    
}
