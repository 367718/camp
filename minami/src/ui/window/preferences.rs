use gtk::prelude::*;

use crate::PreferencesSection;

use super::{
    WINDOW_SPACING, FIELDS_SPACING,
    SECTIONS_LISTBOX_ROW_WIDTH, SECTIONS_LISTBOX_ROW_HEIGHT,
    General,
};

pub struct Preferences {
    pub listbox: gtk::ListBox,
    pub stack: gtk::Stack,
    
    pub candidates: Candidates,
    pub feeds: Feeds,
    pub kinds: Kinds,
    pub formats: Formats,
    pub media: Media,
    pub paths: Paths,
}

pub struct Candidates {
    pub candidates_treeview: gtk::TreeView,
    pub downloaded_treeview: gtk::TreeView,

    pub candidates_buttons_box: gtk::Box,
    pub downloaded_buttons_box: gtk::Box,
}

pub struct Feeds {
    pub treeview: gtk::TreeView,
    pub buttons_box: gtk::Box,
}

pub struct Kinds {
    pub treeview: gtk::TreeView,
    pub buttons_box: gtk::Box,
}

pub struct Formats {
    pub treeview: gtk::TreeView,
    pub buttons_box: gtk::Box,
}

pub struct Media {
    pub player_entry: gtk::Entry,
    pub iconify_switch: gtk::Switch,
    pub flag_entry: gtk::Entry,
    pub timeout_spin: gtk::SpinButton,
    pub autoselect_switch: gtk::Switch,
    pub lookup_entry: gtk::Entry,
    pub bind_entry: gtk::Entry,
    
    pub buttons_box: gtk::Box,
}

pub struct Paths {
    pub files_button: gtk::Button,
    pub downloads_button: gtk::Button,
    pub database_button: gtk::Button,
    
    pub files_entry: gtk::Entry,
    pub downloads_entry: gtk::Entry,
    pub pipe_entry: gtk::Entry,
    pub database_entry: gtk::Entry,
    
    pub buttons_box: gtk::Box,
}

impl Preferences {
    
    pub fn new(general: &General) -> Self {
        
        /*
        
        scrolled_window
            { listbox }
        /scrolled_window
        
        vertical_box
            
            { stack }
                
                { candidates }
                { feeds }
                { kinds }
                { formats }
                { media }
                { paths }
                
            /stack
            
        /vertical_box
        
        */
        
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        general.switchers_box.add(&scrolled_window);
        
        // ---------- listbox ----------
        
        let listbox = {
            
            gtk::ListBox::builder()
            .visible(true)
            .build()
            
        };
        
        scrolled_window.add(&listbox);
        
        // ---------- section box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        general.sections_stack.add_named(&section_box, "Preferences");
        
        // ---------- subsections ----------
        
        let stack = {
            
            gtk::Stack::builder()
            .visible(true)
            .transition_duration(0)
            .build()
            
        };
        
        section_box.add(&stack);
        
        let (candidates_box, candidates) = Candidates::new();
        let (feeds_box, feeds) = Feeds::new();
        let (kinds_box, kinds) = Kinds::new();
        let (formats_box, formats) = Formats::new();
        let (media_box, media) = Media::new();
        let (paths_box, paths) = Paths::new();
        
        stack.add_named(&candidates_box, PreferencesSection::Candidates.display());
        stack.add_named(&feeds_box, PreferencesSection::Feeds.display());
        stack.add_named(&kinds_box, PreferencesSection::Kinds.display());
        stack.add_named(&formats_box, PreferencesSection::Formats.display());
        stack.add_named(&media_box, PreferencesSection::Media.display());
        stack.add_named(&paths_box, PreferencesSection::Paths.display());
        
        // ---------- listbox ----------
        
        for section in PreferencesSection::iter() {
            listbox.add(
                &gtk::ListBoxRow::builder()
                .visible(true)
				.can_focus(false)
                .width_request(SECTIONS_LISTBOX_ROW_WIDTH)
                .height_request(SECTIONS_LISTBOX_ROW_HEIGHT)
                .name(section.display())
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label(section.display())
                    .halign(gtk::Align::Start)
                    .build()
                    
                })
                .build()
            );
        }
        
        listbox.set_header_func(Some(Box::new(|row, _| {
            if row.widget_name() == PreferencesSection::Candidates.display() {
                
                let header_box = {
                    
                    gtk::Box::builder()
                    .visible(true)
                    .orientation(gtk::Orientation::Vertical)
                    .build()
                    
                };
                
                header_box.add(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .sensitive(false)
                    .width_request(SECTIONS_LISTBOX_ROW_WIDTH)
                    .height_request(SECTIONS_LISTBOX_ROW_HEIGHT)
                    .xalign(0.0)
                    .label("Preferences")
                    .halign(gtk::Align::Start)
                    .build()
                    
                });
                
                header_box.add(&{
                    
                    gtk::Separator::builder()
                    .visible(true)
                    .valign(gtk::Align::Center)
                    .orientation(gtk::Orientation::Horizontal)
                    .build()
                    
                });
                
                row.set_header(Some(&header_box));
                
            }
        })));
        
        // ---------- return ----------
        
        Self {
            listbox,
            stack,
            
            candidates,
            feeds,
            kinds,
            formats,
            media,
            paths,
        }
        
    }
    
}

