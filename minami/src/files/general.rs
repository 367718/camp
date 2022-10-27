use std::{
    collections::HashMap,
    path::Path,
};

use gtk::{
    gdk,
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
    build(app, state, sender);
    bind(app, state, sender);
}

fn build(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- menus ----------
    
    state.ui.widgets().menus.files.popup.menu.insert_action_group("app", Some(app));
    
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

fn bind(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
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
    
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.files.new_treeview,
        &state.ui.widgets().window.files.watched_treeview,
    ];
    
    for treeview in treeviews {
        
        // open popup menu (Right-click)
        treeview.connect_button_release_event({
            let sender_cloned = sender.clone();
            move |_, button| {
                if button.button() == 3 {
                    sender_cloned.send(Message::Files(FilesActions::MenuPopup(button.coords()))).unwrap();
                }
                Inhibit(false)
            }
        });
        
        // open popup menu (Menu key)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, key| {
                if *key.keyval() == 65_383 {
                    sender_cloned.send(Message::Files(FilesActions::MenuPopup(None))).unwrap();
                }
                Inhibit(false)
            }
        });
        
    }
    
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
        
        let name = match entry.name.to_str() {
            Some(name) => name,
            _ => continue,
        };
        
        let path = match entry.path.to_str() {
            Some(path) => path,
            _ => continue,
        };
        
        // subdirectory
        
        let container_iter = match entry.container.as_ref() {
            
            Some(container) => {
                
                let container = match container.to_str() {
                    Some(container) => container,
                    None => continue,
                };
                
                Some(&*containers.entry((entry.mark != FilesMark::None, container)).or_insert_with(|| {
                    files_store.insert_with_values(
                        None,
                        None,
                        &[
                            (0, &""),
                            
                            (1, &false),
                            (2, &(entry.mark != FilesMark::None)),
                            
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
                
                (1, &(entry.mark == FilesMark::Updated)),
                (2, &(entry.mark != FilesMark::None)),
                
                (3, &name),
            ],
        );
        
    }
}

pub fn add(state: &mut State, sender: &Sender<Message>, path: &Path) {
    if let Ok(added) = state.files.add(path) {
        
        let files_store = &state.ui.widgets().stores.files.entries.store;
        
        for entry in added {
            
            let name = match entry.name.to_str() {
                Some(name) => name,
                _ => continue,
            };
            
            let path = match entry.path.to_str() {
                Some(path) => path,
                _ => continue,
            };
            
            let container_iter = match entry.container.as_ref() {
                
                Some(container) => {
                    
                    let container = match container.to_str() {
                        Some(container) => container,
                        None => continue,
                    };
                    
                    let mut result = None;
                    
                    // subdirectory is already present
                    
                    files_store.foreach(|_, _, store_iter| {
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let current = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        if watched == (entry.mark != FilesMark::None) && current == container {
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
                                (2, &(entry.mark != FilesMark::None)),
                                
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
                    
                    (1, &(entry.mark == FilesMark::Updated)),
                    (2, &(entry.mark != FilesMark::None)),
                    
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
            
            let search = match entry.path.to_str() {
                Some(search) => search,
                None => continue,
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

pub fn menu_popup(state: &State, coords: Option<(f64, f64)>) {
    let treeview = match state.ui.files_current_treeview() {
        Some(treeview) => treeview,
        None => return,
    };
    
    let (treepaths, _) = treeview.selection().selected_rows();
    
    if treepaths.is_empty() {
        return;
    }
    
    let files_popup = &state.ui.widgets().menus.files.popup.menu;
    
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
                
                files_popup.set_rect_anchor_dx(x);
                files_popup.set_rect_anchor_dy(y);
                
                files_popup.popup_at_widget(
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
        
        files_popup.set_rect_anchor_dx(5);
        files_popup.set_rect_anchor_dy(rect.y() + rect.height() + 5);
        
        files_popup.popup_at_widget(
            treeview,
            gdk::Gravity::NorthWest,
            gdk::Gravity::NorthWest,
            Some(&event),
        );
        
    }
}

pub fn reload(state: &mut State, sender: &Sender<Message>) {
    // ---------- state ----------
    
    state.files = Files::new(
        state.params.paths_files(true),
        state.params.media_flag(true),
        &state.database.formats_iter()
            .map(|(_, entry)| entry.name.as_ref())
            .collect::<Vec<&str>>(),
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
