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
    
    for treeview in &state.ui.widgets().window.watchlist.treeviews {
        
        // lookup selected title (CONTROL + L/l)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                match eventkey.keyval() {
                    
                    key if (key == gdk::keys::constants::L || key == gdk::keys::constants::l) && eventkey.state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        sender_cloned.send(Message::Watchlist(WatchlistActions::Lookup)).unwrap();
                        glib::Propagation::Stop
                    },
                    
                    _ => glib::Propagation::Proceed,
                    
                }
            }
        });
        
    }
}

pub fn lookup(state: &State) {
    let Some(treeview) = state.ui.watchlist_current_treeview() else {
        return;
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
    let url = lookup.replace("%s", &chikuwa::percent_encode(&title));
    
    if let Err(error) = chikuwa::execute_app(&url) {
        state.ui.dialogs_error_show(&error.to_string());
    }
}
