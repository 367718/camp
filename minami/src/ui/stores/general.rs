use std::cmp::Ordering;

use gtk::{
    glib,
    prelude::*,
};

pub struct Stores {
    pub search: Search,
}

pub struct Search {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
}

impl Stores {
    
    pub fn new() -> Self {
        Self {
            search: Search::new(),
        }
    }
    
}

impl Search {
    
    fn new() -> Self {
        
        // ---------- store ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => file
                glib::types::Type::BOOL,
                // 1 => name / title
                glib::types::Type::STRING,
                // 2 => display
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- sort ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        // files first
        sort.set_sort_func(gtk::SortColumn::Index(0), |model, first_iter, second_iter| {
            
            let first_file = model.value(first_iter, 0).get::<bool>().unwrap();
            let second_file = model.value(second_iter, 0).get::<bool>().unwrap();
            
            if first_file && ! second_file { Ordering::Less }
            else if ! first_file && second_file { Ordering::Greater }
            else {
                
                let first_name = model.value(first_iter, 1).get::<glib::GString>().unwrap();
                let second_name = model.value(second_iter, 1).get::<glib::GString>().unwrap();
                
                chikuwa::natural_cmp(&first_name, &second_name)
                
            }
            
        });
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
        }
        
    }
    
}
