use gtk::{
    gdk,
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    WatchlistActions, GeneralActions,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(app, state);
    bind(state, sender);
}

fn build(app: &gtk::Application, state: &State) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- menus ----------
    
    state.ui.widgets().menus.watchlist.popup.menu.insert_action_group("app", Some(app));
    
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
    // 1 => modified (weight)
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
                
                (1, &(u32::from(entry.good.as_int()) * 400)),
                (2, &entry.status.as_int()),
                
                (3, &entry.title),
                
                (4, &entry.good.display()),
                (5, &state.database.kinds_get(entry.kind).map_or("", |kind| &kind.name)),
                (6, &entry.progress),
            ],
        );
    }
}

fn bind(state: &State, sender: &Sender<Message>) {
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.watchlist.watching_treeview,
        &state.ui.widgets().window.watchlist.on_hold_treeview,
        &state.ui.widgets().window.watchlist.plan_to_watch_treeview,
        &state.ui.widgets().window.watchlist.completed_treeview,
    ];
    
    for treeview in treeviews {
        
        // open popup menu (Right-click)
        treeview.connect_button_release_event({
            let sender_cloned = sender.clone();
            move |_, button| {
                if button.button() == 3 {
                    sender_cloned.send(Message::Watchlist(WatchlistActions::MenuPopup(button.coords()))).unwrap();
                }
                Inhibit(false)
            }
        });
        
        // open popup menu (Menu key)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, key| {
                if *key.keyval() == 65_383 {
                    sender_cloned.send(Message::Watchlist(WatchlistActions::MenuPopup(None))).unwrap();
                }
                Inhibit(false)
            }
        });
        
    }
}

pub fn menu_popup(state: &State, coords: Option<(f64, f64)>) {
    let Some(treeview) = state.ui.watchlist_current_treeview() else {
        return;
    };
    
    let (treepaths, _) = treeview.selection().selected_rows();
    
    if treepaths.is_empty() {
        return;
    }
    
    let watchlist_popup = &state.ui.widgets().menus.watchlist.popup.menu;
    
    let mut event = gdk::Event::new(gdk::EventType::Nothing);
    
    // prevent "no trigger event", "no display for event" and "event not holding seat" warnings
    if let Some(seat) = state.ui.widgets().window.general.window.display().default_seat() {
        event.set_device(seat.pointer().as_ref());
    }
    
    // mouse
    // check if pointer is within the position of the first selected row
    if let Some((x, y)) = coords.map(|(x, y)| (x as i32, y as i32)) {
        if let Some((mouse_path, _, _, _)) = treeview.path_at_pos(x, y) {
            if treepaths.first() == mouse_path.as_ref() {
                
                let rect = treeview.background_area(treepaths.first(), None::<&gtk::TreeViewColumn>);
                
                watchlist_popup.set_rect_anchor_dx(x);
                watchlist_popup.set_rect_anchor_dy(y + rect.height());
                
                watchlist_popup.popup_at_widget(
                    treeview,
                    gdk::Gravity::NorthWest,
                    gdk::Gravity::NorthWest,
                    Some(&event),
                );
                
            }
        }
    
    // keyboard
    // show menu at an offsetted position from the last selected row
    } else {
        
        let rect = treeview.background_area(treepaths.last(), None::<&gtk::TreeViewColumn>);
        
        watchlist_popup.set_rect_anchor_dx(5);
        watchlist_popup.set_rect_anchor_dy(rect.y() + (rect.height() * 2) + 5);
        
        watchlist_popup.popup_at_widget(
            treeview,
            gdk::Gravity::NorthWest,
            gdk::Gravity::NorthWest,
            Some(&event),
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
