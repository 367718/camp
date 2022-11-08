use std::{
    collections::HashSet,
    ffi::OsStr,
    path::PathBuf,
    process::{ Command, Stdio },
};

use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message, FilesSection,
    FilesActions,
    FilesMark,
    SeriesId,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    bind(app, state, sender);
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let play_action = gio::SimpleAction::new("files.file.play", None);
    let mark_action = gio::SimpleAction::new("files.file.mark", None);
    let rename_action = gio::SimpleAction::new("files.file.rename", None);
    let move_action = gio::SimpleAction::new("files.file.move", None);
    let delete_action = gio::SimpleAction::new("files.file.delete", None);
    let maintenance_action = gio::SimpleAction::new("files.file.maintenance", None);
    let directory_action = gio::SimpleAction::new("files.file.directory", None);
    
    // play files in queue
    play_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Play)).unwrap()
    });
    
    // mark files as watched
    mark_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::MarkAsWatched)).unwrap()
    });
    
    // rename first selected file
    rename_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Rename)).unwrap()
    });
    
    // move files to specified folder
    move_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::MoveToFolder)).unwrap()
    });
    
    // delete files
    delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Delete)).unwrap()
    });
    
    // perform maintenance
    maintenance_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Maintenance)).unwrap()
    });
    
    // open files directory
    directory_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::OpenDirectory)).unwrap()
    });
    
    app.add_action(&play_action);
    app.add_action(&mark_action);
    app.add_action(&rename_action);
    app.add_action(&move_action);
    app.add_action(&delete_action);
    app.add_action(&maintenance_action);
    app.add_action(&directory_action);
    
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.files.new_treeview,
        &state.ui.widgets().window.files.watched_treeview,
    ];
    
    for treeview in treeviews {
        
        // rename first selected file (F2)
        // move files to specified folder (F3)
        // mark as watched (Delete)
        // delete files (SHIFT + Delete)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                match eventkey.keyval() {
                    
                    gdk::keys::constants::F2 => sender_cloned.send(Message::Files(FilesActions::Rename)).unwrap(),
                    gdk::keys::constants::F3 => sender_cloned.send(Message::Files(FilesActions::MoveToFolder)).unwrap(),
                    
                    key if key == gdk::keys::constants::Delete && ! eventkey.state().contains(gdk::ModifierType::SHIFT_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::MarkAsWatched)).unwrap();
                    },
                    
                    key if key == gdk::keys::constants::Delete && eventkey.state().contains(gdk::ModifierType::SHIFT_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::Delete)).unwrap();
                    },
                    
                    _ => return Inhibit(false),
                    
                }
                
                Inhibit(true)
            }
        });
        
        // play files in queue (Double-click, Return, Space)
        treeview.connect_row_activated({
            let sender_cloned = sender.clone();
            move |_, _, _| sender_cloned.send(Message::Files(FilesActions::Play)).unwrap()
        });
        
        // mark as watched (Middle-click)
        treeview.connect_event({
            let sender_cloned = sender.clone();
            move |_, event| {
                if event.event_type() == gdk::EventType::ButtonRelease && event.button().unwrap() == 2 {
                    sender_cloned.send(Message::Files(FilesActions::MarkAsWatched)).unwrap();
                }
                Inhibit(false)
            }
        });
        
        // refresh queue on each selection change
        treeview.selection().connect_changed({
            let sender_cloned = sender.clone();
            move |_| sender_cloned.send(Message::Files(FilesActions::RefreshQueue)).unwrap()
        });
        
    }
}

pub fn play(state: &mut State) {
    if let Some(list) = state.ui.widgets().window.files.listbox.selected_row() {
        
        // make sure changes to previously selected directories are picked up
        refresh_queue(state);
        
        let mut args = state.params.media_player(true)
            .split(' ')
            .map(OsStr::new);
        
        let Some(player) = args.next() else {
            return;
        };
        
        let unmarked = list.widget_name() == FilesSection::New.display();
        
        let result = Command::new(player)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(args.chain(
                state.files.queue()
                .filter(|file| (file.mark == FilesMark::None) == unmarked)
                .map(|file| file.path.as_os_str())
            ))
            .spawn();
        
        if let Err(error) = result {
            state.ui.dialogs_error_show(&error.to_string());
        } else if state.params.media_iconify(true) {
            state.ui.widgets().window.general.window.iconify();
        }
        
    }
}

