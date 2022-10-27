use gtk::{
    gdk,
    prelude::*,
};

pub struct Menus {
    pub bar: Bar,
}

pub struct Bar {
    pub menu: gtk::MenuBar,
}

impl Menus {
    
    pub fn new() -> Self {
        Self {
            bar: Bar::new(),
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
                
            /menu
            
            menu ("_View")
                
                menu_item ("_Focus search", "app.general.search.focus", "CONTROL + F")
                
                separator_menu_item
                
                menu_item ("Switch to _next section", "app.general.section.next", "CONTROL + Page down")
                menu_item ("Switch to _previous section", "app.general.section.previous", "CONTROL + Page up")
                
            /menu
            
            menu ("_Tools")
                
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
        gtk::builders::MenuItemBuilder::new()
        .visible(true)
        .sensitive(false)
        .label("Edit")
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
        
        // separator
        
        {
            
            menu.append(
                &gtk::builders::SeparatorMenuItemBuilder::new()
                .visible(true)
                .build()
            );
            
        }
        
        // next section
        
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
        
        // previous section
        
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
        
        // backup database
        
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
