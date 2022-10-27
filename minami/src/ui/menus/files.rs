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
                
                menu_item ("_Play files", "app.files.file.play", "Return")
                menu_item ("Mark files as _watched", "app.files.file.mark", "Delete")
                
                separator_menu_item
                
                menu_item ("Re_name files", "app.files.file.rename", "F2")
                menu_item ("Move files to _folder", "app.files.file.move", "F3")
                menu_item ("_Delete files", "app.files.file.delete", "SHIFT + Delete")
                
                separator_menu_item
                
                menu_item ("Perform _maintenance", "app.files.file.maintenance")
                menu_item ("Open files di_rectory", "app.files.file.directory", "CONTROL + R")
                menu_item ("Re_load files", "app.files.general.reload")
                
                separator_menu_item
                
                menu_item ("_Quit", "app.general.save_and_quit", "CONTROL + Q")
                
            /menu
            
            menu ("_Edit")
                
                menu_item ("_Add candidate", "app.files.edit.candidate", "Insert")
                menu_item ("Add _series", "app.files.edit.series", "SHIFT + Insert")
                
                separator_menu_item
                
                menu_item ("_Copy names", "app.files.edit.copy", "CONTROL + C")
                
            /menu
            
            menu ("_View")
                
                menu_item ("_Focus search", "app.general.search.focus", "CONTROL + F")
                menu_item ("Focus curr_ent list", "app.general.section.focus", "CONTROL + E")
                
                separator_menu_item
                
                menu_item ("Switch to _next section", "app.general.section.next", "CONTROL + Page down")
                menu_item ("Switch to _previous section", "app.general.section.previous", "CONTROL + Page up")
                
            /menu
            
            menu ("_Tools")
                
                menu_item ("_Lookup name", "app.files.tools.lookup", "CONTROL + L")
                
                separator_menu_item
                
                menu_item ("_Start remote control", "app.files.tools.remote", "CONTROL + O")
                
                separator_menu_item
                
                menu_item ("_Download new releases", "app.files.tools.download", "CONTROL + D")
                menu_item ("_Update watched releases", "app.files.tools.update", "CONTROL + U")
                
                separator_menu_item
                
                menu_item ("_Backup database", "app.general.backup_database")
                
            /menu
            
        /menu
        
        */
        
        // ---------- menu ----------
        
        let menu = {
            
            gtk::builders::MenuBarBuilder::new()
            .visible(true)
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
        
        // play
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.play")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Play files")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_293, // Return
                        gdk::ModifierType::empty(),
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // mark
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.mark")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Mark files as _watched")
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
        
        // rename
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.rename")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Re_name files")
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
        
        // move
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.move")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Move files to _folder")
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
        
        // delete
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.delete")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Delete files")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_535, // Delete
                        gdk::ModifierType::SHIFT_MASK,
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
        
        // maintenance
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.maintenance")
                .label("Perform _maintenance")
                .use_underline(true)
                .build()
            );
            
        }
        
        // directory
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.directory")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Open files di_rectory")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        82, // R
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // reload
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.general.reload")
                .label("Re_load files")
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
        
        // candidate
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.edit.candidate")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Add candidate")
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
        
        // series
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.edit.series")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Add _series")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        65_379, // Insert
                        gdk::ModifierType::SHIFT_MASK,
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
                .action_name("app.files.edit.copy")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Copy names")
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
                .action_name("app.files.tools.lookup")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Lookup name")
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
        
        // remote
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.tools.remote")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("Start rem_ote control")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        79, // O
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
        
        // download
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.tools.download")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Download new releases")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        68, // D
                        gdk::ModifierType::CONTROL_MASK,
                    );
                    
                    label
                    
                })
                .build()
            );
            
        }
        
        // update
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.tools.update")
                .child(&{
                    
                    let label = gtk::builders::AccelLabelBuilder::new()
                        .visible(true)
                        .label("_Update watched releases")
                        .use_underline(true)
                        .xalign(0.0)
                        .build();
                    
                    label.set_accel(
                        85, // U
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
            
            menu_item ("_Play files", "app.files.file.play")
            menu_item ("Mark files as _watched", "app.files.file.mark")
            
            separator_menu_item
            
            menu_item ("Re_name files", "app.files.file.rename")
            menu_item ("Move files to _folder", "app.files.file.move")
            menu_item ("_Delete files", "app.files.file.delete")
            
            separator_menu_item
            
            menu_item ("_Add candidate", "app.files.edit.candidate")
            menu_item ("Add _series", "app.files.edit.series")
            
            separator_menu_item
            
            menu_item ("_Copy names", "app.files.edit.copy")
            
            separator_menu_item
            
            menu_item ("_Lookup name", "app.files.tools.lookup")
            
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
        
        // play
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.play")
                .label("_Play files")
                .use_underline(true)
                .build()
            );
            
        }
        
        // mark
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.mark")
                .label("Mark files as _watched")
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
        
        // rename
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.rename")
                .label("Re_name files")
                .use_underline(true)
                .build()
            );
            
        }
        
        // folder
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.move")
                .label("Move files to _folder")
                .use_underline(true)
                .build()
            );
            
        }
        
        // delete
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.file.delete")
                .label("_Delete files")
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
        
        // candidate
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.edit.candidate")
                .label("_Add candidate")
                .use_underline(true)
                .build()
            );
            
        }
        
        // series
        
        {
            
            menu.append(
                &gtk::builders::MenuItemBuilder::new()
                .visible(true)
                .action_name("app.files.edit.series")
                .label("Add _series")
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
                .action_name("app.files.edit.copy")
                .label("_Copy names")
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
                .action_name("app.files.tools.lookup")
                .label("_Lookup name")
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
