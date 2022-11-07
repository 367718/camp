use gtk::{
    glib,
    pango,
    prelude::*,
};

use super::{
    DIALOGS_SPACING, FIELDS_SPACING,
    Window,
};

pub struct Dialogs {
    pub rename: Rename,
    pub move_to_folder: MoveToFolder,
    pub maintenance: Maintenance,
    pub job: Job,
}

pub struct Rename {
    pub dialog: gtk::Dialog,
    pub current_label: gtk::Label,
    pub new_entry: gtk::Entry,
}

pub struct MoveToFolder {
    pub dialog: gtk::Dialog,
    pub folder_entry: gtk::Entry,
    pub folder_completion: gtk::EntryCompletion,
}

pub struct Maintenance {
    pub dialog: gtk::Dialog,
}

pub struct Job {
    pub dialog: gtk::Dialog,
    pub progress_textview: gtk::TextView,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            rename: Rename::new(window),
            move_to_folder: MoveToFolder::new(window),
            maintenance: Maintenance::new(window),
            job: Job::new(),
        }
    }
    
}

impl Rename {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                ----- current -----
                
                horizontal_box
                    static_label
                    { current_label }
                /horizontal_box
                
                ----- new -----
                
                horizontal_box
                    static_label
                    { new_entry }
                /horizontal_box
                
                static_label
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title("Rename")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(750)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();        
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- current ----------
        
        let current_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Current name:")
                .xalign(1.0)
                .width_chars(12)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&current_box);
        
        let current_label = {
            
            gtk::Label::builder()
            .visible(true)
            .selectable(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build()
            
        };
        
        current_box.add(&current_label);
        
        // ---------- new ----------
        
        let new_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("New name:")
                .xalign(1.0)
                .width_chars(12)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&new_box);
        
        let new_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        new_box.add(&new_entry);
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("File extension will not be changed.")
                .xalign(0.0)
                .sensitive(false)
                .build()
            );
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            current_label,
            new_entry,
        }
        
    }
    
}

impl MoveToFolder {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                horizontal_box
                    static_label
                    { folder_entry }
                /horizontal_box
                
                static_label
                static_label
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title("Move to folder")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(575)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- folder ----------
        
        let folder_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Folder name:")
                .xalign(0.0)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&folder_box);
        
        let folder_entry = {
            
            // this could have been a SearchEntry, but it does not look like the primary icon can be disabled once set
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        folder_entry.style_context().add_class("completion");
        
        folder_box.add(&folder_entry);
        
        // ---------- completion ----------
        
        let folder_completion = Self::build_completion();
        
        folder_entry.set_completion(Some(&folder_completion));
        
        // ---------- static labels ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("Directory will be created if it doesn't exist.")
                .xalign(0.0)
                .sensitive(false)
                .build()
            );
            
        }
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("Files will be moved to root if no name is provided.")
                .xalign(0.0)
                .sensitive(false)
                .build()
            );
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            folder_entry,
            folder_completion,
        }
        
    }
    
    fn build_completion() -> gtk::EntryCompletion {
        // ---------- completion ----------
        
        let completion = {
            
            gtk::EntryCompletion::builder()
                .text_column(0)
                .build()
            
        };
        
        // ---------- render ----------
        
        let cell = gtk::CellRendererText::new();
        cell.set_ellipsize(pango::EllipsizeMode::Middle);
        
        completion.pack_end(&cell, true);
        completion.add_attribute(&cell, "text", 0);
        
        // ---------- match function ----------
        
        completion.set_match_func(|comp, query, iter| {
            
            let model = comp.model().unwrap();
            let name = model.value(iter, 0).get::<glib::GString>().unwrap();
            
            crate::general::case_insensitive_contains(&name, query)
            
        });
        
        // ---------- return ----------
        
        completion
    }
    
}

impl Maintenance {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                static_label
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title("Perform maintenance")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(500)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- static labels ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .use_markup(true)
                .label(r#"This process will <b>permanently delete</b> every file marked as "Updated" or considered irrelevant and every directory considered empty."#)
                .xalign(0.0)
                .build()
            );
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Cancel);
        
        // ---------- return ----------
        
        Self {
            dialog,
        }
        
    }
    
}

impl Job {
    
    fn new() -> Self {
        
        /*
        
        content_area
            
            scrolled_window
                { progress_textview }
            /scrolled_window
            
        /content_area
        
        action_area
            
            button ("Close", gtk::ResponseType::Close)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .window_position(gtk::WindowPosition::Center)
            .default_width(825)
            .default_height(650)
            .deletable(false)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();        
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- progress scrolled ----------
        
        let progress_scrolled = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        content_area.add(&progress_scrolled);
        
        // ---------- progress textview ----------
        
        let progress_textview = {
            
            gtk::TextView::builder()
            .visible(true)
            .editable(false)
            .left_margin(6)
            .right_margin(6)
            .top_margin(6)
            .bottom_margin(6)
            .monospace(true)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .build()
            
        };
        
        progress_scrolled.add(&progress_textview);
        
        // ---------- buttons ----------
        
        dialog.add_button("Close", gtk::ResponseType::Close);
        dialog.set_default_response(gtk::ResponseType::Close);
        
        // ---------- return ----------
        
        Self {
            dialog,
            progress_textview,
        }
        
    }
    
}
