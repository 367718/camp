use std::{
    collections::HashMap,
    path::Path,
};

use gtk::{
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    FilesActions, GeneralActions,
    Files, FilesMark, FilesWatcherEvent,
};

pub fn init(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    build(state, sender);
    bind(app, sender);
}

fn build(state: &mut State, sender: &Sender<Message>) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widgets ----------
    
    let new_sort = &state.ui.widgets().stores.files.entries.new_sort;
    let watched_sort = &state.ui.widgets().stores.files.entries.watched_sort;
    
    // ---------- set sort ids ----------
    
    new_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    watched_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().window.files.new_treeview.set_model(Some(new_sort));
    state.ui.widgets().window.files.watched_treeview.set_model(Some(watched_sort));
    
    // ---------- file watcher ----------
    
    mount_watcher(state, sender);
}

fn bind(app: &gtk::Application, sender: &Sender<Message>) {
    // ---------- global hotkeys ----------
    
    app.set_accels_for_action("app.files.file.directory", &["<Primary>R"]);
    app.set_accels_for_action("app.files.tools.download", &["<Primary>D"]);
    app.set_accels_for_action("app.files.tools.update", &["<Primary>U"]);
    app.set_accels_for_action("app.files.tools.remote", &["<Primary>O"]);
    
    // ---------- actions ----------
    
    let reload_action = gio::SimpleAction::new("files.general.reload", None);
    
    // reload files
    reload_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Reload)).unwrap()
    });
    
    app.add_action(&reload_action);
}

fn fill(state: &State) {
    // 0 => path (empty if container)
    // 1 => updated (strikethrough)
    // 2 => watched
    // 3 => file stem
    
    let files_store = &state.ui.widgets().stores.files.entries.store;
    files_store.clear();
    
    let mut containers: HashMap<(bool, &str), gtk::TreeIter> = HashMap::with_capacity(state.files.count());
    
    for entry in state.files.iter() {
        
        let Some(name) = entry.name().to_str() else {
            continue;
        };
        
        let Some(path) = entry.path().to_str() else {
            continue;
        };
        
        let container_iter = match entry.container().as_ref() {
            
            Some(container) => {
                
                let Some(container) = container.to_str() else {
                    continue;
                };
                
                Some(&*containers.entry((entry.mark() != FilesMark::None, container)).or_insert_with(|| {
                    files_store.insert_with_values(
                        None,
                        None,
                        &[
                            (0, &""),
                            
                            (1, &false),
                            (2, &(entry.mark() != FilesMark::None)),
                            
                            (3, &container),
                        ],
                    )
                }))
                
            },
            
            None => None,
            
        };
        
        files_store.insert_with_values(
            container_iter,
            None,
            &[
                (0, &path),
                
                (1, &(entry.mark() == FilesMark::Updated)),
                (2, &(entry.mark() != FilesMark::None)),
                
                (3, &name),
            ],
        );
        
    }
}

pub fn add(state: &mut State, sender: &Sender<Message>, path: &Path) {
    if let Ok(added) = state.files.add(path) {
        
        let files_store = &state.ui.widgets().stores.files.entries.store;
        
        for entry in added {
            
            let Some(name) = entry.name().to_str() else {
                continue;
            };
            
            let Some(path) = entry.path().to_str() else {
                continue;
            };
            
            let container_iter = match entry.container().as_ref() {
                
                Some(container) => {
                    
                    let Some(container) = container.to_str() else {
                        continue;
                    };
                    
                    let mut result = None;
                    
                    // subdirectory is already present
                    
                    files_store.foreach(|_, _, store_iter| {
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let current = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        if watched == (entry.mark() != FilesMark::None) && current == container {
                            result = Some(*store_iter);
                            return true;
                        }
                        
                        false
                    });
                    
                    // subdirectory must be inserted
                    
                    if result.is_none() {
                        
                        result = Some(files_store.insert_with_values(
                            None,
                            None,
                            &[
                                (0, &""),
                                
                                (1, &false),
                                (2, &(entry.mark() != FilesMark::None)),
                                
                                (3, &container),
                            ],
                        ));
                        
                    }
                    
                    result
                    
                },
                
                None => None,
                
            };
            
            files_store.insert_with_values(
                container_iter.as_ref(),
                None,
                &[
                    (0, &path),
                    
                    (1, &(entry.mark() == FilesMark::Updated)),
                    (2, &(entry.mark() != FilesMark::None)),
                    
                    (3, &name),
                ],
            );
            
        }
        
        sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
        
    }
}

pub fn remove(state: &mut State, sender: &Sender<Message>, path: &Path) {
    if let Ok(removed) = state.files.remove(path) {
        
        let files_store = &state.ui.widgets().stores.files.entries.store;
        
        for entry in removed {
            
            let Some(search) = entry.path().to_str() else {
                continue;
            };
            
            files_store.foreach(|_, _, store_iter| {
                let current = files_store.value(store_iter, 0).get::<glib::GString>().unwrap();
                
                if current == search {
                    // remove the parent rather than the child if it's going to end up empty
                    match files_store.iter_parent(store_iter).filter(|parent_iter| files_store.iter_n_children(Some(parent_iter)) == 1) {
                        Some(parent_iter) => files_store.remove(&parent_iter),
                        None => files_store.remove(store_iter),
                    };
                    
                    return true;
                }
                
                false
            });
            
        }
        
        sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
        
    }
}

pub fn reload(state: &mut State, sender: &Sender<Message>) {
    // ---------- state ----------
    
    state.files = Files::new(
        state.params.paths_files(true),
        state.params.media_flag(true),
        state.database.formats_iter().map(|(_, entry)| entry.name()),
    );
    
    // ---------- widgets ----------
    
    let new_sort = &state.ui.widgets().stores.files.entries.new_sort;
    let watched_sort = &state.ui.widgets().stores.files.entries.watched_sort;
    
    // ---------- unset models ----------
    
    state.ui.widgets().window.files.new_treeview.set_model(None::<&gtk::TreeModel>);
    state.ui.widgets().window.files.watched_treeview.set_model(None::<&gtk::TreeModel>);
    
    // ---------- unset sort ids ----------
    
    new_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    watched_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ids ----------
    
    new_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    watched_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().window.files.new_treeview.set_model(Some(new_sort));
    state.ui.widgets().window.files.watched_treeview.set_model(Some(watched_sort));
    
    // ---------- remount watcher ----------
    
    mount_watcher(state, sender);
    
    // ---------- global search ----------
    
    sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
}

fn mount_watcher(state: &mut State, sender: &Sender<Message>) {
    let sender_cloned = sender.clone();
    
    let subscription = move |event| {
        match event {
            
            FilesWatcherEvent::FileAdded(path) => sender_cloned.send(Message::Files(FilesActions::Add(path))).unwrap(),
            FilesWatcherEvent::FileRemoved(path) => sender_cloned.send(Message::Files(FilesActions::Remove(path))).unwrap(),
            
            FilesWatcherEvent::Interrupted(_) => sender_cloned.send(Message::Files(FilesActions::ShowFrame(true))).unwrap(),
            
        }
    };
    
    let mounted = state.files.mount_watcher(subscription);
    
    sender.send(Message::Files(FilesActions::ShowFrame(mounted.is_err()))).unwrap();
}

pub fn show_frame(state: &State, show: bool) {
    state.ui.widgets().window.files.frame.set_visible(show);
}
