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
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widgets ----------
    
    let watching_sort = &state.ui.widgets().stores.watchlist.entries.watching_sort;
    let on_hold_sort = &state.ui.widgets().stores.watchlist.entries.on_hold_sort;
    let plan_to_watch_sort = &state.ui.widgets().stores.watchlist.entries.plan_to_watch_sort;
    let completed_sort = &state.ui.widgets().stores.watchlist.entries.completed_sort;
    
    let candidates_watching_sort = &state.ui.widgets().stores.watchlist.entries.candidates_watching_sort;
    let candidates_on_hold_sort = &state.ui.widgets().stores.watchlist.entries.candidates_on_hold_sort;
    let candidates_plan_to_watch_sort = &state.ui.widgets().stores.watchlist.entries.candidates_plan_to_watch_sort;
    
    // ---------- set sort ids ----------
    
    watching_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    on_hold_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    completed_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    
    candidates_watching_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    candidates_on_hold_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    candidates_plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().window.watchlist.watching_treeview.set_model(Some(watching_sort));
    state.ui.widgets().window.watchlist.on_hold_treeview.set_model(Some(on_hold_sort));
    state.ui.widgets().window.watchlist.plan_to_watch_treeview.set_model(Some(plan_to_watch_sort));
    state.ui.widgets().window.watchlist.completed_treeview.set_model(Some(completed_sort));
    
    state.ui.widgets().dialogs.preferences.candidates_series.watching_treeview.set_model(Some(candidates_watching_sort));
    state.ui.widgets().dialogs.preferences.candidates_series.on_hold_treeview.set_model(Some(candidates_on_hold_sort));
    state.ui.widgets().dialogs.preferences.candidates_series.plan_to_watch_treeview.set_model(Some(candidates_plan_to_watch_sort));
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
    // ---------- widgets ----------
    
    let watching_sort = &state.ui.widgets().stores.watchlist.entries.watching_sort;
    let on_hold_sort = &state.ui.widgets().stores.watchlist.entries.on_hold_sort;
    let plan_to_watch_sort = &state.ui.widgets().stores.watchlist.entries.plan_to_watch_sort;
    let completed_sort = &state.ui.widgets().stores.watchlist.entries.completed_sort;
    
    let candidates_watching_sort = &state.ui.widgets().stores.watchlist.entries.candidates_watching_sort;
    let candidates_on_hold_sort = &state.ui.widgets().stores.watchlist.entries.candidates_on_hold_sort;
    let candidates_plan_to_watch_sort = &state.ui.widgets().stores.watchlist.entries.candidates_plan_to_watch_sort;
    
    // ---------- unset models ----------
    
    state.ui.widgets().window.watchlist.watching_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().window.watchlist.on_hold_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().window.watchlist.plan_to_watch_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().window.watchlist.completed_treeview.set_model(None::<&gtk::TreeModel>);
    
    state.ui.widgets().dialogs.preferences.candidates_series.watching_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().dialogs.preferences.candidates_series.on_hold_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().dialogs.preferences.candidates_series.plan_to_watch_treeview.set_model(None::<&gtk::TreeModel>);
    
    // ---------- set sort ids ----------
    
    watching_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    on_hold_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    completed_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    candidates_watching_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    candidates_on_hold_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    candidates_plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ids ----------
    
    watching_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    on_hold_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    completed_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    
    candidates_watching_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    candidates_on_hold_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    candidates_plan_to_watch_sort.set_sort_column_id(gtk::SortColumn::Index(3), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().window.watchlist.watching_treeview.set_model(Some(watching_sort));
    state.ui.widgets().window.watchlist.on_hold_treeview.set_model(Some(on_hold_sort));
    state.ui.widgets().window.watchlist.plan_to_watch_treeview.set_model(Some(plan_to_watch_sort));
    state.ui.widgets().window.watchlist.completed_treeview.set_model(Some(completed_sort));
    
    state.ui.widgets().dialogs.preferences.candidates_series.watching_treeview.set_model(Some(candidates_watching_sort));
    state.ui.widgets().dialogs.preferences.candidates_series.on_hold_treeview.set_model(Some(candidates_on_hold_sort));
    state.ui.widgets().dialogs.preferences.candidates_series.plan_to_watch_treeview.set_model(Some(candidates_plan_to_watch_sort));
    
    // ---------- global search ----------
    
    sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
}
