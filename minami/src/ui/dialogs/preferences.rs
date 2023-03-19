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
    pub treeviews: Vec<gtk::TreeView>,
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
            
            main_box
                
                ----- title -----
                
                title_box
                    static_label ("Title:")
                    { title_entry }
                    image
                /title_box
                
                ----- group -----
                
                group_box
                    static_label ("Group:")
                    { group_entry }
                    image
                /group_box
                
                ----- quality -----
                
                quality_box
                    static_label ("Quality:")
                    { quality_entry }
                    image
                /quality_box
                
                ----- series -----
                
                series_box
                    static_label ("Series:")
                    { series_entry }
                    button (select_series, gtk::ResponseType::Other(1))
                /series_box
                
                ----- offset -----
                
                offset_box
                    static_label ("Offset:")
                    { offset_spin }
                /offset_box
                
                ----- current -----
                
                current_box
                    static_label ("Current:")
                    { current_switch }
                /current_box
                
                ----- downloaded -----
                
                downloaded_box
                    static_label ("Downloaded:")
                    { downloaded_spin }
                /downloaded_box
                
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Other(0))
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
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
        if let Some(button_box) = confirm_button.parent().and_downcast::<gtk::ButtonBox>() {
            let series_button = gtk::Button::builder()
                .visible(true)
                .image(&gtk::Image::from_icon_name(Some("open-menu-symbolic"), gtk::IconSize::Menu))
                .tooltip_text("Select series")
                .build();
            
            dialog.add_action_widget(&series_button, gtk::ResponseType::Other(1));
            
            button_box.remove(&series_button);
            series_box.add(&series_button);
        }
        
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
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
        gtk::Box::builder()
        .visible(true)
        .orientation(gtk::Orientation::Horizontal)
        .spacing(FIELDS_SPACING)
        .child(&{
            
            gtk::Label::builder()
            .visible(true)
            .label(text)
            .xalign(1.0)
            .width_chars(12)
            .build()
            
        })
        .build()
    }
    
    fn build_title() -> (gtk::Box, gtk::Entry) {
        // ---------- title box ----------
        
        let title_box = Self::build_field_box("Title:");
        
        // ---------- title entry ----------
        
        let title_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        title_box.add(&title_entry);
        
        // ---------- image ----------
        
        {
            
            title_box.add(
                &gtk::Image::builder()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text("Folders will be included when evaluating files for updates")
                .build()
            );
            
        }
        
        // ---------- return ----------
        
        (title_box, title_entry)
    }
    
    fn build_group() -> (gtk::Box, gtk::Entry) {
        // ---------- group box ----------
        
        let group_box = Self::build_field_box("Group:");
        
        // ---------- group entry ----------
        
        let group_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .width_chars(35)
            .build()
            
        };
        
        group_box.add(&group_entry);
        
        // ---------- image ----------
        
        {
            
            group_box.add(
                &gtk::Image::builder()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text(r#"A blank group means "match any""#)
                .build()
            );
            
        }
        
        // ---------- return ----------
        
        (group_box, group_entry)
    }
    
    fn build_quality() -> (gtk::Box, gtk::Entry) {
        // ---------- quality box ----------
        
        let quality_box = Self::build_field_box("Quality:");
        
        // ---------- quality entry ----------
        
        let quality_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .width_chars(35)
            .build()
            
        };
        
        quality_box.add(&quality_entry);
        
        // ---------- image ----------
        
        {
            
            quality_box.add(
                &gtk::Image::builder()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text(r#"A blank quality means "match any""#)
                .build()
            );
            
        }
        
        // ---------- return ----------
        
        (quality_box, quality_entry)
    }
    
    fn build_series() -> (gtk::Box, gtk::Entry) {
        // ---------- series box ----------
        
        let series_box = Self::build_field_box("Series:");
        
        // ---------- series entry ----------
        
        let series_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .sensitive(false)
            .build()
            
        };
        
        series_box.add(&series_entry);
        
        // ---------- return ----------
        
        (series_box, series_entry)
    }
    
    fn build_offset() -> (gtk::Box, gtk::SpinButton) {
        // ---------- offset box ----------
        
        let offset_box = Self::build_field_box("Offset:");
        
        // ---------- offset spin ----------
        
        let offset_spin = {
            
            gtk::SpinButton::builder()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::Adjustment::builder()
                .upper(9999.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        offset_box.add(&offset_spin);
        
        // ---------- return ----------
        
        (offset_box, offset_spin)
    }
    
    fn build_current() -> (gtk::Box, gtk::Switch) {
        // ---------- current box ----------
        
        let current_box = Self::build_field_box("Current:");
        
        // ---------- current switch ----------
        
        let current_switch = {
            
            gtk::Switch::builder()
            .visible(true)
            .build()
            
        };
        
        current_box.add(&current_switch);
        
        // ---------- return ----------
        
        (current_box, current_switch)
    }
    
    fn build_downloaded() -> (gtk::Box, gtk::SpinButton) {
        // ---------- downloaded box ----------
        
        let downloaded_box = Self::build_field_box("Downloaded:");
        
        // ---------- downloaded spin ----------
        
        let downloaded_spin = {
            
            gtk::SpinButton::builder()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::Adjustment::builder()
                .upper(99_999.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        downloaded_box.add(&downloaded_spin);
        
        // ---------- return ----------
        
        (downloaded_box, downloaded_spin)
    }
    
}

impl CandidatesSeries {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            { notebook }
                
                watching_scrolled
                    treeview
                /watching_scrolled
                
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
            
            gtk::Dialog::builder()
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
            
            gtk::Notebook::builder()
            .visible(true)
            .show_border(false)
            .build()
            
        };
        
        content_area.add(&notebook);
        
        // ---------- treeviews ----------
        
        let mut treeviews = Vec::with_capacity(WatchlistSection::iter().count().saturating_sub(1));
        
        for section in WatchlistSection::iter() {
            if section != WatchlistSection::Completed {
                
                let (scrolled, treeview) = Self::build_treeview();
                
                notebook.append_page(
                    &scrolled,
                    Some(&{
                        
                        gtk::Label::builder()
                        .visible(true)
                        .label(section.display())
                        .width_chars(12)
                        .build()
                        
                    })
                );
                
                treeviews.push(treeview);
                
            }
        }
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Other(0));
        dialog.add_button("Add new", gtk::ResponseType::Other(1));
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Other(0));
        
        // ---------- return ----------
        
        Self {
            dialog,
            notebook,
            treeviews,
        }
        
    }
    
    fn build_treeview() -> (gtk::ScrolledWindow, gtk::TreeView) {
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .margin_top(6)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        // ---------- treeview ----------
        
        let treeview = {
            
            gtk::TreeView::builder()
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
        
        CellLayoutExt::pack_end(&title_column, &title_cell, true);
        TreeViewColumnExt::add_attribute(&title_column, &title_cell, "text", 3);
        
        title_column.set_sort_column_id(3);
        
        treeview.append_column(&title_column);
        
        treeview.set_search_column(3);
        
        // ---------- return ----------
        
        (scrolled_window, treeview)
    }
    
}