impl Candidates {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        horizontal_box
            
            ----- candidates -----
            
            vertical_box
                
                scrolled_window
                    { candidates_treeview }
                /scrolled_window
                
                { candidates_buttons_box }
                    
                    button ("Add", "app.preferences.candidates.candidates.add")
                    button ("Edit", "app.preferences.candidates.candidates.edit")
                    button ("Delete", "app.preferences.candidates.candidates.delete")
                    
                /candidates_buttons_box
                
            /vertical_box
            
            ----- downloaded -----
            
            vertical_box
                
                scrolled_window
                    { downloaded_treeview }
                /scrolled_window
                
                { downloaded_buttons_box }
                    
                    button ("Add", "app.preferences.candidates.downloaded.add")
                    button ("Delete", "app.preferences.candidates.downloaded.delete")
                    
                /downloaded_buttons_box
                
            /vertical_box
            
        /horizontal_box
        
        */
        
        // ---------- horizontal_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        // ---------- candidates ----------
        
        let (candidates_box, candidates_treeview, candidates_buttons_box) = Self::build_candidates();
        
        section_box.add(&candidates_box);
        
        // ---------- downloaded ----------
        
        let (downloaded_box, downloaded_treeview, downloaded_buttons_box) = Self::build_downloaded();
        
        section_box.add(&downloaded_box);
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                candidates_treeview,
                downloaded_treeview,
                
                candidates_buttons_box,
                downloaded_buttons_box,
            },
        )
        
    }
    
    fn build_candidates() -> (gtk::Box, gtk::TreeView, gtk::Box) {
        // ---------- vertical_box ----------
        
        let candidates_box = {
            
            gtk::Box::builder()
            .visible(true)
            .hexpand(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        candidates_box.add(&scrolled_window);
        
        // ---------- candidates_treeview ----------
        
        let candidates_treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&candidates_treeview);
        
        // 0 => title
        
        let title_column = gtk::TreeViewColumn::new();
        title_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let title_cell = gtk::CellRendererText::new();
        
        gtk::prelude::CellLayoutExt::pack_end(&title_column, &title_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&title_column, &title_cell, "text", 1);
        
        candidates_treeview.append_column(&title_column);
        
        // ---------- buttons ----------
        
        let candidates_buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        candidates_box.add(&candidates_buttons_box);
        
        // candidate add
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Add")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.candidates.candidates.add")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            candidates_buttons_box.add(&button);
            
        }
        
        // candidate edit
        
        {
            
            candidates_buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Edit")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.candidates.candidates.edit")
                .build()
            );
            
        }
        
        // candidate delete
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Delete")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.candidates.candidates.delete")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            candidates_buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (candidates_box, candidates_treeview, candidates_buttons_box)
    }
    
    fn build_downloaded() -> (gtk::Box, gtk::TreeView, gtk::Box) {
        // ---------- vertical_box ----------
        
        let downloaded_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        downloaded_box.add(&scrolled_window);
        
        // ---------- downloaded_treeview ----------
        
        let downloaded_treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&downloaded_treeview);
        
        // 0 => download
        
        let download_column = gtk::TreeViewColumn::new();
        download_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let download_cell = gtk::CellRendererText::new();
        download_cell.set_xalign(0.90);
        
        gtk::prelude::CellLayoutExt::pack_end(&download_column, &download_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&download_column, &download_cell, "text", 0);
        
        downloaded_treeview.append_column(&download_column);
        
        // ---------- buttons ----------
        
        let downloaded_buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        downloaded_box.add(&downloaded_buttons_box);
        
        // add
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Add")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.candidates.downloaded.add")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            downloaded_buttons_box.add(&button);
            
        }
        
        // delete
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Delete")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.candidates.downloaded.delete")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            downloaded_buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (downloaded_box, downloaded_treeview, downloaded_buttons_box)
    }
    
}

