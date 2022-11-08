use gtk::{
    gdk,
    gio,
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, WatchlistActions,
    KindsId, KindsEntry,
};

pub fn init(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widget ----------
    
    let kinds_sort = &state.ui.widgets().stores.preferences.kinds.sort;
    
    // ---------- set sort ----------
    
    kinds_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().dialogs.watchlist.series.kind_combo.set_model(Some(kinds_sort));
    state.ui.widgets().window.preferences.kinds.treeview.set_model(Some(kinds_sort));
}

fn fill(state: &State) {
    // 0 => id
    // 1 => name
    // 2 => id for combo
    
    let kinds_store = &state.ui.widgets().stores.preferences.kinds.store;
    kinds_store.clear();
    
    for (id, kind) in state.database.kinds_iter() {
        kinds_store.insert_with_values(
            None,
            &[
                (0, &id.as_int()),
                (1, &kind.name),
                (2, &id.as_int().to_string()),
            ],
        );
    }
}

fn bind(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let add_action = gio::SimpleAction::new("preferences.kinds.add", None);
    let edit_action = gio::SimpleAction::new("preferences.kinds.edit", None);
    let delete_action = gio::SimpleAction::new("preferences.kinds.delete", None);
    
    // add kind
    add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::KindsAdd)).unwrap()
    });
    
    // edit kind
    edit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::KindsEdit)).unwrap()
    });
    
    // delete kind
    delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::KindsDelete)).unwrap()
    });
    
    app.add_action(&add_action);
    app.add_action(&edit_action);
    app.add_action(&delete_action);
    
    // ---------- treeviews ----------
    
    let kinds_treeview = &state.ui.widgets().window.preferences.kinds.treeview;
    
    // add kind (Insert)
    // edit kind (F2)
    // delete kind (Delete)
    kinds_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Insert => sender_cloned.send(Message::Preferences(PreferencesActions::KindsAdd)).unwrap(),
                gdk::keys::constants::F2 => sender_cloned.send(Message::Preferences(PreferencesActions::KindsEdit)).unwrap(),
                gdk::keys::constants::Delete => sender_cloned.send(Message::Preferences(PreferencesActions::KindsDelete)).unwrap(),
                _ => (),
            }
            Inhibit(false)
        }
    });
    
    // edit kind (Double-click, Return, Space)
    kinds_treeview.connect_row_activated({
        let sender_cloned = sender.clone();
        move |_, _, _| sender_cloned.send(Message::Preferences(PreferencesActions::KindsEdit)).unwrap()
    });
}

pub fn add(state: &mut State) {
    let kinds_dialog = &state.ui.widgets().dialogs.preferences.kinds.dialog;
    
    kinds_dialog.set_title("Add kind");
    
    let name_entry = &state.ui.widgets().dialogs.preferences.kinds.name_entry;
    name_entry.set_text("");
    
    loop {
        
        name_entry.grab_focus();
        
        let response = kinds_dialog.run();
        
        kinds_dialog.unrealize();
        kinds_dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Ok => {
                
                let entry = KindsEntry {
                    name: name_entry.text().to_string(),
                };
                
                match state.database.kinds_add(entry) {
                    
                    Ok(id) => {
                        
                        let kind = state.database.kinds_get(id).unwrap();
                        
                        let kinds_treeview = &state.ui.widgets().window.preferences.kinds.treeview;
                        
                        let kinds_store = &state.ui.widgets().stores.preferences.kinds.store;
                        let kinds_sort = &state.ui.widgets().stores.preferences.kinds.sort;
                        
                        let store_iter = kinds_store.insert_with_values(
                            None,
                            &[
                                (0, &id.as_int()),
                                (1, &kind.name),
                                (2, &id.as_int().to_string()),
                            ],
                        );
                        
                        let sort_iter = kinds_sort.convert_child_iter_to_iter(&store_iter).unwrap();
                        let treepath = kinds_sort.path(&sort_iter).unwrap();
                        kinds_treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
                        kinds_treeview.grab_focus();
                        
                        break;
                        
                    },
                    
                    Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                    
                }
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

pub fn edit(state: &mut State, sender: &Sender<Message>) {
    let kinds_treeview = &state.ui.widgets().window.preferences.kinds.treeview;
    
    let Some((treemodel, treeiter)) = kinds_treeview.selection().selected() else {
        return;
    };
    
    let id = KindsId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
    
    match state.database.kinds_get(id) {
        
        Some(previous) => {
            
            let kinds_dialog = &state.ui.widgets().dialogs.preferences.kinds.dialog;
            
            kinds_dialog.set_title("Edit kind");
            
            let name_entry = &state.ui.widgets().dialogs.preferences.kinds.name_entry;
            
            name_entry.set_text(&previous.name);
            
            loop {
                
                name_entry.grab_focus();
                
                let response = kinds_dialog.run();
                
                kinds_dialog.unrealize();
                kinds_dialog.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Ok => {
                        
                        let entry = KindsEntry {
                            name: name_entry.text().to_string(),
                        };
                        
                        match state.database.kinds_edit(id, entry) {
                            
                            Ok(_) => {
                                
                                let kind = state.database.kinds_get(id).unwrap();
                                
                                let kinds_store = &state.ui.widgets().stores.preferences.kinds.store;
                                let kinds_sort = &state.ui.widgets().stores.preferences.kinds.sort;
                                
                                let store_iter = kinds_sort.convert_iter_to_child_iter(&treeiter);
                                
                                kinds_store.set_value(&store_iter, 1, &kind.name.to_value());
                                
                                kinds_treeview.grab_focus();
                                
                                // signal that watchlist should be reloaded
                                if state.database.series_iter().any(|(_, entry)| entry.kind == id) {
                                    sender.send(Message::Watchlist(WatchlistActions::Reload)).unwrap();
                                }
                                
                                break;
                                
                            },
                            
                            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                            
                        }
                        
                    },
                    
                    // cancel
                    
                    _ => break,
                    
                }
                
            }
            
        },
        
        None => state.ui.dialogs_error_show("Kind not found"),
        
    }
}

pub fn delete(state: &mut State) {
    let Some((treemodel, treeiter)) = state.ui.widgets().window.preferences.kinds.treeview.selection().selected() else {
        return;
    };
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        let id = KindsId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
        
        match state.database.kinds_remove(id) {
            
            Ok(_) => {
                
                let kinds_store = &state.ui.widgets().stores.preferences.kinds.store;
                let kinds_sort = &state.ui.widgets().stores.preferences.kinds.sort;
                
                let store_iter = kinds_sort.convert_iter_to_child_iter(&treeiter);
                
                kinds_store.remove(&store_iter);
                
            },
            
            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
            
        }
        
    }
}

pub fn reload(state: &State, sender: &Sender<Message>) {
    // ---------- widget ----------
    
    let kinds_sort = &state.ui.widgets().stores.preferences.kinds.sort;
    
    // ---------- unset model ----------
    
    state.ui.widgets().dialogs.watchlist.series.kind_combo.set_model(None::<&gtk::TreeModel>);
    
    // ---------- unset sort ----------
    
    kinds_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ----------
    
    kinds_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().dialogs.watchlist.series.kind_combo.set_model(Some(kinds_sort));
    
    // ---------- watchlist ----------
    
    sender.send(Message::Watchlist(WatchlistActions::Reload)).unwrap();
}
