use gtk::{
    gdk,
    prelude::*,
};

pub struct Preferences {
    pub menu: gtk::MenuBar,
}

impl Preferences {
    
    pub fn new() -> Self {
        
        /*
        
        { menu }
            
            menu ("_File")
                
                menu_item ("_Quit", "app.general.save_and_quit", "CONTROL + Q")
                
            /menu
            
            menu ("_Edit")
                
            /menu
            
            menu ("_View")
                
                menu_item ("Focus _search", "app.general.search.focus", "CONTROL + F")
                menu_item ("Focus _current section", "app.general.section.focus.start", "CONTROL + E")
                
                separator_menu_item
                
                menu_item ("Switch to _next section", "app.general.section.switch.next", "CONTROL + Page down")
                menu_item ("Switch to _previous section", "app.general.section.switch.previous", "CONTROL + Page up")
                
            /menu
            
            menu ("_Tools")
                
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
                        *gdk::keys::constants::Q,
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
        gtk::MenuItem::builder()
        .visible(true)
        .sensitive(false)
        .label("Edit")
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
                        *gdk::keys::constants::F,
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
                        *gdk::keys::constants::E,
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
        
        // next section
        
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
                        *gdk::keys::constants::Page_Down,
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
                        *gdk::keys::constants::Page_Up,
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
        
        // backup database
        
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