impl Feeds {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        vertical_box
            
            scrolled_window
                { treeview }
            /scrolled_window
            
            { buttons_box }
                
                button ("Add", "app.preferences.feeds.add")
                button ("Edit", "app.preferences.feeds.edit")
                button ("Delete", "app.preferences.feeds.delete")
                
            /buttons_box
            
        /vertical_box
        
        */
        
        // ---------- vertical_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        section_box.add(&scrolled_window);
        
        // ---------- treeview ----------
        
        let treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .reorderable(true)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        // 0 => url
        
        let url_column = gtk::TreeViewColumn::new();
        url_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let url_cell = gtk::CellRendererText::new();
        
        gtk::prelude::CellLayoutExt::pack_end(&url_column, &url_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&url_column, &url_cell, "text", 1);
        
        treeview.append_column(&url_column);
        
        // ---------- buttons ----------
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // add
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Add")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.feeds.add")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // edit
        
        {
            
            buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Edit")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.feeds.edit")
                .build()
            );
            
        }
        
        // delete
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Delete")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.feeds.delete")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                treeview,
                buttons_box,
            },
        )
        
    }
    
}

impl Kinds {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        vertical_box
            
            scrolled_window
                { treeview }
            /scrolled_window
            
            { buttons_box }
                
                button ("Add", "app.preferences.kinds.add")
                button ("Edit", "app.preferences.kinds.edit")
                button ("Delete", "app.preferences.kinds.delete")
                
            /buttons_box
            
        /vertical_box
        
        */
        
        // ---------- vertical_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        section_box.add(&scrolled_window);
        
        // ---------- treeview ----------
        
        let treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .reorderable(true)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        // 0 => name
        
        let name_column = gtk::TreeViewColumn::new();
        name_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let name_cell = gtk::CellRendererText::new();
        
        gtk::prelude::CellLayoutExt::pack_end(&name_column, &name_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&name_column, &name_cell, "text", 1);
        
        treeview.append_column(&name_column);
        
        // ---------- buttons ----------
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // add
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Add")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.kinds.add")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // edit
        
        {
            
            buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Edit")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.kinds.edit")
                .build()
            );
            
        }
        
        // delete
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Delete")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.kinds.delete")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                treeview,
                buttons_box,
            },
        )
        
    }
    
}

impl Formats {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        vertical_box
            
            scrolled_window
                { treeview }
            /scrolled_window
            
            { buttons_box }
                
                button ("Add", "app.preferences.formats.add")
                button ("Edit", "app.preferences.formats.edit")
                button ("Delete", "app.preferences.formats.delete")
                
            /buttons_box
            
        /vertical_box
        
        */
        
        // ---------- vertical_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- treeview ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        section_box.add(&scrolled_window);
        
        let treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        // 0 => name
        
        let name_column = gtk::TreeViewColumn::new();
        name_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let name_cell = gtk::CellRendererText::new();
        
        gtk::prelude::CellLayoutExt::pack_end(&name_column, &name_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&name_column, &name_cell, "text", 1);
        
        treeview.append_column(&name_column);
        
        // ---------- buttons ----------
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // add
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Add")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.formats.add")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // edit
        
        {
            
            buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Edit")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.formats.edit")
                .build()
            );
            
        }
        
        // delete
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Delete")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.formats.delete")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                treeview,
                buttons_box,
            },
        )
        
    }
    
}

impl Media {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        vertical_box
            
            scrolled_window
                
                fields_box
                    
                    ----- player -----
                    
                    horizontal_box
                        static_label
                        { player_entry }
                    /horizontal_box
                    
                    ----- iconify -----
                    
                    horizontal_box
                        static_label
                        { iconify_switch }
                    /horizontal_box
                    
                    ----- flag -----
                    
                    horizontal_box
                        static_label
                        { flag_entry }
                    /horizontal_box
                    
                    ----- timeout -----
                    
                    horizontal_box
                        static_label
                        { timeout_spin }
                            adjustment
                        /timeout_spin
                        static_label
                    /horizontal_box
                    
                    ----- autoselect -----
                    
                    horizontal_box
                        static_label
                        { autoselect_switch }
                    /horizontal_box
                    
                    ----- lookup -----
                    
                    horizontal_box
                        static_label
                        { lookup_entry }
                        warning_image
                    /horizontal_box
                    
