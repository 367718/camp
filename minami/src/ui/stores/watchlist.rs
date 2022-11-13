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
    
    pub watching_filter: gtk::TreeModelFilter,
    pub on_hold_filter: gtk::TreeModelFilter,
    pub plan_to_watch_filter: gtk::TreeModelFilter,
    pub completed_filter: gtk::TreeModelFilter,
    
    pub watching_sort: gtk::TreeModelSort,
    pub on_hold_sort: gtk::TreeModelSort,
    pub plan_to_watch_sort: gtk::TreeModelSort,
    pub completed_sort: gtk::TreeModelSort,
    
    pub candidates_watching_sort: gtk::TreeModelSort,
    pub candidates_on_hold_sort: gtk::TreeModelSort,
    pub candidates_plan_to_watch_sort: gtk::TreeModelSort,
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
                glib::types::Type::U32,
                
                // 1 => weight
                glib::types::Type::U32,
                // 2 => status
                glib::types::Type::U8,
                
                // 3 => title
                glib::types::Type::STRING,
                // 4 => good
                glib::types::Type::STRING,
                // 5 => kind
                glib::types::Type::STRING,
                // 6 => progress
                glib::types::Type::U32,
            ],
        );
        
        // ---------- filters ----------
        
        let watching_filter = gtk::TreeModelFilter::new(&store, None);
        let on_hold_filter = gtk::TreeModelFilter::new(&store, None);
        let plan_to_watch_filter = gtk::TreeModelFilter::new(&store, None);
        let completed_filter = gtk::TreeModelFilter::new(&store, None);
        
        Self::set_visible_func(&watching_filter, WatchlistSection::Watching.as_int());
        Self::set_visible_func(&on_hold_filter, WatchlistSection::OnHold.as_int());
        Self::set_visible_func(&plan_to_watch_filter, WatchlistSection::PlanToWatch.as_int());
        Self::set_visible_func(&completed_filter, WatchlistSection::Completed.as_int());
        
        // ---------- sorts ----------
        
        let watching_sort = gtk::TreeModelSort::new(&watching_filter);
        let on_hold_sort = gtk::TreeModelSort::new(&on_hold_filter);
        let plan_to_watch_sort = gtk::TreeModelSort::new(&plan_to_watch_filter);
        let completed_sort = gtk::TreeModelSort::new(&completed_filter);
        
        Self::set_sort_func(&watching_sort);
        Self::set_sort_func(&on_hold_sort);
        Self::set_sort_func(&plan_to_watch_sort);
        Self::set_sort_func(&completed_sort);
        
        let candidates_watching_sort = gtk::TreeModelSort::new(&watching_filter);
        let candidates_on_hold_sort = gtk::TreeModelSort::new(&on_hold_filter);
        let candidates_plan_to_watch_sort = gtk::TreeModelSort::new(&plan_to_watch_filter);
        
        Self::set_sort_func(&candidates_watching_sort);
        Self::set_sort_func(&candidates_on_hold_sort);
        Self::set_sort_func(&candidates_plan_to_watch_sort);
        
        // ---------- return ----------
        
        Self {
            store,
            
            watching_filter,
            on_hold_filter,
            plan_to_watch_filter,
            completed_filter,
            
            watching_sort,
            on_hold_sort,
            plan_to_watch_sort,
            completed_sort,
            
            candidates_watching_sort,
            candidates_on_hold_sort,
            candidates_plan_to_watch_sort,
        }
        
    }
    
    fn set_visible_func(filter: &gtk::TreeModelFilter, show: u8) {
        filter.set_visible_func(move |model, iter| {
            
            model.value(iter, 2).get::<u8>().unwrap() == show
            
        });
    }
    
    fn set_sort_func(sort: &gtk::TreeModelSort) {
        sort.set_sort_func(gtk::SortColumn::Index(3), |model, first_iter, second_iter| {
            
            let first_title = model.value(first_iter, 3).get::<glib::GString>().unwrap();
            let second_title = model.value(second_iter, 3).get::<glib::GString>().unwrap();
            
            crate::general::natural_cmp(&first_title, &second_title)
            
        });
    }
    
}
