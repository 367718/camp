use gtk::{
    gdk_pixbuf,
    gio,
    glib,
    pango,
    prelude::*,
};

use crate::{ APP_NAME, APP_ICON };

use super::{
    DIALOGS_SPACING, FIELDS_SPACING,
    Window,
};

pub struct Dialogs {
    pub error: Error,
    pub delete: Delete,
    pub file_load_error: FileLoadError,
    pub file_save_error: FileSaveError,
    pub file_chooser: FileChooser,
}

pub struct Error {
    pub dialog: gtk::Dialog,
    pub message_label: gtk::Label,
}

pub struct Delete {
    pub dialog: gtk::Dialog,
}

pub struct FileLoadError {
    pub dialog: gtk::Dialog,
    pub message_label: gtk::Label,
    pub path_label: gtk::Label,
    pub error_label: gtk::Label,
}

pub struct FileSaveError {
    pub dialog: gtk::Dialog,
    pub message_label: gtk::Label,
    pub path_label: gtk::Label,
    pub error_label: gtk::Label,
}

pub struct FileChooser {
    pub dialog: gtk::FileChooserNative,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            error: Error::new(),
            delete: Delete::new(window),
            file_load_error: FileLoadError::new(),
            file_save_error: FileSaveError::new(window),
            file_chooser: FileChooser::new(),
        }
    }
    
}

impl Error {
    
    fn new() -> Self {
        
        /*
        
        content_area
            
            vertical_box
                static_label
                { message_label }
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Close", gtk::ResponseType::Close)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title(APP_NAME)
            .icon(&{
                
                let bytes = glib::Bytes::from_static(APP_ICON);
                let stream = gio::MemoryInputStream::from_bytes(&bytes);
                gdk_pixbuf::Pixbuf::from_stream(&stream, None::<&gio::Cancellable>).unwrap()
                
            })
            .window_position(gtk::WindowPosition::Center)
            .default_width(400)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("The following errors were encountered:")
                .xalign(0.0)
                .build()
            );
            
        }
        
        // ---------- message label ----------
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .xalign(0.0)
            .wrap(true)
            .build()
            
        };
        
        main_box.add(&message_label);
        
        // ---------- buttons ----------
        
        dialog.add_button("Close", gtk::ResponseType::Close);
        dialog.set_default_response(gtk::ResponseType::Close);
        
        // ---------- return ----------
        
        Self {
            dialog,
            message_label,
        }
        
    }
    
}

impl Delete {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
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
            .title("Delete elements")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(475)
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
                .label("Are you sure you want to permanently delete the selected elements?")
                .xalign(0.0)
                .build()
            );
            
        }
        
        {
            
            main_box.add(&{
                
                let label = gtk::Label::builder()
                    .visible(true)
                    .label("This action cannot be undone.")
                    .xalign(0.0)
                    .build();
                
                label.style_context().add_class("weight-bold");
                
                label
                
            });
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Cancel);
        
        // ---------- return ----------
        
        Self {
            dialog,
        }
        
    }
    
}

impl FileLoadError {
    
    fn new() -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                static_label
                
                ----- path -----
                
                horizontal_box
                    static_label
                    { path_label }
                /horizontal_box
                
                ----- message -----
                
                horizontal_box
                    static_label
                    { message_label }
                /horizontal_box
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Generate new", gtk::ResponseType::Other(0))
            button ("Select another", gtk::ResponseType::Other(1))
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title(APP_NAME)
            .icon(&{
                
                let bytes = glib::Bytes::from_static(APP_ICON);
                let stream = gio::MemoryInputStream::from_bytes(&bytes);
                gdk_pixbuf::Pixbuf::from_stream(&stream, None::<&gio::Cancellable>).unwrap()
                
            })
            .window_position(gtk::WindowPosition::Center)
            .default_width(600)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- message ----------
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .label("A file could not be loaded.")
            .xalign(0.0)
            .build()
            
        };
        
        main_box.add(&message_label);
        
        // ---------- path ----------
        
        let path_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Path:")
                .xalign(1.0)
                .width_chars(7)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&path_box);
        
        let path_label = {
            
            gtk::Label::builder()
            .visible(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build()
            
        };
        
        path_label.style_context().add_class("weight-bold");
        
        path_box.add(&path_label);
        
        // ---------- error ----------
        
        let error_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Error:")
                .xalign(1.0)
                .width_chars(7)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&error_box);
        
        let error_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        error_label.style_context().add_class("weight-bold");
        
        error_box.add(&error_label);
        
        // ---------- buttons ----------
        
        let generate_button = dialog.add_button("Generate new", gtk::ResponseType::Other(0));
        dialog.add_button("Select another", gtk::ResponseType::Other(1));
        let exit_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        generate_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        exit_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Other(0));
        
        // ---------- return ----------
        
        Self {
            dialog,
            message_label,
            path_label,
            error_label,
        }
        
    }
    
}

impl FileSaveError {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                static_label
                
                ----- path -----
                
                horizontal_box
                    static_label
                    { path_label }
                /horizontal_box
                
                ----- message -----
                
                horizontal_box
                    static_label
                    { message_label }
                /horizontal_box
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Try again", gtk::ResponseType::Ok)
            button ("Give up", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
            .title("File save error")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(600)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- message ----------
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .label("A file could not be saved.")
            .xalign(0.0)
            .build()
            
        };
        
        main_box.add(&message_label);
        
        // ---------- path ----------
        
        let path_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Path:")
                .xalign(1.0)
                .width_chars(7)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&path_box);
        
        let path_label = {
            
            gtk::Label::builder()
            .visible(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build()
            
        };
        
        path_label.style_context().add_class("weight-bold");
        
        path_box.add(&path_label);
        
        // ---------- error ----------
        
        let error_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label("Error:")
                .xalign(1.0)
                .width_chars(7)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&error_box);
        
        let error_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        error_label.style_context().add_class("weight-bold");
        
        error_box.add(&error_label);
        
        // ---------- buttons ----------
        
        let try_again_button = dialog.add_button("Try again", gtk::ResponseType::Ok);
        let give_up_button = dialog.add_button("Give up", gtk::ResponseType::Cancel);
        
        try_again_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        give_up_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            message_label,
            path_label,
            error_label,
        }
        
    }
    
}

impl FileChooser {
    
    fn new() -> Self {
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::FileChooserNative::new(
                None,
                None::<&gtk::Window>,
                gtk::FileChooserAction::Open,
                Some("Confirm"),
                Some("Cancel"),
            )
            
        };
        
        // ---------- return ----------
        
        Self {
            dialog,
        }
        
    }
    
}
