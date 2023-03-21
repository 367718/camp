use gtk::{
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    GeneralActions,
};

pub fn init(state: &State) {
    build(state);
}

fn build(state: &State) {
    fill(state);
    
    // watchlist
    
    for (treeview, sort) in state.ui.widgets().window.watchlist.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
        treeview.set_model(Some(sort));
    }
    
    // candidates
    
    for (treeview, sort) in state.ui.widgets().dialogs.preferences.candidates_series.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
        treeview.set_model(Some(sort));
    }
}

fn fill(state: &State) {
    // 0 => id
    // 1 => weight
    // 2 => status
    // 3 => title
    // 4 => good
    // 5 => kind
    // 6 => progress
    
    let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
    watchlist_store.clear();
    
    for (id, entry) in state.database.series_iter() {
        watchlist_store.insert_with_values(
            None,
            &[
                (0, &id.as_int()),
                
                (1, &(u32::from(entry.good().as_int()) * 400)),
                (2, &entry.status().as_int()),
                
                (3, &entry.title()),
                (4, &entry.good().display()),
                (5, &state.database.kinds_get(entry.kind()).map_or("", |kind| kind.name())),
                (6, &entry.progress()),
            ],
        );
    }
}

pub fn reload(state: &State, sender: &Sender<Message>) {
    // watchlist

    for (treeview, sort) in state.ui.widgets().window.watchlist.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        treeview.set_model(None::<&gtk::TreeModel>);
        sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    }
    
    // candidates
    
    for (treeview, sort) in state.ui.widgets().dialogs.preferences.candidates_series.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        treeview.set_model(None::<&gtk::TreeModel>);
        sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    }
    
    fill(state);
    
    // watchlist
    
    for (treeview, sort) in state.ui.widgets().window.watchlist.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
        treeview.set_model(Some(sort));
    }
    
    // candidates
    
    for (treeview, sort) in state.ui.widgets().dialogs.preferences.candidates_series.treeviews.iter().zip(state.ui.widgets().stores.watchlist.entries.sorts.iter()) {
        sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
        treeview.set_model(Some(sort));
    }
    
    sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
}
