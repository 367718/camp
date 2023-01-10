use gtk::{
    gdk,
    prelude::*,
};

pub struct Watchlist {
    pub menu: gtk::MenuBar,
}

impl Watchlist {
    
    pub fn new() -> Self {
        
        /*
        
        { menu }
            
            menu ("_File")
                
                menu_item ("_Quit", "app.general.save_and_quit", "CONTROL + Q")
                
            /menu
            
            menu ("_Edit")
                
                menu_item ("_Add series", "app.watchlist.edit.add", "Insert")
                menu_item ("_Edit series", "app.watchlist.edit.edit", "F2")
                
                separator_menu_item
                
                menu_item ("_Delete series", "app.watchlist.edit.delete", "Delete")
                
                separator_menu_item
                
                menu_item ("Copy _titles", "app.watchlist.edit.copy", "CONTROL + C")
                
            /menu
            
            menu ("_View")
                
                menu_item ("Focus _search", "app.general.search.focus", "CONTROL + F")
                menu_item ("Focus _current section", "app.general.section.focus.start", "CONTROL + E")
                
                separator_menu_item
                
                menu_item ("Switch to _next section", "app.general.section.switch.next", "CONTROL + Page down")
                menu_item ("Switch to _previous section", "app.general.section.switch.previous", "CONTROL + Page up")
                
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
            
            gtk::MenuBar::builder()
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
            
            gtk::Menu::builder()
            .visible(true)
            .build()
            
        };
        
        // quit
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.save_and_quit")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
        
        gtk::MenuItem::builder()
        .visible(true)
        .label("_File")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_edit() -> gtk::MenuItem {
        let menu = {
            
            gtk::Menu::builder()
            .visible(true)
            .build()
            
        };
        
        // add
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.watchlist.edit.add")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
        
        // edit
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.watchlist.edit.edit")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
        
        // separator
        
        {
            
            menu.append(
                &gtk::SeparatorMenuItem::builder()
                .visible(true)
                .build()
            );
            
        }
        
        // delete
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.watchlist.edit.delete")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
                &gtk::SeparatorMenuItem::builder()
                .visible(true)
                .build()
            );
            
        }
        
        // copy
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.watchlist.edit.copy")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
                        .visible(true)
                        .label("Copy _titles")
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
                &gtk::SeparatorMenuItem::builder()
                .visible(true)
                .build()
            );
            
        }
        
        gtk::MenuItem::builder()
        .visible(true)
        .label("_Edit")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_view() -> gtk::MenuItem {
        let menu = {
            
            gtk::Menu::builder()
            .visible(true)
            .build()
            
        };
        
        // focus search
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.search.focus")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
                        .visible(true)
                        .label("Focus _search")
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
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.section.focus.start")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
                        .visible(true)
                        .label("Focus _current section")
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
                &gtk::SeparatorMenuItem::builder()
                .visible(true)
                .build()
            );
            
        }
        
        // next
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.section.switch.next")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.section.switch.previous")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
        
        gtk::MenuItem::builder()
        .visible(true)
        .label("_View")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
    fn build_tools() -> gtk::MenuItem {
        let menu = {
            
            gtk::Menu::builder()
            .visible(true)
            .build()
            
        };
        
        // lookup
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.watchlist.tools.lookup")
                .child(&{
                    
                    let label = gtk::AccelLabel::builder()
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
                &gtk::SeparatorMenuItem::builder()
                .visible(true)
                .build()
            );
            
        }
        
        // backup
        
        {
            
            menu.append(
                &gtk::MenuItem::builder()
                .visible(true)
                .action_name("app.general.backup_database")
                .label("_Backup database")
                .use_underline(true)
                .build()
            );
            
        }
        
        gtk::MenuItem::builder()
        .visible(true)
        .label("_Tools")
        .use_underline(true)
        .submenu(&menu)
        .build()
    }
    
}