                    ----- bind -----
                    
                    horizontal_box
                        static_label
                        { bind_entry }
                    /horizontal_box
                    
                /fields_box
                
            /scrolled_window
            
            { buttons_box }
                
                button ("Confirm", "app.preferences.media.confirm")
                button ("Unlock", "app.preferences.media.unlock")
                button ("Discard", "app.preferences.media.discard")
                
            /buttons_box
            
        /vertical_box
        
        */
        
        // ---------- vertical_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .valign(gtk::Align::Fill)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled_window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .valign(gtk::Align::Fill)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        section_box.add(&scrolled_window);
        
        // ---------- fields ----------
        
        let fields_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .margin(WINDOW_SPACING)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        scrolled_window.add(&fields_box);
        
        let (player_box, player_entry) = Self::build_player();
        let (iconify_box, iconify_switch) = Self::build_iconify();
        let (flag_box, flag_entry) = Self::build_flag();
        let (timeout_box, timeout_spin) = Self::build_timeout();
        let (autoselect_box, autoselect_switch) = Self::build_autoselect();
        let (lookup_box, lookup_entry) = Self::build_lookup();
        let (bind_box, bind_entry) = Self::build_bind();
        
        {
            
            fields_box.add(&player_box);
            fields_box.add(&iconify_box);            
            fields_box.add(&flag_box);
            fields_box.add(&timeout_box);
            fields_box.add(&autoselect_box);
            fields_box.add(&lookup_box);
            fields_box.add(&bind_box);
            
        }
        
        // ---------- buttons ----------
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // confirm
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Confirm")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.media.confirm")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // unlock
        
        {
            
            buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Unlock")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.media.unlock")
                .build()
            );
            
        }
        
        // discard
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Discard")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.media.discard")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                player_entry,
                iconify_switch,
                flag_entry,
                timeout_spin,
                autoselect_switch,
                lookup_entry,
                bind_entry,
                
                buttons_box,
            },
        )
        
    }
    
    fn build_field_box(text: &str) -> gtk::Box {
        gtk::Box::builder()
        .visible(true)
        .valign(gtk::Align::Center)
        .orientation(gtk::Orientation::Horizontal)
        .spacing(FIELDS_SPACING)
        .child(&{
            
            gtk::Label::builder()
            .visible(true)
            .label(text)
            .xalign(1.0)
            .width_chars(26)
            .build()
            
        })
        .build()
    }
    
    fn build_player() -> (gtk::Box, gtk::Entry) {
        let player_box = Self::build_field_box("Command for playing files:");
        
        let player_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .sensitive(false)
            .hexpand(true)
            .build()
            
        };
        
        player_box.add(&player_entry);
        
        (player_box, player_entry)
    }
    
    fn build_iconify() -> (gtk::Box, gtk::Switch) {
        let iconify_box = Self::build_field_box("Iconify on file played:");
        
        let iconify_switch = {
            
            gtk::Switch::builder()
            .visible(true)
            .sensitive(false)
            .build()
            
        };
        
        iconify_box.add(&iconify_switch);
        
        (iconify_box, iconify_switch)
    }
    
    fn build_flag() -> (gtk::Box, gtk::Entry) {
        let flag_box = Self::build_field_box("Flag for marking files:");
        
        let flag_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .sensitive(false)
            .hexpand(true)
            .build()
            
        };
        
        flag_box.add(&flag_entry);
        
        (flag_box, flag_entry)
    }
    
    fn build_timeout() -> (gtk::Box, gtk::SpinButton) {
        let timeout_box = Self::build_field_box("Downloads timeout:");
        
        let timeout_spin = {
            
            gtk::SpinButton::builder()
            .visible(true)
            .sensitive(false)
            .snap_to_ticks(true)
            .numeric(true)
            .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
            .adjustment(&{
                
                gtk::Adjustment::builder()
                .upper(86400.0)
                .step_increment(1.0)
                .page_increment(10.0)
                .build()
                
            })
            .build()
            
        };
        
        timeout_box.add(&timeout_spin);
        
        {
            
            timeout_box.add(
                &gtk::Label::builder()
                .visible(true)
                .label("seconds")
                .build()
            );
            
        }
        
        (timeout_box, timeout_spin)
    }
    
    fn build_autoselect() -> (gtk::Box, gtk::Switch) {
        let autoselect_box = Self::build_field_box("Autoselect modified series:");
        
        let autoselect_switch = {
            
            gtk::Switch::builder()
            .visible(true)
            .sensitive(false)
            .build()
            
        };
        
        autoselect_box.add(&autoselect_switch);
        
        (autoselect_box, autoselect_switch)
    }
    
    fn build_lookup() -> (gtk::Box, gtk::Entry) {
        let lookup_box = Self::build_field_box("Lookup URL:");
        
        let lookup_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .sensitive(false)
            .hexpand(true)
            .build()
            
        };
        
        lookup_box.add(&lookup_entry);
        
        {
            
            lookup_box.add(
                &gtk::Image::builder()
                .visible(true)
                .icon_name("dialog-information-symbolic")
                .tooltip_text("'%s' will be replaced by query")
                .build()
            );
            
        }
        
        (lookup_box, lookup_entry)
    }
    
    fn build_bind() -> (gtk::Box, gtk::Entry) {
        let bind_box = Self::build_field_box("Remote control address:");
        
        let bind_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .sensitive(false)
            .hexpand(true)
            .build()
            
        };
        
        bind_box.add(&bind_entry);
        
        (bind_box, bind_entry)
    }
    
}

