use gtk::{
    glib,
    prelude::*,
};

use crate::WatchlistSection;

pub struct Stores {
    pub entries: Entries,
}

pub struct Entries {
    pub store: gtk::ListStore,
    pub filters: Vec<gtk::TreeModelFilter>,
    pub sorts: Vec<gtk::TreeModelSort>,
}

impl Stores {
    
    pub fn new() -> Self {
        Self {
            entries: Entries::new(),
        }
    }
    
}

impl Entries {
    
    pub fn new() -> Self {
        
        // ---------- stores ----------
        
        let store = gtk::ListStore::new(
            &[
                // 0 => id
                glib::types::Type::I64,
                
                // 1 => weight
                glib::types::Type::I64,
                // 2 => status
                glib::types::Type::I64,
                
                // 3 => title
                glib::types::Type::STRING,
                // 4 => good
                glib::types::Type::STRING,
                // 5 => kind
                glib::types::Type::STRING,
                // 6 => progress
                glib::types::Type::I64,
            ],
        );
        
        // ---------- filters ----------
        
        let mut filters = Vec::with_capacity(WatchlistSection::iter().count());
        
        for section in WatchlistSection::iter() {
            
            let filter = gtk::TreeModelFilter::new(&store, None);
            Self::set_visible_func(&filter, section.to_int());
            
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
    
    fn set_visible_func(filter: &gtk::TreeModelFilter, show: i64) {
        filter.set_visible_func(move |model, iter| {
            
            model.value(iter, 2).get::<i64>().unwrap() == show
            
        });
    }
    
    fn set_sort_func(sort: &gtk::TreeModelSort) {
        sort.set_sort_func(gtk::SortColumn::Index(3), |model, first_iter, second_iter| {
            
            let first_title = model.value(first_iter, 3)
                .get::<&glib::GStr>()
                .map(|title| title.to_lowercase())
                .unwrap();
            
            let second_title = model.value(second_iter, 3)
                .get::<&glib::GStr>()
                .map(|title| title.to_lowercase())
                .unwrap();
            
            first_title.to_lowercase().cmp(&second_title.to_lowercase())
            
        });
    }
    
}
