use gtk::{
    pango,
    prelude::*,
};

use crate::WatchlistSection;

use super::{
    DIALOGS_SPACING, FIELDS_SPACING,
    Window,
};

pub struct Dialogs {
    pub candidates: Candidates,
    pub candidates_series: CandidatesSeries,
    pub candidates_downloaded: CandidatesDownloaded,
    pub feeds: Feeds,
    pub kinds: Kinds,
    pub formats: Formats,
}

pub struct Candidates {
    pub dialog: gtk::Dialog,
    pub title_entry: gtk::Entry,
    pub group_entry: gtk::Entry,
    pub quality_entry: gtk::Entry,
    pub series_entry: gtk::Entry,
    pub offset_spin: gtk::SpinButton,
    pub current_switch: gtk::Switch,
    pub downloaded_spin: gtk::SpinButton,
}

pub struct CandidatesSeries {
    pub dialog: gtk::Dialog,
    pub notebook: gtk::Notebook,
    pub watching_treeview: gtk::TreeView,
    pub on_hold_treeview: gtk::TreeView,
    pub plan_to_watch_treeview: gtk::TreeView,
}

pub struct CandidatesDownloaded {
    pub dialog: gtk::Dialog,
    pub title_label: gtk::Label,
    pub download_spin: gtk::SpinButton,
}

pub struct Feeds {
    pub dialog: gtk::Dialog,
    pub url_entry: gtk::Entry,
}

pub struct Kinds {
    pub dialog: gtk::Dialog,
    pub name_entry: gtk::Entry,
}

pub struct Formats {
    pub dialog: gtk::Dialog,
    pub name_entry: gtk::Entry,
}

impl Dialogs {
    
    pub fn new(window: &Window) -> Self {
        Self {
            candidates: Candidates::new(window),
            candidates_series: CandidatesSeries::new(window),
            candidates_downloaded: CandidatesDownloaded::new(window),
            feeds: Feeds::new(window),
            kinds: Kinds::new(window),
            formats: Formats::new(window),
        }
    }
    
}

impl Candidates {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                ----- title -----
                
                horizontal_box
                    static_label
                    { title_entry }
                    image
                /horizontal_box
                
                ----- group -----
                
                horizontal_box
                    static_label
                    { group_entry }
                    image
                /horizontal_box
                
                ----- quality -----
                
                horizontal_box
                    static_label
                    { quality_entry }
                    image
                /horizontal_box
                
                ----- series -----
                
                horizontal_box
                    static_label
                    { series_entry }
                    button (select_series -> Other(1))
                /horizontal_box
                
                ----- offset -----
                
                horizontal_box
                    static_label
                    { offset_spin }
                /horizontal_box
                
                ----- current -----
                
                horizontal_box
                    static_label
                    { current_switch }
                /horizontal_box
                
                ----- downloaded -----
                
