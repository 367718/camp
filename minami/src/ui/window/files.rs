use gtk::{
    pango,
    prelude::*,
};

use crate::FilesSection;

use super::{
    WINDOW_SPACING,
    SECTIONS_LISTBOX_ROW_WIDTH, SECTIONS_LISTBOX_ROW_HEIGHT,
    General,
};

pub struct Files {
    pub listbox: gtk::ListBox,
    pub stack: gtk::Stack,
    
    pub new_treeview: gtk::TreeView,
    pub watched_treeview: gtk::TreeView,
    
    pub frame: gtk::Frame,
    pub buttons_box: gtk::Box,
}

impl Files {
    
    pub fn new(general: &General) -> Self {
        
        /*
        
        scrolled_window
            { listbox }
        /scrolled_window
        
        section_box
            
            { stack }
                
                ----- new -----
                
                new_scrolled_window
                    { new_treeview }
                /new_scrolled_window
                
                ----- watched -----
                
                watched_scrolled_window
                    { watched_treeview }
                /watched_scrolled_window
                
            /stack
            
            { frame }
                static_label
            /frame
            
            { buttons_box }
                
                button ("Play", "app.files.file.play")
                button ("Mark", "app.files.file.mark")
                button ("Move", "app.files.file.move")
                button ("Lookup", "app.files.tools.lookup")
                
            /buttons_box
            
        /section_box
        
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
        
        for section in FilesSection::iter() {
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
            if row.index() == 0 {
                
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
                    .label("Files")
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
        
        // ---------- section box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        general.sections_stack.add_named(&section_box, "Files");
        
        // ---------- subsections ----------
        
        let stack = {
            
            gtk::Stack::builder()
            .visible(true)
            .transition_duration(0)
            .build()
            
        };
        
        section_box.add(&stack);
        
        let (new_scrolled_window, new_treeview) = Self::build_treeview();
        let (watched_scrolled_window, watched_treeview) = Self::build_treeview();
        
        stack.add_named(&new_scrolled_window, FilesSection::New.display());
        stack.add_named(&watched_scrolled_window, FilesSection::Watched.display());
        
        // make sure global search scrolling works as intended
        
        new_treeview.realize();
        watched_treeview.realize();
        
        // ---------- frame ----------
        
        let frame = gtk::Frame::builder()
            .no_show_all(true)
            .shadow_type(gtk::ShadowType::In)
            .child(&{
                
                let label = gtk::Label::builder()
                .visible(true)
                .label("The file watcher is not currently running. Changes will not be detected.")
                .xalign(0.0)
                .margin(6)
                .build();
                
                label.style_context().add_class("foreground-red");
                
                label
                
            })
            .build();
        
        section_box.add(&frame);
        
        // ---------- buttons ----------
        
        let buttons_box = Self::build_buttons();
        
        section_box.add(&buttons_box);
        
        // ---------- return ----------
        
        Self {
            listbox,
            stack,
            
            new_treeview,
            watched_treeview,
            
            frame,
            buttons_box,
        }
        
    }
    
    fn build_treeview() -> (gtk::ScrolledWindow, gtk::TreeView) {
        // ---------- scrolled window ----------
        
        let scrolled_window = {
            
            gtk::ScrolledWindow::builder()
            .visible(true)
            .vexpand(true)
            .shadow_type(gtk::ShadowType::In)
            .build()
            
        };
        
        // ---------- treeview ----------
        
        let treeview = {
            
            gtk::TreeView::builder()
            .visible(true)
            .headers_visible(false)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .enable_tree_lines(true)
            .fixed_height_mode(true)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        treeview.selection().set_mode(gtk::SelectionMode::Multiple);
        
        // 0 => file stem
        
        let file_stem_column = gtk::TreeViewColumn::new();
        file_stem_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
        
        let file_stem_cell = gtk::CellRendererText::new();
        file_stem_cell.set_ellipsize(pango::EllipsizeMode::Middle);
        
        CellLayoutExt::pack_end(&file_stem_column, &file_stem_cell, true);
        TreeViewColumnExt::add_attribute(&file_stem_column, &file_stem_cell, "strikethrough", 1);
        TreeViewColumnExt::add_attribute(&file_stem_column, &file_stem_cell, "text", 3);
        
        treeview.append_column(&file_stem_column);
        
        // ---------- return ----------
        
        (scrolled_window, treeview)
    }
    
    fn build_buttons() -> gtk::Box {
        // ---------- buttons box ----------
        
        let buttons_box = super::build_buttons_box();
        
        buttons_box.set_margin_top(WINDOW_SPACING);
        
        // ---------- play ----------
        
        buttons_box.add(&super::build_button(
            "Play",
            "app.files.file.play",
            Some(gtk::STYLE_CLASS_SUGGESTED_ACTION),
        ));
        
        // ---------- mark ----------
        
        buttons_box.add(&super::build_button(
            "Mark",
            "app.files.file.mark",
            None,
        ));
        
        // ---------- move ----------
        
        buttons_box.add(&super::build_button(
            "Move",
            "app.files.file.move",
            None,
        ));
        
        // ---------- lookup ----------
        
        buttons_box.add(&super::build_button(
            "Lookup",
            "app.files.tools.lookup",
            None,
        ));
        
        // ---------- return ----------
        
        buttons_box
    }
    
}
