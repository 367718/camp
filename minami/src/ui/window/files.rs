use gtk::prelude::*;

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
}

impl Files {
    
    pub fn new(general: &General) -> Self {
        
        /*
        
        scrolled_window
            { listbox }
        /scrolled_window
        
        vertical_box
            
            { stack }
                
                ----- new -----
                
                vertical_box
                    
                    scrolled_window
                        { new_treeview }
                    /scrolled window
                    
                /vertical_box
                
                ----- watched -----
                
                vertical_box
                    
                    scrolled_window
                        { watched_treeview }
                    /scrolled_window
                    
                /vertical_box
                
            /stack
            
            { frame }
                static_label
            /frame
            
            horizontal_box
                
                button ("Play", "app.files.file.play")
                button ("Mark", "app.files.file.mark")
                button ("Move", "app.files.file.move")
                button ("Lookup", "app.files.tools.lookup")
                
            /horizontal_box
            
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
        
        general.sections_stack.add_named(&section_box, "Files");
        
        // ---------- subsections ----------
        
        let stack = {
            
            gtk::Stack::builder()
            .visible(true)
            .transition_duration(0)
            .build()
            
        };
        
        section_box.add(&stack);
        
        let (new_box, new_treeview) = Self::build_treeview();
        let (watched_box, watched_treeview) = Self::build_treeview();
        
        stack.add_named(&new_box, FilesSection::New.display());
        stack.add_named(&watched_box, FilesSection::Watched.display());
        
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
        
        let buttons_box = {
            
            gtk::Box::builder()
            .visible(true)
            .homogeneous(true)
            .margin_top(WINDOW_SPACING)
            .spacing(WINDOW_SPACING)
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .build()
            
        };
        
        section_box.add(&buttons_box);
        
        // play
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Play")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.files.file.play")
                .build();
            
            button.style_context().add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            
            buttons_box.add(&button);
            
        }
        
        // mark
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Mark")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.files.file.mark")
                .build();
            
            buttons_box.add(&button);
            
        }
        
        // move
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Move")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.files.file.move")
                .build();
            
            buttons_box.add(&button);
            
        }
        
        // lookup
        
        {
            
            let button = gtk::Button::builder()
                .visible(true)
                .child(&{
                    
                    gtk::Label::builder()
                    .visible(true)
                    .label("Lookup")
                    .xalign(0.5)
                    .width_chars(7)
                    .build()
                    
                })
                .action_name("app.files.tools.lookup")
                .build();
            
            buttons_box.add(&button);
            
        }
        
        // ---------- listbox ----------
        
        for section in FilesSection::iter() {
            listbox.add(
                &gtk::ListBoxRow::builder()
                .visible(true)
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
            if row.widget_name() == FilesSection::New.display() {
                
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
        
        // ---------- return ----------
        
        Self {
            listbox,
            stack,
            
            new_treeview,
            watched_treeview,
            
            frame,
        }
        
    }
    
    fn build_treeview() -> (gtk::Box, gtk::TreeView) {
        // ---------- section_box ----------
        
        let section_box = {
            
            gtk::Box::builder()
            .visible(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        // ---------- scrolled_window ----------
        
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
        
        gtk::prelude::CellLayoutExt::pack_end(&file_stem_column, &file_stem_cell, true);
        gtk::prelude::TreeViewColumnExt::add_attribute(&file_stem_column, &file_stem_cell, "strikethrough", 1);
        gtk::prelude::TreeViewColumnExt::add_attribute(&file_stem_column, &file_stem_cell, "text", 3);
        
        treeview.append_column(&file_stem_column);
        
        (section_box, treeview)
    }
    
}