                horizontal_box
                    static_label
                    { downloaded_spin }
                /horizontal_box
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Other(0))
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
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
        
        // ---------- fields ----------
        
        let (title_box, title_entry) = Self::build_title();
        let (group_box, group_entry) = Self::build_group();
        let (quality_box, quality_entry) = Self::build_quality();
        let (series_box, series_entry) = Self::build_series();
        let (offset_box, offset_spin) = Self::build_offset();
        let (current_box, current_switch) = Self::build_current();
        let (downloaded_box, downloaded_spin) = Self::build_downloaded();
        
        {
            
            main_box.add(&title_box);
            main_box.add(&group_box);
            main_box.add(&quality_box);
            main_box.add(&series_box);
            main_box.add(&offset_box);
            main_box.add(&current_box);
            main_box.add(&downloaded_box);
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Other(0));
        
        // select series (in content area)
        if let Some(parent) = confirm_button.parent() {
            if let Some(button_box) = parent.downcast_ref::<gtk::ButtonBox>() {
                let series_button = gtk::builders::ButtonBuilder::new()
                    .visible(true)
                    .image(&gtk::Image::from_icon_name(Some("open-menu-symbolic"), gtk::IconSize::Menu))
                    .tooltip_text("Select series")
                    .build();
                
                dialog.add_action_widget(&series_button, gtk::ResponseType::Other(1));
                
                button_box.remove(&series_button);
                series_box.add(&series_button);
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
            group_entry,
            quality_entry,
            series_entry,
            offset_spin,
            current_switch,
            downloaded_spin,
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
            .width_chars(12)
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
        
        {
            
            title_box.add(
                &gtk::builders::ImageBuilder::new()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text("Folders will be included when evaluating files for updates")
                .build()
            );
            
        }
        
        (title_box, title_entry)
    }
    
    fn build_group() -> (gtk::Box, gtk::Entry) {
        let group_box = Self::build_field_box("Group:");
        
        let group_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .width_chars(35)
            .build()
            
        };
        
        group_box.add(&group_entry);
        
        {
            
            group_box.add(
                &gtk::builders::ImageBuilder::new()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text(r#"A blank group means "match any""#)
                .build()
            );
            
        }
        
        (group_box, group_entry)
    }
    
    fn build_quality() -> (gtk::Box, gtk::Entry) {
        let quality_box = Self::build_field_box("Quality:");
        
        let quality_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .width_chars(35)
            .build()
            
        };
        
        quality_box.add(&quality_entry);
        
        {
            
            quality_box.add(
                &gtk::builders::ImageBuilder::new()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text(r#"A blank quality means "match any""#)
                .build()
            );
            
        }
        
        (quality_box, quality_entry)
    }
    
    fn build_series() -> (gtk::Box, gtk::Entry) {
        let series_box = Self::build_field_box("Series:");
        
        let series_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .sensitive(false)
            .build()
            
        };
        
        series_box.add(&series_entry);
        
        (series_box, series_entry)
    }
    
    fn build_offset() -> (gtk::Box, gtk::SpinButton) {
        let offset_box = Self::build_field_box("Offset:");
        
        let offset_spin = {
            
            gtk::builders::SpinButtonBuilder::new()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::builders::AdjustmentBuilder::new()
                .upper(9999.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        offset_box.add(&offset_spin);
        
        (offset_box, offset_spin)
    }
    
    fn build_current() -> (gtk::Box, gtk::Switch) {
        let current_box = Self::build_field_box("Current:");
        
        let current_switch = {
            
            gtk::builders::SwitchBuilder::new()
            .visible(true)
            .build()
            
        };
        
        current_box.add(&current_switch);
        
        (current_box, current_switch)
    }
    
    fn build_downloaded() -> (gtk::Box, gtk::SpinButton) {
        let downloaded_box = Self::build_field_box("Downloaded:");
        
        let downloaded_spin = {
            
            gtk::builders::SpinButtonBuilder::new()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::builders::AdjustmentBuilder::new()
                .upper(99_999.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        downloaded_box.add(&downloaded_spin);
        
        (downloaded_box, downloaded_spin)
    }
    
}

impl CandidatesSeries {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            { notebook }
                
                scrolled_window
                    { watching_treeview }
                /scrolled_window
                
                scrolled_window
                    { on_hold_treeview }
                /scrolled_window
                
                scrolled_window
                    { plan_to_watch_treeview }
                /scrolled_window
                
            /notebook
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Other(0))
            button ("Add new", gtk::ResponseType::Other(1))
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .title("Select series")
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(750)
            .default_height(650)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- notebook ----------
        
        let notebook = {
            
            gtk::builders::NotebookBuilder::new()
            .visible(true)
            .show_border(false)
            .build()
            
        };
        
        content_area.add(&notebook);
        
        // ---------- treeviews ----------
        
        let (watching_scrolled, watching_treeview) = Self::build_treeview();
        let (on_hold_scrolled, on_hold_treeview) = Self::build_treeview();
        let (plan_to_watch_scrolled, plan_to_watch_treeview) = Self::build_treeview();
        
        {
            
            notebook.append_page(
                &watching_scrolled,
                Some(&{
                    
                    gtk::builders::LabelBuilder::new()
                    .visible(true)
                    .label(WatchlistSection::Watching.display())
                    .width_chars(12)
                    .build()
                    
                })
            );
            
            notebook.append_page(
                &on_hold_scrolled,
                Some(&{
                    
                    gtk::builders::LabelBuilder::new()
                    .visible(true)
                    .label(WatchlistSection::OnHold.display())
                    .width_chars(12)
                    .build()
                    
                })
            );
            
            notebook.append_page(
                &plan_to_watch_scrolled,
                Some(&{
                    
                    gtk::builders::LabelBuilder::new()
                    .visible(true)
                    .label(WatchlistSection::PlanToWatch.display())
                    .width_chars(12)
                    .build()
                    
                })
            );
            
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Other(0));
        dialog.add_button("Add new", gtk::ResponseType::Other(1));
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Other(0));
        
        // ---------- return ----------
        
        Self {
            dialog,
            
            notebook,
            
            watching_treeview,
            on_hold_treeview,
            plan_to_watch_treeview,
        }
        
    }
    
    fn build_treeview() -> (gtk::ScrolledWindow, gtk::TreeView) {
        let scrolled_window = {
            
            gtk::builders::ScrolledWindowBuilder::new()
            .visible(true)
            .vexpand(true)
            .margin_top(6)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        let treeview = {
            
            gtk::builders::TreeViewBuilder::new()
            .visible(true)
            .headers_visible(false)
            .enable_search(true)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        // 0 => title
        
        let title_column = gtk::TreeViewColumn::new();
        title_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let title_cell = gtk::CellRendererText::new();
        
        gtk::prelude::CellLayoutExt::pack_end(&title_column, &title_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&title_column, &title_cell, "text", 3);
        
        title_column.set_sort_column_id(3);
        
        treeview.append_column(&title_column);
        
        treeview.set_search_column(3);
        
        (scrolled_window, treeview)
    }
    
}

impl CandidatesDownloaded {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            vertical_box
                
                ----- title -----
                
                horizontal_box
                    static_label
                    { title_label }
                /horizontal_box
                
                ----- download -----
                
                horizontal_box
                    static_label
                    { download_spin }
                /horizontal_box
                
            /vertical_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(450)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();        
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Vertical);
        content_area.add(&main_box);
        
        // ---------- title ----------
        
        let title_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::builders::LabelBuilder::new()
                .visible(true)
                .label("Candidate:")
                .xalign(1.0)
                .width_chars(10)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&title_box);
        
        let title_label = {
            
            gtk::builders::LabelBuilder::new()
            .visible(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build()
            
        };
        
        title_box.add(&title_label);
        
        // ---------- download ----------
        
        let download_box = {
            
            gtk::builders::BoxBuilder::new()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::builders::LabelBuilder::new()
                .visible(true)
                .label("Download:")
                .xalign(1.0)
                .width_chars(10)
                .build()
                
            })
            .build()
            
        };
        
        main_box.add(&download_box);
        
        let download_spin = {
            
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
        
        download_box.add(&download_spin);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            title_label,
            download_spin,
        }
        
    }
    
}

