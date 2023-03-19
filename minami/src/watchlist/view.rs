use gtk::{
    gdk,
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    GeneralActions,
};

pub fn init(state: &State, sender: &Sender<Message>) {
    bind(state, sender);
}

fn bind(state: &State, sender: &Sender<Message>) {
    // ---------- treeviews ----------
    
    for treeview in &state.ui.widgets().window.watchlist.treeviews {
        
        // focus global search entry (SHIFT + Tab)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
                    sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
    
    // ---------- buttons ----------
    
    for button in &state.ui.widgets().window.watchlist.buttons_box.children() {
        
        // prevent selection of treeview (Up Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Up {
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
    
    if let Some(button) = state.ui.widgets().window.watchlist.buttons_box.children().first() {
        
        // prevent selection of preferences listbox (Left Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Left {
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
}
