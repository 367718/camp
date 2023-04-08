use gtk::{
    gdk,
    gio,
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, GeneralActions,
    FeedsId, FeedsEntry,
};

pub fn init(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widget ----------
    
    let feeds_sort = &state.ui.widgets().stores.preferences.feeds.sort;
    
    // ---------- set sort ----------
    
    feeds_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().window.preferences.feeds.treeview.set_model(Some(feeds_sort));
}

fn fill(state: &State) {
    // 0 => id
    // 1 => url
    
    let feeds_store = &state.ui.widgets().stores.preferences.feeds.store;
    feeds_store.clear();
    
    for (id, feed) in state.database.feeds_iter() {
        feeds_store.insert_with_values(
            None,
            &[
                (0, &i64::from(id)),
                (1, &feed.url()),
            ],
        );
    }
}

fn bind(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let add_action = gio::SimpleAction::new("preferences.feeds.add", None);
    let edit_action = gio::SimpleAction::new("preferences.feeds.edit", None);
    let delete_action = gio::SimpleAction::new("preferences.feeds.delete", None);
    
    // add feed
    add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FeedsAdd)).unwrap()
    });
    
    // edit feed
    edit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FeedsEdit)).unwrap()
    });
    
    // delete feed
    delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FeedsDelete)).unwrap()
    });
    
    app.add_action(&add_action);
    app.add_action(&edit_action);
    app.add_action(&delete_action);
    
    // ---------- treeviews ----------
    
    let feeds_treeview = &state.ui.widgets().window.preferences.feeds.treeview;
    
    // add feed (Insert)
    // edit feed (F2)
    // delete feed (Delete)
    feeds_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Insert => sender_cloned.send(Message::Preferences(PreferencesActions::FeedsAdd)).unwrap(),
                gdk::keys::constants::F2 => sender_cloned.send(Message::Preferences(PreferencesActions::FeedsEdit)).unwrap(),
                gdk::keys::constants::Delete => sender_cloned.send(Message::Preferences(PreferencesActions::FeedsDelete)).unwrap(),
                _ => (),
            }
            Inhibit(false)
        }
    });
    
    // edit feed (Double-click, Return, Space)
    feeds_treeview.connect_row_activated({
        let sender_cloned = sender.clone();
        move |_, _, _| sender_cloned.send(Message::Preferences(PreferencesActions::FeedsEdit)).unwrap()
    });
    
    // focus global search entry (SHIFT + Tab)
    feeds_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
                sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                return Inhibit(true);
            }
            Inhibit(false)
        }
    });
    
    // ---------- buttons ----------
    
    for button in &state.ui.widgets().window.preferences.feeds.buttons_box.children() {
        
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
    
    if let Some(button) = state.ui.widgets().window.preferences.feeds.buttons_box.children().first() {
        
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

pub fn add(state: &mut State) {
    let feeds_dialog = &state.ui.widgets().dialogs.preferences.feeds.dialog;
    
    feeds_dialog.set_title("Add feed");
    
    let url_entry = &state.ui.widgets().dialogs.preferences.feeds.url_entry;
    url_entry.set_text("");
    
    loop {
        
        url_entry.grab_focus();
        
        let response = feeds_dialog.run();
        
        feeds_dialog.unrealize();
        feeds_dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Ok => {
                
                let entry = FeedsEntry::new()
                    .with_url(url_entry.text().to_string());
                
                match state.database.feeds_add(entry) {
                    
                    Ok(id) => {
                        
                        let feed = state.database.feeds_get(id).unwrap();
                        
                        let feeds_treeview = &state.ui.widgets().window.preferences.feeds.treeview;
                        
                        let feeds_store = &state.ui.widgets().stores.preferences.feeds.store;
                        let feeds_sort = &state.ui.widgets().stores.preferences.feeds.sort;
                        
                        let store_iter = feeds_store.insert_with_values(
                            None,
                            &[
                                (0, &i64::from(id)),
                                (1, &feed.url()),
                            ],
                        );
                        
                        let sort_iter = feeds_sort.convert_child_iter_to_iter(&store_iter).unwrap();
                        let treepath = feeds_sort.path(&sort_iter).unwrap();
                        feeds_treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
                        feeds_treeview.grab_focus();
                        
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

pub fn edit(state: &mut State) {
    let feeds_treeview = &state.ui.widgets().window.preferences.feeds.treeview;
    
    let Some((treemodel, treeiter)) = feeds_treeview.selection().selected() else {
        return;
    };
    
    let id = FeedsId::from(treemodel.value(&treeiter, 0).get::<i64>().unwrap());
    
    match state.database.feeds_get(id) {
        
        Some(previous) => {
            
            let feeds_dialog = &state.ui.widgets().dialogs.preferences.feeds.dialog;
            
            feeds_dialog.set_title("Edit feed");
            
            let url_entry = &state.ui.widgets().dialogs.preferences.feeds.url_entry;
            
            url_entry.set_text(previous.url());
            
            loop {
                
                url_entry.grab_focus();
                
                let response = feeds_dialog.run();
                
                feeds_dialog.unrealize();
                feeds_dialog.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Ok => {
                        
                        let entry = FeedsEntry::new()
                            .with_url(url_entry.text().to_string());
                        
                        match state.database.feeds_edit(id, entry) {
                            
                            Ok(_) => {
                                
                                let feed = state.database.feeds_get(id).unwrap();
                                
                                let feeds_store = &state.ui.widgets().stores.preferences.feeds.store;
                                let feeds_sort = &state.ui.widgets().stores.preferences.feeds.sort;
                                
                                let store_iter = feeds_sort.convert_iter_to_child_iter(&treeiter);
                                
                                feeds_store.set_value(&store_iter, 1, &feed.url().to_value());
                                
                                feeds_treeview.grab_focus();
                                
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
        
        None => state.ui.dialogs_error_show("Feed not found"),
        
    }
}

pub fn delete(state: &mut State) {
    let Some((treemodel, treeiter)) = state.ui.widgets().window.preferences.feeds.treeview.selection().selected() else {
        return;
    };
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        let id = FeedsId::from(treemodel.value(&treeiter, 0).get::<i64>().unwrap());
        
        match state.database.feeds_remove(id) {
            
            Ok(_) => {
                
                let feeds_store = &state.ui.widgets().stores.preferences.feeds.store;
                let feeds_sort = &state.ui.widgets().stores.preferences.feeds.sort;
                
                let store_iter = feeds_sort.convert_iter_to_child_iter(&treeiter);
                
                feeds_store.remove(&store_iter);
                
            },
            
            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
            
        }
        
    }
}

pub fn reload(state: &State) {
    // ---------- widget ----------
    
    let feeds_sort = &state.ui.widgets().stores.preferences.feeds.sort;
    
    // ---------- unset model ----------
    
    state.ui.widgets().window.preferences.feeds.treeview.set_model(None::<&gtk::TreeModel>);
    
    // ---------- unset sort ----------
    
    feeds_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ----------
    
    feeds_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().window.preferences.feeds.treeview.set_model(Some(feeds_sort));
}
