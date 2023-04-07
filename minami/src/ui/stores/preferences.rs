use gtk::{
    glib,
    prelude::*,
};

pub struct Stores {
    pub candidates: Candidates,
    pub feeds: Feeds,
    pub kinds: Kinds,
    pub formats: Formats,
}

pub struct Candidates {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
    
    pub downloaded_store: gtk::ListStore,
    pub downloaded_sort: gtk::TreeModelSort,
}

pub struct Feeds {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
}

pub struct Kinds {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
}

pub struct Formats {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
}

impl Stores {
    
    pub fn new() -> Self {
        Self {
            candidates: Candidates::new(),
            feeds: Feeds::new(),
            kinds: Kinds::new(),
            formats: Formats::new(),
        }
    }
    
}

impl Candidates {
    
    fn new() -> Self {
        
        // ---------- stores ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => id
                glib::types::Type::I64,
                // 1 => title
                glib::types::Type::STRING,
            ],
        );
        
        let downloaded_store = gtk::ListStore::new(
            &[
                // 0 => download
                glib::types::Type::I64,
            ],
        );
        
        // ---------- sorts ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        sort.set_sort_func(gtk::SortColumn::Index(0), move |model, first_iter, second_iter| {
            
            let first_title = model.value(first_iter, 1).get::<glib::GString>().unwrap();
            let second_title = model.value(second_iter, 1).get::<glib::GString>().unwrap();
            
            crate::general::natural_cmp(&first_title, &second_title)
            
        });
        
        let downloaded_sort = gtk::TreeModelSort::new(&downloaded_store);
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
            
            downloaded_store,
            downloaded_sort,
        }
        
    }
    
}

impl Feeds {
    
    fn new() -> Self {
        
        // ---------- store ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => id
                glib::types::Type::I64,
                // 1 => url
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- sort ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
        }
        
    }
    
}

impl Kinds {
    
    fn new() -> Self {
        
        // ---------- store ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => id
                glib::types::Type::I64,
                // 1 => name
                glib::types::Type::STRING,
                // 2 => id for combo
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- sort ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
        }
        
    }
    
}

impl Formats {
    
    fn new() -> Self {
        
        // ---------- store ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => id
                glib::types::Type::I64,
                // 1 => name
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- sort ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
        }
        
    }
    
}
