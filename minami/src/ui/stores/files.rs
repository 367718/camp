use std::cmp::Ordering;

use gtk::{
    glib,
    prelude::*,
};

use crate::FilesSection;

pub struct Stores {
    pub entries: Entries,
    pub move_to_folder: MoveToFolder,
}

pub struct Entries {
    pub store: gtk::TreeStore,
    pub filters: Vec<gtk::TreeModelFilter>,
    pub sorts: Vec<gtk::TreeModelSort>,
}

pub struct MoveToFolder {
    pub store: gtk::ListStore,
    pub sort: gtk::TreeModelSort,
}

impl Stores {
    
    pub fn new() -> Self {
        Self {
            entries: Entries::new(),
            move_to_folder: MoveToFolder::new(),
        }
    }
    
}

impl Entries {
    
    pub fn new() -> Self {
        
        // ---------- stores ----------
        
        let store = gtk::TreeStore::new(
            &[
                // 0 => path (empty if container)
                glib::types::Type::STRING,
                
                // 1 => updated (strikethrough)
                glib::types::Type::BOOL,
                // 2 => watched
                glib::types::Type::BOOL,
                
                // 3 => file stem
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- filters ----------
        
        let mut filters = Vec::with_capacity(FilesSection::iter().count());
        
        for section in FilesSection::iter() {
            
            let filter = gtk::TreeModelFilter::new(&store, None);
            Self::set_visible_func(&filter, section != FilesSection::New);
            
            filters.push(filter);
            
        }
        
        // ---------- sorts ----------
        
        let mut sorts = Vec::with_capacity(filters.len());
        
        for filter in &filters {
            
            let sort = gtk::TreeModelSort::new(filter);
            Self::set_sort_func(&sort);
            
            sorts.push(sort);
            
        }
        
        // ---------- return ----------
        
        Self {
            store,
            filters,
            sorts,
        }
        
    }
    
    fn set_visible_func(filter: &gtk::TreeModelFilter, show: bool) {
        filter.set_visible_func(move |model, iter| {
            
            model.value(iter, 2).get::<bool>().unwrap() == show
            
        });
    }
    
    fn set_sort_func(sort: &gtk::TreeModelSort) {
        sort.set_sort_func(gtk::SortColumn::Index(0), |model, first_iter, second_iter| {
            
            let first_path = model.value(first_iter, 0).get::<glib::GString>().unwrap();
            let second_path = model.value(second_iter, 0).get::<glib::GString>().unwrap();
            
            if first_path.is_empty() && ! second_path.is_empty() { Ordering::Greater }
            else if ! first_path.is_empty() && second_path.is_empty() { Ordering::Less }
            else {
                
                let first_file_stem = model.value(first_iter, 3).get::<glib::GString>().unwrap();
                let second_file_stem = model.value(second_iter, 3).get::<glib::GString>().unwrap();
                
                chikuwa::natural_cmp(&first_file_stem, &second_file_stem)
                
            }
            
        });
    }
    
}

impl MoveToFolder {
    
    fn new() -> Self {
        
        // ---------- stores ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => name
                glib::types::Type::STRING,
            ],
        );
        
        // ---------- sorts ----------
        
        let sort = gtk::TreeModelSort::new(&store);
        
        sort.set_sort_func(gtk::SortColumn::Index(0), |model, first_iter, second_iter| {
            
            let first_name = model.value(first_iter, 0).get::<glib::GString>().unwrap();
            let second_name = model.value(second_iter, 0).get::<glib::GString>().unwrap();
            
            chikuwa::natural_cmp(&first_name, &second_name)
            
        });
        
        // ---------- return ----------
        
        Self {
            store,
            sort,
        }
        
    }
    
}
