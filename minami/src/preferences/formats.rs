use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, FilesActions, GeneralActions,
    FormatsId, FormatsEntry,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widget ----------
    
    let formats_sort = &state.ui.widgets().stores.preferences.formats.sort;
    
    // ---------- set sort ----------
    
    formats_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().window.preferences.formats.treeview.set_model(Some(formats_sort));
}

fn fill(state: &State) {
    // 0 => id
    // 1 => name
    
    let formats_store = &state.ui.widgets().stores.preferences.formats.store;
    formats_store.clear();
    
    for (id, format) in state.database.formats_iter() {
        formats_store.insert_with_values(
            None,
            &[
                (0, &id.to_int()),
                (1, &format.name()),
            ],
        );
    }
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let add_action = gio::SimpleAction::new("preferences.formats.add", None);
    let edit_action = gio::SimpleAction::new("preferences.formats.edit", None);
    let delete_action = gio::SimpleAction::new("preferences.formats.delete", None);
    
    // add format
    add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FormatsAdd)).unwrap()
    });
    
    // edit format
    edit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FormatsEdit)).unwrap()
    });
    
    // delete format
    delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::FormatsDelete)).unwrap()
    });
    
    app.add_action(&add_action);
    app.add_action(&edit_action);
    app.add_action(&delete_action);
    
    // ---------- treeviews ----------
    
    let formats_treeview = &state.ui.widgets().window.preferences.formats.treeview;
    
    // add format (Insert)
    // edit format (F2)
    // delete format (Delete)
    formats_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Insert => sender_cloned.send(Message::Preferences(PreferencesActions::FormatsAdd)).unwrap(),
                gdk::keys::constants::F2 => sender_cloned.send(Message::Preferences(PreferencesActions::FormatsEdit)).unwrap(),
                gdk::keys::constants::Delete => sender_cloned.send(Message::Preferences(PreferencesActions::FormatsDelete)).unwrap(),
                _ => (),
            }
            glib::Propagation::Proceed
        }
    });
    
    // edit feed (Double-click, Return, Space)
    formats_treeview.connect_row_activated({
        let sender_cloned = sender.clone();
        move |_, _, _| sender_cloned.send(Message::Preferences(PreferencesActions::FormatsEdit)).unwrap()
    });
    
    // focus global search entry (SHIFT + Tab)
    formats_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
                sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    });
    
    // ---------- buttons ----------
    
    for button in &state.ui.widgets().window.preferences.formats.buttons_box.children() {
        
        // prevent selection of treeview (Up Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Up {
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        });
        
    }
    
    if let Some(button) = state.ui.widgets().window.preferences.formats.buttons_box.children().first() {
        
        // prevent selection of preferences listbox (Left Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Left {
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        });
        
    }
}

pub fn add(state: &mut State, sender: &Sender<Message>) {
    let formats_dialog = &state.ui.widgets().dialogs.preferences.formats.dialog;
    
    formats_dialog.set_title("Add format");
    
    let name_entry = &state.ui.widgets().dialogs.preferences.formats.name_entry;
    name_entry.set_text("");
    
    loop {
        
        name_entry.grab_focus();
        
        let response = formats_dialog.run();
        
        formats_dialog.unrealize();
        formats_dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Ok => {
                
                let entry = FormatsEntry::new()
                    .with_name(name_entry.text().to_string());
                
                match state.database.formats_add(entry) {
                    
                    Ok(id) => {
                        
                        let format = state.database.formats_get(id).unwrap();
                        
                        let formats_treeview = &state.ui.widgets().window.preferences.formats.treeview;
                        
                        let formats_store = &state.ui.widgets().stores.preferences.formats.store;
                        let formats_sort = &state.ui.widgets().stores.preferences.formats.sort;
                        
                        let store_iter = formats_store.insert_with_values(
                            None,
                            &[
                                (0, &id.to_int()),
                                (1, &format.name()),
                            ],
                        );
                        
                        let sort_iter = formats_sort.convert_child_iter_to_iter(&store_iter).unwrap();
                        let treepath = formats_sort.path(&sort_iter).unwrap();
                        formats_treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
                        formats_treeview.grab_focus();
                        
                        sender.send(Message::Files(FilesActions::Reload)).unwrap();
                        
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
    let formats_treeview = &state.ui.widgets().window.preferences.formats.treeview;
    
    let Some((treemodel, treeiter)) = formats_treeview.selection().selected() else {
        return;
    };
    
    let id = FormatsId::from(treemodel.value(&treeiter, 0).get::<i64>().unwrap());
    
    match state.database.formats_get(id) {
        
        Some(previous) => {
            
            let formats_dialog = &state.ui.widgets().dialogs.preferences.formats.dialog;
            
            formats_dialog.set_title("Edit format");
            
            let name_entry = &state.ui.widgets().dialogs.preferences.formats.name_entry;
            
            name_entry.set_text(previous.name());
            
            loop {
                
                name_entry.grab_focus();
                
                let response = formats_dialog.run();
                
                formats_dialog.unrealize();
                formats_dialog.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Ok => {
                        
                        let entry = FormatsEntry::new()
                            .with_name(name_entry.text().to_string());
                        
                        match state.database.formats_edit(id, entry) {
                            
                            Ok(_) => {
                                
                                let format = state.database.formats_get(id).unwrap();
                                
                                let formats_store = &state.ui.widgets().stores.preferences.formats.store;
                                let formats_sort = &state.ui.widgets().stores.preferences.formats.sort;
                                
                                let store_iter = formats_sort.convert_iter_to_child_iter(&treeiter);
                                
                                formats_store.set_value(&store_iter, 1, &format.name().to_value());
                                
                                formats_treeview.grab_focus();
                                
                                sender.send(Message::Files(FilesActions::Reload)).unwrap();
                                
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
        
        None => state.ui.dialogs_error_show("Format not found"),
        
    }
}

pub fn delete(state: &mut State, sender: &Sender<Message>) {
    let Some((treemodel, treeiter)) = state.ui.widgets().window.preferences.formats.treeview.selection().selected() else {
        return;
    };
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        let id = FormatsId::from(treemodel.value(&treeiter, 0).get::<i64>().unwrap());
        
        match state.database.formats_remove(id) {
            
            Ok(_) => {
                
                let formats_store = &state.ui.widgets().stores.preferences.formats.store;
                let formats_sort = &state.ui.widgets().stores.preferences.formats.sort;
                
                let store_iter = formats_sort.convert_iter_to_child_iter(&treeiter);
                
                formats_store.remove(&store_iter);
                
                sender.send(Message::Files(FilesActions::Reload)).unwrap();
                
            },
            
            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
            
        }
        
    }
}

pub fn reload(state: &State, sender: &Sender<Message>) {
    // ---------- widget ----------
    
    let formats_sort = &state.ui.widgets().stores.preferences.formats.sort;
    
    // ---------- unset model ----------
    
    state.ui.widgets().window.preferences.formats.treeview.set_model(None::<&gtk::TreeModel>);
    
    // ---------- unset sort ----------
    
    formats_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ----------
    
    formats_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().window.preferences.formats.treeview.set_model(Some(formats_sort));
    
    // ---------- files ----------
    
    sender.send(Message::Files(FilesActions::Reload)).unwrap();
}