impl Paths {
    
    fn new() -> (gtk::Box, Self) {
        
        /*
        
        vertical_box
            
            scrolled_window
                
                ----- files -----
                
                horizontal_box
                    static_label
                    { files_button }
                    { files_entry }
                /horizontal_box
                
                ----- downloads -----
                
                horizontal_box
                    static_label
                    { downloads_button }
                    { downloads_entry }
                /horizontal_box
                
                ----- pipe -----
                
                horizontal_box
                    static_label
                    { pipe_entry }
                /horizontal_box
                
                ----- database -----
                
                horizontal_box
                    static_label
                    { database_button }
                    { database_entry }
                /horizontal_box
                
            /scrolled_window
            
            { buttons_box }
                
                button ("Confirm", "app.preferences.paths.confirm")
                button ("Unlock", "app.preferences.paths.unlock")
                button ("Discard", "app.preferences.paths.discard")
                
            /buttons_box
            
        /vertical_box
        
        */
        
        // ---------- vertical_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .spacing(WINDOW_SPACING)
            .valign(gtk::Align::Fill)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled_window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .valign(gtk::Align::Fill)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        section_box.add(&scrolled_window);
        
        // ---------- fields ----------
        
        let fields_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .margin(WINDOW_SPACING)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        scrolled_window.add(&fields_box);
        
        let (files_box, files_button, files_entry) = Self::build_field("Files directory:");
        let (downloads_box, downloads_button, downloads_entry) = Self::build_field("Downloads directory:");
        let (pipe_box, _, pipe_entry) = Self::build_field("Remote control pipe path:");
        let (database_box, database_button, database_entry) = Self::build_field("Database file:");
        
        {
            
            fields_box.add(&files_box);
            fields_box.add(&downloads_box);
            fields_box.add(&pipe_box);
            fields_box.add(&database_box);
            
        }
        
        // ---------- buttons ----------
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // confirm
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Confirm")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.paths.confirm")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // unlock
        
        {
            
            buttons_box.add(
                &gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Unlock")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.paths.unlock")
                .build()
            );
            
        }
        
        // discard
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Discard")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.preferences.paths.discard")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_DESTRUCTIVE_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // ---------- return ----------
        
        (
            section_box,
            Self {
                files_button,
                downloads_button,
                database_button,
                
                files_entry,
                downloads_entry,
                pipe_entry,
                database_entry,
                
                buttons_box,
            },
        )
        
    }
    
    fn build_field(text: &str) -> (gtk::Box, gtk::Button, gtk::Entry) {
        let field_box = {
            
            gtk::Box::builder()
            .visible(true)
            .valign(gtk::Align::Center)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(FIELDS_SPACING)
            .child(&{
                
                gtk::Label::builder()
                .visible(true)
                .label(text)
                .xalign(1.0)
                .width_chars(26)
                .build()
                
            })
            .build()
            
        };
        
        let field_button = {
            
            gtk::Button::builder()
            .visible(true)
            .sensitive(false)
            .image(&gtk::Image::from_icon_name(Some("folder-symbolic"), gtk::IconSize::Menu))
            .build()
            
        };
        
        field_box.add(&field_button);
        
        let field_entry = {
            
            gtk::Entry::builder()
            .visible(true)
            .sensitive(false)
            .hexpand(true)
            .build()
            
        };
        
        field_box.add(&field_entry);
        
        (field_box, field_button, field_entry)
    }
    
}
