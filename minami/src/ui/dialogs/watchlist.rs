use gtk::prelude::*;

use crate::SeriesStatus;

use super::{
    DIALOGS_SPACING, FIELDS_SPACING,
    Window,
};

pub struct Dialogs {
    pub series: Series,
}

pub struct Series {
    pub dialog: gtk::Dialog,
    pub title_entry: gtk::Entry,
    pub kind_combo: gtk::ComboBox,
    pub status_combo: gtk::ComboBoxText,
    pub progress_spin: gtk::SpinButton,
    pub good_switch: gtk::Switch,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            series: Series::new(window),
        }
    }
    
}

impl Series {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                ----- title -----
                
                horizontal_box
                    static_label
                    { title_entry }
                    button (lookup title -> Other(2))
                /horizontal_box
                
                ----- kind -----
                
                horizontal_box
                    static_label
                    { kind_combo }
                /horizontal_box
                
                ----- status -----
                
                horizontal_box
                    static_label
                    { status_combo }
                    image
                /horizontal_box
                
                ----- progress -----
                
                horizontal_box
                    static_label
                    { progress_spin }
                /horizontal_box
                
                ----- good -----
                
                horizontal_box
                    static_label
                    { good_switch }
                /horizontal_box
                
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Other(0))
            button ("Confirm and add another", gtk::ResponseType::Other(1))
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(650)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- fields ----------
        
        let (title_box, title_entry) = Self::build_title();
        let (kind_box, kind_combo) = Self::build_kind();
        let (status_box, status_combo) = Self::build_status();
        let (progress_box, progress_spin) = Self::build_progress();
        let (good_box, good_switch) = Self::build_good();
        
        {
            
            main_box.add(&title_box);
            main_box.add(&kind_box);
            main_box.add(&status_box);
            main_box.add(&progress_box);
            main_box.add(&good_box);
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Other(0));
        
        dialog.add_button("Confirm and add another", gtk::ResponseType::Other(1));
        
        // lookup title (in content area)
        if let Some(parent) = confirm_button.parent() {
            if let Some(button_box) = parent.downcast_ref::<gtk::ButtonBox>() {
                let title_button = gtk::builders::ButtonBuilder::new()
                    .visible(true)
                    .image(&gtk::Image::from_icon_name(Some("edit-find-symbolic"), gtk::IconSize::Menu))
                    .tooltip_text("Lookup title")
                    .build();
                
                dialog.add_action_widget(&title_button, gtk::ResponseType::Other(2));
                
                button_box.remove(&title_button);
                title_box.add(&title_button);
            }
        }
        
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Other(0));
        
        // ---------- return ----------
        
        Self {
            dialog,
            
            title_entry,
            kind_combo,
            status_combo,
            progress_spin,
            good_switch,
        }
        
    }
    
    fn build_field_box(text: &str) -> gtk::Box {
        gtk::builders::BoxBuilder::new()
        .visible(true)
        .orientation(gtk::Orientation::Horizontal)
        .spacing(FIELDS_SPACING)
        .child(&{
            
            gtk::builders::LabelBuilder::new()
            .visible(true)
            .label(text)
            .xalign(1.0)
            .width_chars(8)
            .build()
            
        })
        .build()
    }
    
    fn build_title() -> (gtk::Box, gtk::Entry) {
        let title_box = Self::build_field_box("Title:");
        
        let title_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        title_box.add(&title_entry);
        
        (title_box, title_entry)
    }
    
    fn build_kind() -> (gtk::Box, gtk::ComboBox) {
        let kind_box = Self::build_field_box("Kind:");
        
        let kind_combo = {
            
            gtk::builders::ComboBoxBuilder::new()
            .visible(true)
            .width_request(175)
            .id_column(2)
            .build()
            
        };
        
        kind_box.add(&kind_combo);
        
        let name_cell = gtk::CellRendererText::new();
        kind_combo.pack_start(&name_cell, true);
        kind_combo.add_attribute(&name_cell, "text", 1);
        
        (kind_box, kind_combo)
    }
    
    fn build_status() -> (gtk::Box, gtk::ComboBoxText) {
        let status_box = Self::build_field_box("Status:");
        
        let status_combo = {
            
            gtk::builders::ComboBoxTextBuilder::new()
            .visible(true)
            .width_request(175)
            .build()
            
        };
        
        status_box.add(&status_combo);
        
        for status in SeriesStatus::iter() {
            status_combo.append(Some(&status.as_int().to_string()), status.display());
        }
        
        {
            
            status_box.add(
                &gtk::builders::ImageBuilder::new()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text("If the status is changed, any candidate related to this series will be deleted")
                .build()
            );
            
        }
        
        (status_box, status_combo)
    }
    
    fn build_progress() -> (gtk::Box, gtk::SpinButton) {
        let progress_box = Self::build_field_box("Progress:");
        
        let progress_spin = {
            
            gtk::builders::SpinButtonBuilder::new()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::builders::AdjustmentBuilder::new()
                .upper(99999.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        progress_box.add(&progress_spin);
        
        (progress_box, progress_spin)
    }
    
    fn build_good() -> (gtk::Box, gtk::Switch) {
        let good_box = Self::build_field_box("Good:");
        
        let good_switch = {
            
            gtk::builders::SwitchBuilder::new()
            .visible(true)
            .build()
            
        };
        
        good_box.add(&good_switch);
        
        (good_box, good_switch)
    }
    
}