pub fn rename(state: &State) {
    let Some(treeview) = state.ui.files_current_treeview() else {
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
    let filepath = treemodel.value(&treeiter, 0).get::<glib::GString>().unwrap();
    
    if filepath.is_empty() {
        state.ui.dialogs_error_show(r#"Renaming directores is not allowed; "Move to folder" may be used instead"#);
        return;
    }
    
    let file_stem = treemodel.value(&treeiter, 3).get::<glib::GString>().unwrap();
    
    let rename_dialog = &state.ui.widgets().dialogs.files.rename.dialog;
    let new_entry = &state.ui.widgets().dialogs.files.rename.new_entry;
    
    state.ui.widgets().dialogs.files.rename.current_label.set_label(&file_stem);
    new_entry.set_text(&file_stem);
    
    loop {
        
        new_entry.grab_focus();
        
        let response = rename_dialog.run();
        
        rename_dialog.unrealize();
        rename_dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Ok => {
                
                if let Err(error) = state.files.rename(&filepath, &new_entry.text()) {
                    state.ui.dialogs_error_show(&error.to_string());
                    continue;
                }
                
                break;
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

pub fn move_to_folder(state: &State) {
    let Some(treeview) = state.ui.files_current_treeview() else {
        return;
    };
    
    let (treepaths, treemodel) = treeview.selection().selected_rows();
    
    if treepaths.is_empty() {
        return;
    }
    
    let filepaths = selected_filepaths(state);
    
    let treeiter = treemodel.iter(treepaths.first().unwrap()).unwrap();
    let file_stem = treemodel.value(&treeiter, 3).get::<glib::GString>().unwrap();
    
    let folder_entry = &state.ui.widgets().dialogs.files.move_to_folder.folder_entry;
    
    // use first selected file stem as initial value
    folder_entry.set_text(&file_stem);
    
    folder_entry.grab_focus();
    
    let mut containers = HashSet::new();
    
    let files_store = &state.ui.widgets().stores.files.entries.store;
    
    // compute folder name suggestions
    
    let move_to_folder_store = &state.ui.widgets().stores.files.move_to_folder.store;
    let move_to_folder_sort = &state.ui.widgets().stores.files.move_to_folder.sort;
    
    move_to_folder_store.clear();
    
    state.ui.widgets().dialogs.files.move_to_folder.folder_completion.set_model(None::<&gtk::TreeModel>);
    move_to_folder_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    if let Some(treeiter) = files_store.iter_first() {
        loop {
            
            if files_store.iter_has_child(&treeiter) {
                let container = files_store.value(&treeiter, 3).get::<glib::GString>().unwrap();
                
                if ! containers.contains(&container) {
                    
                    move_to_folder_store.insert_with_values(
                        None,
                        &[
                            (0, &container),
                        ],
                    );
                    
                    containers.insert(container);
                    
                }
            }
            
            if ! files_store.iter_next(&treeiter) {
                break;
            }
            
        }
    }
    
    move_to_folder_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    state.ui.widgets().dialogs.files.move_to_folder.folder_completion.set_model(Some(move_to_folder_sort));
    
    let move_to_folder_dialog = &state.ui.widgets().dialogs.files.move_to_folder.dialog;
    let response = move_to_folder_dialog.run();
    
    move_to_folder_dialog.unrealize();
    move_to_folder_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        let input = folder_entry.text();
        
        let folder_name = if input.is_empty() {
            None
        } else {
            Some(input)
        };
        
        for path in &filepaths {
            
            if let Err(error) = state.files.move_to_folder(path, folder_name.as_ref()) {
                state.ui.dialogs_error_show(&error.to_string());
                break;
            }
            
        }
        
    }
}

pub fn delete(state: &mut State) {
    let filepaths = selected_filepaths(state);
    
    if filepaths.is_empty() {
        return;
    }
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        for path in &filepaths {
            
            if let Err(error) = state.files.delete(path) {
                state.ui.dialogs_error_show(&error.to_string());
                break;
            }
            
        }
        
    }
}

pub fn mark_as_watched(state: &mut State) {
    if let Some(list) = state.ui.widgets().window.files.listbox.selected_row() {
        
        // unset mark
        let mark = if list.widget_name() == FilesSection::Watched.display() {
            FilesMark::None
        } else {
            FilesMark::Watched
        };
        
        for path in selected_filepaths(state) {
            
            if let Some(file) = state.files.get(&path) {
                
                if file.mark == FilesMark::Updated {
                    state.ui.dialogs_error_show(r#"A file marked as "Updated" cannot be unmarked as "Watched""#);
                    break;
                }
                
                if let Err(error) = state.files.mark(&path, mark) {
                    state.ui.dialogs_error_show(&error.to_string());
                    break;
                }
                
                continue;
                
            }
            
            state.ui.dialogs_error_show("File not found");
            break;
            
        }
        
    }
}

pub fn mark_as_updated(state: &mut State, updates: Vec<(SeriesId, u32, PathBuf)>) {
    if updates.is_empty() {
        return;
    }
    
    let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
    
    for (id, episode, path) in updates {
        
        let Some(current) = state.database.series_get(id) else {
            continue;
        };
        
        if current.progress < episode {
            
            let mut new = current.clone();
            new.progress = episode;
            
            if state.database.series_edit(id, new).is_err() {
                continue;
            }
            
            watchlist_store.foreach(|_, _, store_iter| {
                let current = SeriesId::from(watchlist_store.value(store_iter, 0).get::<u32>().unwrap());
                
                if current == id {
                    watchlist_store.set(
                        store_iter,
                        &[
                            (6, &episode),
                        ],
                    );
                    
                    return true;
                }
                
                false
            });
            
        }
        
        state.files.mark(&path, FilesMark::Updated).ok();
        
    }
}

pub fn refresh_queue(state: &mut State) {
    
    fn process_row(treemodel: &gtk::TreeModel, treeiter: &gtk::TreeIter, selected: &mut Vec<glib::GString>) {
        match treemodel.iter_children(Some(treeiter)) {
            
            // subdirectory
            
            Some(iter_child) => {
                
                selected.reserve(treemodel.iter_n_children(Some(treeiter)) as usize);
                
                // directory
                let file_stem = treemodel.value(treeiter, 3).get::<glib::GString>().unwrap();
                selected.push(file_stem);
                
                loop {
                    
                    let filepath = treemodel.value(&iter_child, 0).get::<glib::GString>().unwrap();
                    selected.push(filepath);
                    
                    if ! treemodel.iter_next(&iter_child) {
                        break;
                    }
                    
                }
                
            },
            
            // file
            
            None => {
                
                let filepath = treemodel.value(treeiter, 0).get::<glib::GString>().unwrap();
                selected.push(filepath);
                
            },
            
        }
        
    }
    
    let new_selection = state.ui.widgets().window.files.new_treeview.selection();
    let watched_selection = state.ui.widgets().window.files.watched_treeview.selection();
    
    let mut selected = Vec::with_capacity((new_selection.count_selected_rows() + watched_selection.count_selected_rows()) as usize);
    
    new_selection.selected_foreach(|treemodel, _, treeiter| process_row(treemodel, treeiter, &mut selected));
    watched_selection.selected_foreach(|treemodel, _, treeiter| process_row(treemodel, treeiter, &mut selected));
    
    state.files.refresh_queue(&selected);
    
}

pub fn maintenance(state: &mut State) {
    let maintenance_dialog = &state.ui.widgets().dialogs.files.maintenance.dialog;
    
    let response = maintenance_dialog.run();
    
    maintenance_dialog.unrealize();
    maintenance_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        if let Err(error) = state.files.perform_maintenance() {
            state.ui.dialogs_error_show(&error.to_string());
        }
        
    }
}

pub fn open_directory(state: &State) {
    let root = state.params.paths_files(true);
    
    if root.is_dir() {
        
        if let Some(root) = root.to_str() {
            
            if let Err(error) = crate::general::open(root) {
                state.ui.dialogs_error_show(&error.to_string());
            }
            
            return;
            
        }
        
    }
    
    state.ui.dialogs_error_show("Files directory is not valid");
}

fn selected_filepaths(state: &State) -> Vec<glib::GString> {
    let mut filepaths = Vec::new();
    
    if let Some(treeview) = state.ui.files_current_treeview() {
        
        let selection = treeview.selection();
        
        filepaths.reserve(selection.count_selected_rows() as usize);
        
        selection.selected_foreach(|treemodel, _, treeiter| {
            match treemodel.iter_children(Some(treeiter)) {
                
                // subdirectory
                
                Some(iter_child) => {
                    
                    filepaths.reserve(treemodel.iter_n_children(Some(treeiter)) as usize);
                    
                    loop {
                        
                        let filepath = treemodel.value(&iter_child, 0).get::<glib::GString>().unwrap();
                        filepaths.push(filepath);
                        
                        if ! treemodel.iter_next(&iter_child) {
                            break;
                        }
                        
                    }
                    
                },
                
                // file
                
                None => {
                    
                    // skip if parent is selected
                    if let Some(parent_iter) = treemodel.iter_parent(treeiter) {
                        if selection.iter_is_selected(&parent_iter) {
                            return;
                        }
                    }
                    
                    let filepath = treemodel.value(treeiter, 0).get::<glib::GString>().unwrap();
                    filepaths.push(filepath);
                    
                },
                
            }
        });
        
    }
    
    filepaths
}
