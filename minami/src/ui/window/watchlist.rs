use gtk::{
    pango,
    prelude::*,
};

use crate::WatchlistSection;

use super::{
    WINDOW_SPACING,
    SECTIONS_LISTBOX_ROW_WIDTH, SECTIONS_LISTBOX_ROW_HEIGHT,
    General,
};

pub struct Watchlist {
    pub listbox: gtk::ListBox,
    pub stack: gtk::Stack,
    
    pub watching_treeview: gtk::TreeView,
    pub on_hold_treeview: gtk::TreeView,
    pub plan_to_watch_treeview: gtk::TreeView,
    pub completed_treeview: gtk::TreeView,

    pub buttons_box: gtk::Box,
}

impl Watchlist {
    
    pub fn new(general: &General) -> Self {
        
        /*
        
        scrolled_window
            { listbox }
        scrolled_window
        
        section_box
            
            { stack }
                
                ----- watching -----
                
                watching_scrolled_window
                    { watchlist_treeview }
                /watching_scrolled_window
                
                ----- on hold -----
                
                on_hold_scrolled_window
                    { on_hold_treeview }
                /on_hold_scrolled_window
                
                ----- plan to watch -----
                
                plan_to_watch_scrolled_window
                    { plan_to_watch_treeview }
                /plan_to_watch_scrolled_window
                
                ----- completed -----
                
                completed_scrolled_window
                    { completed_treeview }
                /completed_scrolled_window
                
            /stack
            
            { buttons_box }
                
                button ("Add", "app.watchlist.edit.add")
                button ("Edit", "app.watchlist.edit.edit")
                button ("Delete", "app.watchlist.edit.delete")
                button ("Lookup", "app.watchlist.tools.lookup")
                
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
        
        for section in WatchlistSection::iter() {
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
                    .label("Watchlist")
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
            .spacing(WINDOW_SPACING)
            .orientation(gtk::Orientation::Vertical)
            .build()
            
        };
        
        general.sections_stack.add_named(&section_box, "Watchlist");
        
        // ---------- subsections ----------
        
        let stack = {
            
            gtk::Stack::builder()
            .visible(true)
            .transition_duration(0)
            .build()
            
        };
        
        section_box.add(&stack);
        
        let (watching_scrolled_window, watching_treeview) = Self::build_treeview(WatchlistSection::Watching);
        let (on_hold_scrolled_window, on_hold_treeview) = Self::build_treeview(WatchlistSection::OnHold);
        let (plan_to_watch_scrolled_window, plan_to_watch_treeview) = Self::build_treeview(WatchlistSection::PlanToWatch);
        let (completed_scrolled_window, completed_treeview) = Self::build_treeview(WatchlistSection::Completed);
        
        stack.add_named(&watching_scrolled_window, WatchlistSection::Watching.display());
        stack.add_named(&on_hold_scrolled_window, WatchlistSection::OnHold.display());
        stack.add_named(&plan_to_watch_scrolled_window, WatchlistSection::PlanToWatch.display());
        stack.add_named(&completed_scrolled_window, WatchlistSection::Completed.display());
        
        // make sure global search scrolling works as intended
        
        watching_treeview.realize();
        on_hold_treeview.realize();
        plan_to_watch_treeview.realize();
        completed_treeview.realize();
        
        // ---------- buttons ----------
        
        let buttons_box = Self::build_buttons();
        
        section_box.add(&buttons_box);
        
        // ---------- return ----------
        
        Self {
            listbox,
            stack,
            
            watching_treeview,
            on_hold_treeview,
            plan_to_watch_treeview,
            completed_treeview,

            buttons_box,
        }
        
    }
    
    fn build_treeview(section: WatchlistSection) -> (gtk::ScrolledWindow, gtk::TreeView) {
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
            .headers_visible(true)
            .enable_search(false)
            .enable_grid_lines(gtk::TreeViewGridLines::Horizontal)
            .build()
            
        };
        
        scrolled_window.add(&treeview);
        
        treeview.selection().set_mode(gtk::SelectionMode::Multiple);
        
        // 0 => title
        
        // watching / on-hold:
        // 1 => kind
        // 2 => progress
        
        // plan to watch:
        // 1 => kind
        
        // completed:
        // 1 => good
        // 2 => kind
        // 3 => progress
        
        // title column
        
        let title_column = gtk::TreeViewColumn::new();
        title_column.set_title("title");
        title_column.set_expand(true);
        title_column.set_sort_indicator(true);
        title_column.set_reorderable(true);
        
        let title_cell = gtk::CellRendererText::new();
        title_cell.set_ellipsize(pango::EllipsizeMode::End);
        
        CellLayoutExt::pack_end(&title_column, &title_cell, true);
        
        if section == WatchlistSection::Completed {
            TreeViewColumnExt::add_attribute(&title_column, &title_cell, "weight", 1);
        }
        
        TreeViewColumnExt::add_attribute(&title_column, &title_cell, "text", 3);
        
        title_column.set_sort_column_id(3);
        
        treeview.append_column(&title_column);
        
        // good column (only for completed)
        
        if section == WatchlistSection::Completed {
            let good_column = gtk::TreeViewColumn::new();
            good_column.set_title("good");
            good_column.set_sort_indicator(true);
            good_column.set_reorderable(true);
            
            let good_cell = gtk::CellRendererText::new();
            good_cell.set_xalign(0.50);
            
            CellLayoutExt::pack_end(&good_column, &good_cell, true);
            TreeViewColumnExt::add_attribute(&good_column, &good_cell, "weight", 1);
            TreeViewColumnExt::add_attribute(&good_column, &good_cell, "text", 4);
            
            good_column.set_sort_column_id(4);
            
            treeview.append_column(&good_column);
        }
        
        // kind column
        
        let kind_column = gtk::TreeViewColumn::new();
        kind_column.set_title("kind");
        kind_column.set_sort_indicator(true);
        kind_column.set_reorderable(true);
        
        let kind_cell = gtk::CellRendererText::new();
        kind_cell.set_xalign(0.50);
        
        CellLayoutExt::pack_end(&kind_column, &kind_cell, true);
        
        if section == WatchlistSection::Completed {
            TreeViewColumnExt::add_attribute(&kind_column, &kind_cell, "weight", 1);
        }
        
        TreeViewColumnExt::add_attribute(&kind_column, &kind_cell, "text", 5);
        
        kind_column.set_sort_column_id(5);
        
        treeview.append_column(&kind_column);
        
        // progress column (only for watching, on-hold and completed)
        
        if section != WatchlistSection::PlanToWatch {
            let progress_column = gtk::TreeViewColumn::new();
            progress_column.set_title("progress");
            progress_column.set_sort_indicator(true);
            progress_column.set_reorderable(true);
            
            let progress_cell = gtk::CellRendererText::new();
            progress_cell.set_xalign(0.85);
            
            CellLayoutExt::pack_end(&progress_column, &progress_cell, true);
            TreeViewColumnExt::add_attribute(&progress_column, &progress_cell, "weight", 1);
            TreeViewColumnExt::add_attribute(&progress_column, &progress_cell, "text", 6);
            
            progress_column.set_sort_column_id(6);
            
            treeview.append_column(&progress_column);
        }
        
        // ---------- return ----------
        
        (scrolled_window, treeview)
    }
    
    fn build_buttons() -> gtk::Box {
        // ---------- buttons box ----------
        
        let buttons_box = super::build_buttons_box();
        
        // ---------- add ----------
        
        buttons_box.add(&super::build_button(
            "Add",
            "app.watchlist.edit.add",
            Some(gtk::STYLE_CLASS_SUGGESTED_ACTION),
        ));
        
        // ---------- edit ----------
        
        buttons_box.add(&super::build_button(
            "Edit",
            "app.watchlist.edit.edit",
            None,
        ));
        
        // ---------- delete ----------
        
        buttons_box.add(&super::build_button(
            "Delete",
            "app.watchlist.edit.delete",
            Some(gtk::STYLE_CLASS_DESTRUCTIVE_ACTION),
        ));
        
        // ---------- lookup ----------
        
        buttons_box.add(&super::build_button(
            "Lookup",
            "app.watchlist.tools.lookup",
            None,
        ));
        
        // ---------- return ----------
        
        buttons_box
    }
    
}
