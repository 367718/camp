use gtk::{
    gdk,
    prelude::*,
};

pub struct Menus {
    pub bar: Bar,
    pub popup: Popup,
}

pub struct Bar {
    pub menu: gtk::MenuBar,
}

pub struct Popup {
    pub menu: gtk::Menu,
}

impl Menus {
    
    pub fn new() -> Self {
        Self {
            bar: Bar::new(),
            popup: Popup::new(),
        }
    }
    
}

impl Bar {
    
    fn new() -> Self {
        
        /*
        
        { menu }
            
            menu ("_File")
                
                menu_item ("_Quit", "app.general.save_and_quit", "CONTROL + Q")
                
            /menu
            
            menu ("_Edit")
                
                menu_item ("_Add series", "app.watchlist.edit.add", "Insert")
                
                separator_menu_item
                
                menu_item ("_Edit series", "app.watchlist.edit.edit", "F2")
                menu_item ("Set series as comple_ted", "app.watchlist.edit.completed", "F3")
                
                separator_menu_item
                
                menu_item ("_Delete series", "app.watchlist.edit.delete", "Delete")
                
                separator_menu_item
                
                menu_item ("_Increment progress", "app.watchlist.edit.increment", "Add")
                menu_item ("Decre_ment progress", "app.watchlist.edit.decrement", "Subtract")
                
                separator_menu_item
                
                menu_item ("_Copy titles", "app.watchlist.edit.copy", "CONTROL + C")
                
            /menu
            
            menu ("_View")
                
                menu_item ("_Focus search", "app.general.search.focus", "CONTROL + F")
                menu_item ("Focus curr_ent list", "app.general.section.focus", "CONTROL + E")
                
                separator_menu_item
                
                menu_item ("Switch to _next section", "app.general.section.next", "CONTROL + Page down")
                menu_item ("Switch to _previous section", "app.general.section.previous", "CONTROL + Page up")
                
            /menu
            
            menu ("_Tools")
                
                menu_item ("_Lookup title", "app.watchlist.tools.lookup", "CONTROL + L")
                
                separator_menu_item
                
                menu_item ("_Backup database", "app.general.backup_database")
                
            /menu
            
        /menu
        
        */
        
        // ---------- menu ----------
        
        let menu = {
            
            gtk::builders::MenuBarBuilder::new()
            .visible(false)
            .no_show_all(true)
            .hexpand(true)
            .margin_start(1)
            .margin_end(1)
            .build()
            
        };
        
        // ---------- sections ----------
        
        menu.add(&Self::build_file());
        menu.add(&Self::build_edit());
        menu.add(&Self::build_view());
        menu.add(&Self::build_tools());
        
        // ---------- return ----------
        
        Self {
            menu,
        }
        
    }
    
    fn build_file() -> gtk::MenuItem {
        let menu = {
            
            gtk::builders::MenuBuilder::new()
            .visible(true)
            .build()
            
        };
        
        // quit
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.save_and_quit")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Quit")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        81, // Q
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        gtk::builders::MenuItemBuilder::new()
        .visible(true)
        .label("_File")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_edit() -> gtk::MenuItem {
        let menu = {
            
            gtk::builders::MenuBuilder::new()
            .visible(true)
            .build()
            
        };
        
        // add
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.add")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Add series")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_379, // Insert
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // edit
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.edit")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Edit series")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_471, // F2
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // completed
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.completed")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Set series as comple_ted")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_472, // F3
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // delete
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.delete")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Delete series")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_535, // Delete
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // increment
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.increment")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Increment progress")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_451, // Add
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // decrement
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.decrement")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Decre_ment progress")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_453, // Subtract
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // copy
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.copy")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Copy titles")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        67, // C
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        gtk::builders::MenuItemBuilder::new()
        .visible(true)
        .label("_Edit")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_view() -> gtk::MenuItem {
        let menu = {
            
            gtk::builders::MenuBuilder::new()
            .visible(true)
            .build()
            
        };
        
        // focus search
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.search.focus")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Focus search")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        70, // F
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // focus section
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.section.focus")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Focus curr_ent list")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        69, // E
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // next
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.section.next")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Switch to _next section")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_366, // Page down
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // previous
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.section.previous")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Switch to _previous section")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_365, // Page up
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        gtk::builders::MenuItemBuilder::new()
        .visible(true)
        .label("_View")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_tools() -> gtk::MenuItem {
        let menu = {
            
            gtk::builders::MenuBuilder::new()
            .visible(true)
            .build()
            
        };
        
        // lookup
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.tools.lookup")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Lookup title")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        76, // L
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // backup
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.general.backup_database")
                .label("_Backup database")
                .use_underline(true)
                .build()
            );
            
        }
        
        gtk::builders::MenuItemBuilder::new()
        .visible(true)
        .label("_Tools")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
}

impl Popup {
    
    fn new() -> Self {
        
        /*
        
        main_box
            
            menu_item ("_Increment progress", "app.watchlist.edit.increment")
            menu_item ("Decre_ment progress", "app.watchlist.edit.decrement")
            menu_item ("Set series as comple_ted", "app.watchlist.edit.completed")
            
            separator_menu_item
            
            menu_item ("_Edit series", "app.watchlist.edit.edit")
            menu_item ("_Delete series", "app.watchlist.edit.delete")
            
            separator_menu_item
            
            menu_item ("_Copy titles", "app.watchlist.edit.copy")
            
            separator_menu_item
            
            menu_item ("_Lookup title", ""app.watchlist.tools.lookup"")
            
        /main_box
        
        */
        
        // ---------- menu ----------
        
        let menu = {
            
            gtk::builders::MenuBuilder::new()
            .build()
            
        };
        
        // prevent issues when menu does not fit on screen
        menu.set_anchor_hints(gdk::AnchorHints::empty());
        
        // ---------- buttons ----------
        
        // increment
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.increment")
                .label("_Increment progress")
                .use_underline(true)
                .build()
            );
            
        }
        
        // decrement
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.decrement")
                .label("Decre_ment progress")
                .use_underline(true)
                .build()
            );
            
        }
        
        // completed
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.completed")
                .label("Set series as comple_ted")
                .use_underline(true)
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // edit
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.edit")
                .label("_Edit series")
                .use_underline(true)
                .build()
            );
            
        }
        
        // delete
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.delete")
                .label("_Delete series")
                .use_underline(true)
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // copy
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.edit.copy")
                .label("_Copy titles")
                .use_underline(true)
                .build()
            );
            
        }
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // lookup
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.watchlist.tools.lookup")
                .label("_Lookup title")
                .use_underline(true)
                .build()
            );
            
        }
        
        // ---------- return ----------
        
        Self {
            menu,
        }
        
    }
    
}