impl Feeds {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            horizontal_box
                static_label
                { url_entry }
            /horizontal_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(700)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();        
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Horizontal);
        content_area.add(&main_box);
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::builders::LabelBuilder::new()
                .visible(true)
                .label("URL:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let url_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&url_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            url_entry,
        }
        
    }
    
}

impl Kinds {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            horizontal_box
                static_label
                { name_entry }
            /horizontal_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(400)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Horizontal);
        content_area.add(&main_box);
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::builders::LabelBuilder::new()
                .visible(true)
                .label("Name:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let name_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&name_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            name_entry,
        }
        
    }
    
}

impl Formats {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            horizontal_box
                static_label
                { name_entry }
            /horizontal_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::builders::DialogBuilder::new()
            .transient_for(&window.general.window)
            .window_position(gtk::WindowPosition::CenterOnParent)
            .default_width(400)
            .build()
            
        };
        
        // ---------- content area ----------
        
        let content_area = dialog.content_area();
        content_area.set_spacing(DIALOGS_SPACING);
        
        // ---------- main box ----------
        
        let main_box = super::build_main_box(gtk::Orientation::Horizontal);
        content_area.add(&main_box);
        
        // ---------- static label ----------
        
        {
            
            main_box.add(
                &gtk::builders::LabelBuilder::new()
                .visible(true)
                .label("Name:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let name_entry = {
            
            gtk::builders::EntryBuilder::new()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&name_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            name_entry,
        }
        
    }
    
}