impl CandidatesDownloaded {
    
    fn new(window: &Window) -> Self {
        
        /*
        
        content_area
            
            main_box
                
                ----- title -----
                
                title_box
                    static_label ("Candidate:")
                    { title_label }
                /title_box
                
                ----- download -----
                
                download_box
                    static_label ("Download:")
                    { download_spin }
                /download_box
                
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
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
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
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
            
            gtk::Label::builder()
            .visible(true)
            .ellipsize(pango::EllipsizeMode::End)
            .build()
            
        };
        
        title_box.add(&title_label);
        
        // ---------- download ----------
        
        let download_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
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
            
            gtk::SpinButton::builder()
            .visible(true)
            .activates_default(true)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::Adjustment::builder()
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
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
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
            
            main_box
                static_label ("URL:")
                { url_entry }
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
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
                &gtk::Label::builder()
                .visible(true)
                .label("URL:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let url_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&url_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
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
            
            main_box
                static_label ("Name:")
                { name_entry }
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
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
                &gtk::Label::builder()
                .visible(true)
                .label("Name:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let name_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&name_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
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
            
            main_box
                static_label ("Name:")
                { name_entry }
            /main_box
            
        /content_area
        
        action_area
            
            button ("Confirm", gtk::ResponseType::Ok)
            button ("Cancel", gtk::ResponseType::Cancel)
            
        /action_area
        
        */
        
        // ---------- dialog ----------
        
        let dialog = {
            
            gtk::Dialog::builder()
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
                &gtk::Label::builder()
                .visible(true)
                .label("Name:")
                .build()
            );
            
        }
        
        // ---------- entry ----------
        
        let name_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .hexpand(true)
            .activates_default(true)
            .build()
            
        };
        
        main_box.add(&name_entry);
        
        // ---------- buttons ----------
        
        let confirm_button = dialog.add_button("Confirm", gtk::ResponseType::Ok);
        let cancel_button = dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        
        confirm_button.style_context().add_class(gtk::STYLE_CLASS_SUGGESTED_ACTION);
        cancel_button.style_context().add_class(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
        
        dialog.set_default_response(gtk::ResponseType::Ok);
        
        // ---------- return ----------
        
        Self {
            dialog,
            name_entry,
        }
        
    }
    
}
