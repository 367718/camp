use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    WatchlistActions,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    bind(app, state, sender);
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let lookup_action = gio::SimpleAction::new("watchlist.tools.lookup", None);
    
    // lookup selected title
    lookup_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Watchlist(WatchlistActions::Lookup)).unwrap()
    });
    
    app.add_action(&lookup_action);
    
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.watchlist.watching_treeview,
        &state.ui.widgets().window.watchlist.on_hold_treeview,
        &state.ui.widgets().window.watchlist.plan_to_watch_treeview,
        &state.ui.widgets().window.watchlist.completed_treeview,
    ];
    
    for treeview in treeviews {
        
        // lookup selected title (CONTROL + L/l)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, key| {
                
                // lookup selected title (CONTROL + L/l)
                if (*key.keyval() == 76 || *key.keyval() == 108) && key.state().contains(gdk::ModifierType::CONTROL_MASK) {
                    sender_cloned.send(Message::Watchlist(WatchlistActions::Lookup)).unwrap();
                    return Inhibit(true);
                }
                
                Inhibit(false)
                
            }
        });
        
    }
}

pub fn lookup(state: &State) {
    let treeview = match state.ui.watchlist_current_treeview() {
        Some(treeview) => treeview,
        None => return,
    };
    
    let (treepaths, treemodel) = treeview.selection().selected_rows();
    
    if treepaths.is_empty() {
        return;
    }
    
    if treepaths.len() > 1 {
        treeview.set_cursor(treepaths.first().unwrap(), None::<&gtk::TreeViewColumn>, false);
    }
    
    let treeiter = treemodel.iter(treepaths.first().unwrap()).unwrap();
    let title = treemodel.value(&treeiter, 3).get::<glib::GString>().unwrap();
    
    let lookup = state.params.media_lookup(true);
    let url = lookup.replace("%s", &crate::general::percent_encode(&title));
    
    if let Err(error) = crate::general::open(&url) {
        state.ui.dialogs_error_show(&error.to_string());
    }
}
