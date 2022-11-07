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
    pub config_load_error: ConfigLoadError,
    pub config_save_error: ConfigSaveError,
    pub database_load_error: DatabaseLoadError,
    pub database_save_error: DatabaseSaveError,
    pub error: Error,
    pub chooser: Chooser,
    pub delete: Delete,
}

pub struct ConfigLoadError {
    pub dialog: gtk::Dialog,
    pub path_label: gtk::Label,
    pub message_label: gtk::Label,
}

pub struct ConfigSaveError {
    pub dialog: gtk::Dialog,
    pub path_label: gtk::Label,
    pub message_label: gtk::Label,
}

pub struct DatabaseLoadError {
    pub dialog: gtk::Dialog,
    pub path_label: gtk::Label,
    pub message_label: gtk::Label,
}

pub struct DatabaseSaveError {
    pub dialog: gtk::Dialog,
    pub path_label: gtk::Label,
    pub message_label: gtk::Label,
}

pub struct Error {
    pub dialog: gtk::Dialog,
    pub message_label: gtk::Label,
}

pub struct Chooser {
    pub dialog: gtk::FileChooserNative,
}

pub struct Delete {
    pub dialog: gtk::Dialog,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            config_load_error: ConfigLoadError::new(),
            config_save_error: ConfigSaveError::new(window),
            database_load_error: DatabaseLoadError::new(),
            database_save_error: DatabaseSaveError::new(window),
            error: Error::new(),
            chooser: Chooser::new(),
            delete: Delete::new(window),
        }
    }
    
}

impl ConfigLoadError {
    
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
            
            button ("Generate new", gtk::ResponseType::Ok)
            button ("Exit", gtk::ResponseType::Cancel)
            
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
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("The configuration file could not be loaded.")
                .xalign(0.0)
                .build()
            );
            
        }
        
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
        
        // ---------- message ----------
        
        let message_box = {
            
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
        
        main_box.add(&message_box);
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        message_label.style_context().add_class("weight-bold");
        
        message_box.add(&message_label);
        
        // ---------- buttons ----------
        
        let generate_button = dialog.add_button("Generate new", gtk::ResponseType::Ok);
        let exit_button = dialog.add_button("Exit", gtk::ResponseType::Cancel);
        
        generate_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        exit_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            path_label,
            message_label,
        }
        
    }
    
}

impl ConfigSaveError {
    
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
            .title("Configuration save error")
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
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("The configuration file could not be saved.")
                .xalign(0.0)
                .build()
            );
            
        }
        
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
        
        // ---------- message ----------
        
        let message_box = {
            
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
        
        main_box.add(&message_box);
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        message_label.style_context().add_class("weight-bold");
        
        message_box.add(&message_label);
        
        // ---------- buttons ----------
        
        let try_again_button = dialog.add_button("Try again", gtk::ResponseType::Ok);
        let give_up_button = dialog.add_button("Give up", gtk::ResponseType::Cancel);
        
        try_again_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        give_up_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            path_label,
            message_label,
        }
        
    }
    
}

impl DatabaseLoadError {
    
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
            button ("Exit", gtk::ResponseType::Cancel)
            
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
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("The database file could not be loaded.")
                .xalign(0.0)
                .build()
            );
            
        }
        
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
        
        // ---------- message ----------
        
        let message_box = {
            
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
        
        main_box.add(&message_box);
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        message_label.style_context().add_class("weight-bold");
        
        message_box.add(&message_label);
        
        // ---------- buttons ----------
        
        let generate_button = dialog.add_button("Generate new", gtk::ResponseType::Other(0));
        dialog.add_button("Select another", gtk::ResponseType::Other(1));
        let exit_button = dialog.add_button("Exit", gtk::ResponseType::Cancel);
        
        generate_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        exit_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Other(0));
        
        // ---------- return ----------
        
        Self {
            dialog,
            path_label,
            message_label,
        }
        
    }
    
}

impl DatabaseSaveError {
    
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
            .title("Database save error")
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
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("The database file could not be saved.")
                .xalign(0.0)
                .build()
            );
            
        }
        
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
        
        // ---------- message ----------
        
        let message_box = {
            
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
        
        main_box.add(&message_box);
        
        let message_label = {
            
            gtk::Label::builder()
            .visible(true)
            .wrap(true)
            .build()
            
        };
        
        message_label.style_context().add_class("weight-bold");
        
        message_box.add(&message_label);
        
        // ---------- buttons ----------
        
        let try_again_button = dialog.add_button("Try again", gtk::ResponseType::Ok);
        let give_up_button = dialog.add_button("Give up", gtk::ResponseType::Cancel);
        
        try_again_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        give_up_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            path_label,
            message_label,
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

impl Chooser {
    
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
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Cancel);
        
        // ---------- return ----------
        
        Self {
            dialog,
        }
        
    }
    
}
