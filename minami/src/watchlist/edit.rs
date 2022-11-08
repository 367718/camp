use std::error::Error;

use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    WatchlistActions, GeneralActions,
    KindsId,
    SeriesId, SeriesEntry, SeriesStatus, SeriesGood,
    CandidatesId,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    bind(app, state, sender);
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let add_action = gio::SimpleAction::new("watchlist.edit.add", None);
    let edit_action = gio::SimpleAction::new("watchlist.edit.edit", None);
    let delete_action = gio::SimpleAction::new("watchlist.edit.delete", None);
    let copy_action = gio::SimpleAction::new("watchlist.edit.copy", None);
    
    // add series
    add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Watchlist(WatchlistActions::Add(None))).unwrap()
    });
    
    // edit series
    edit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Watchlist(WatchlistActions::Edit)).unwrap()
    });
    
    // delete series
    delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Watchlist(WatchlistActions::Delete)).unwrap()
    });
    
    // copy titles to clipboard
    copy_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Watchlist(WatchlistActions::CopyTitles)).unwrap()
    });
    
    app.add_action(&add_action);
    app.add_action(&edit_action);
    app.add_action(&delete_action);
    app.add_action(&copy_action);
    
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.watchlist.watching_treeview,
        &state.ui.widgets().window.watchlist.on_hold_treeview,
        &state.ui.widgets().window.watchlist.plan_to_watch_treeview,
        &state.ui.widgets().window.watchlist.completed_treeview,
    ];
    
    for treeview in treeviews {
        
        // add series (Insert)
        // edit series (F2)
        // delete series (Delete)
        // copy titles to clipboard (CONTROL + C/c)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                match eventkey.keyval() {
                    
                    gdk::keys::constants::Insert => sender_cloned.send(Message::Watchlist(WatchlistActions::Add(None))).unwrap(),
                    gdk::keys::constants::F2 => sender_cloned.send(Message::Watchlist(WatchlistActions::Edit)).unwrap(),
                    gdk::keys::constants::Delete => sender_cloned.send(Message::Watchlist(WatchlistActions::Delete)).unwrap(),
                    
                    key if (key == gdk::keys::constants::C || key == gdk::keys::constants::c) && eventkey.state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        sender_cloned.send(Message::Watchlist(WatchlistActions::CopyTitles)).unwrap();
                    },
                    
                    _ => return Inhibit(false),
                    
                }
                
                Inhibit(true)
            }
        });
        
        // edit series (Double-click, Return, Space)
        treeview.connect_row_activated({
            let sender_cloned = sender.clone();
            move |_, _, _| sender_cloned.send(Message::Watchlist(WatchlistActions::Edit)).unwrap()
        });
        
    }
}

pub fn add(state: &mut State, sender: &Sender<Message>, prefill: &Option<String>) {
    let series_dialog = &state.ui.widgets().dialogs.watchlist.series.dialog;
    
    series_dialog.set_title("Add series");
    
    // enable "confirm and add another" button
    series_dialog.set_response_sensitive(gtk::ResponseType::Other(1), prefill.is_none());
    
    let title_entry = &state.ui.widgets().dialogs.watchlist.series.title_entry;
    let kind_combo = &state.ui.widgets().dialogs.watchlist.series.kind_combo;
    let status_combo = &state.ui.widgets().dialogs.watchlist.series.status_combo;
    let progress_spin = &state.ui.widgets().dialogs.watchlist.series.progress_spin;
    let good_switch = &state.ui.widgets().dialogs.watchlist.series.good_switch;
    
    title_entry.set_text(prefill.as_deref().unwrap_or(""));
    kind_combo.set_active(Some(0));
    status_combo.set_sensitive(true);
    status_combo.set_active(Some(0));
    progress_spin.set_value(1.0);
    good_switch.set_sensitive(true);
    good_switch.set_active(false);
    
    loop {
        
        title_entry.grab_focus();
        
        let response = series_dialog.run();
        
        // lookup title
        if response != gtk::ResponseType::Other(2) {
            series_dialog.unrealize();
            series_dialog.hide();
        }
        
        match response {
            
            // confirm or add another
            
            gtk::ResponseType::Other(0 | 1) => {
                
                let status = match SeriesStatus::try_from(status_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))) {
                    Ok(value) => value,
                    Err(error) => {
                        state.ui.dialogs_error_show(&error.to_string());
                        continue;
                    }
                };
                
                let new = SeriesEntry {
                    title: title_entry.text().to_string(),
                    kind: KindsId::from(kind_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))),
                    status,
                    progress: progress_spin.text().parse().unwrap_or(0),
                    good: SeriesGood::from(good_switch.is_active()),
                };
                
                match state.database.series_add(new) {
                    
                    Ok(id) => {
                        
                        let series = state.database.series_get(id).unwrap();
                        
                        let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
                        
                        let store_iter = watchlist_store.insert_with_values(
                            None,
                            &[
                                (0, &id.as_int()),
                                
                                (1, &(u32::from(series.good.as_int()) * 400)),
                                (2, &series.status.as_int()),
                                
                                (3, &series.title),
                                
                                (4, &series.good.display()),
                                (5, &state.database.kinds_get(series.kind).map_or("", |kind| &kind.name)),
                                (6, &series.progress),
                            ],
                        );
                        
                        sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
                        
                        if state.params.media_autoselect(true) && prefill.is_none() {
                            
                            for row in state.ui.widgets().window.watchlist.listbox.children() {
                                if row.widget_name() == series.status.display() {
                                    row.activate();
                                    select_series(state, &store_iter);
                                    break;
                                }
                            }
                            
                        }
                        
                        // ---------- confirm ----------
                        
                        if response == gtk::ResponseType::Other(0) {
                            break;
                        }
                        
                        // ---------- confirm and add another ----------
                        
                        // keep status and progress
                        
                        title_entry.set_text("");
                        kind_combo.set_active(Some(0));
                        good_switch.set_active(false);
                        
                    },
                    
                    Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                    
                }
                
            },
            
            // lookup title
            
            gtk::ResponseType::Other(2) => {
                
                let title = title_entry.text();
                
                if ! title.is_empty() {
                    let lookup = state.params.media_lookup(true);
                    let url = lookup.replace("%s", &crate::general::percent_encode(&title));
                    
                    if let Err(error) = crate::general::open(&url) {
                        series_dialog.unrealize();
                        series_dialog.hide();
                        
                        state.ui.dialogs_error_show(&error.to_string());
                    }
                }
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

pub fn edit(state: &mut State, sender: &Sender<Message>) {
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
    let id = SeriesId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
    
    match state.database.series_get(id) {
        
        Some(current) => {
            
            state.ui.widgets().dialogs.watchlist.series.dialog.set_title("Edit series");
            
            // disable "confirm and add another" button
            state.ui.widgets().dialogs.watchlist.series.dialog.set_response_sensitive(gtk::ResponseType::Other(1), false);
            
            let title_entry = &state.ui.widgets().dialogs.watchlist.series.title_entry;
            let kind_combo = &state.ui.widgets().dialogs.watchlist.series.kind_combo;
            let status_combo = &state.ui.widgets().dialogs.watchlist.series.status_combo;
            let progress_spin = &state.ui.widgets().dialogs.watchlist.series.progress_spin;
            let good_switch = &state.ui.widgets().dialogs.watchlist.series.good_switch;
            
            title_entry.set_text(&current.title);
            kind_combo.set_active_id(Some(&current.kind.as_int().to_string()));
            status_combo.set_sensitive(true);
            status_combo.set_active_id(Some(&current.status.as_int().to_string()));
            progress_spin.set_value(f64::from(current.progress));
            good_switch.set_sensitive(true);
            good_switch.set_active(current.good == SeriesGood::Yes);
            
            loop {
                
                state.ui.widgets().dialogs.watchlist.series.title_entry.grab_focus();
                
                let response = state.ui.widgets().dialogs.watchlist.series.dialog.run();
                
                // lookup title
                if response != gtk::ResponseType::Other(2) {
                    state.ui.widgets().dialogs.watchlist.series.dialog.unrealize();
                    state.ui.widgets().dialogs.watchlist.series.dialog.hide();
                }
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Other(0) => {
                        
                        let status = match SeriesStatus::try_from(state.ui.widgets().dialogs.watchlist.series.status_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))) {
                            Ok(value) => value,
                            Err(error) => {
                                state.ui.dialogs_error_show(&error.to_string());
                                continue;
                            }
                        };
                        
                        let new = SeriesEntry {
                            title: state.ui.widgets().dialogs.watchlist.series.title_entry.text().to_string(),
                            kind: KindsId::from(state.ui.widgets().dialogs.watchlist.series.kind_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))),
                            status,
                            progress: state.ui.widgets().dialogs.watchlist.series.progress_spin.text().parse().unwrap_or(0),
                            good: SeriesGood::from(state.ui.widgets().dialogs.watchlist.series.good_switch.is_active()),
                        };
                        
                        if new.status != SeriesStatus::Watching {
                            
                            // delete related candidate, if any
                            if let Err(error) = delete_related_candidate(state, id) {
                                state.ui.dialogs_error_show(&error.to_string());
                                return;
                            }
                            
                        }
                        
                        match state.database.series_edit(id, new) {
                            
                            Ok(_) => {
                                
                                let series = state.database.series_get(id).unwrap();
                                
                                let sort: gtk::TreeModelSort = treemodel.downcast().unwrap();
                                let filter: gtk::TreeModelFilter = sort.model().downcast().unwrap();
                                
                                let filter_iter = sort.convert_iter_to_child_iter(&treeiter);
                                let store_iter = filter.convert_iter_to_child_iter(&filter_iter);
                                
                                let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
                                
                                watchlist_store.set(
                                    &store_iter,
                                    &[
                                        (1, &(u32::from(series.good.as_int()) * 400)),
                                        (2, &series.status.as_int()),
                                        
                                        (3, &series.title),
                                        
                                        (4, &series.good.display()),
                                        (5, &state.database.kinds_get(series.kind).map_or("", |kind| &kind.name)),
                                        (6, &series.progress),
                                    ],
                                );
                                
                                sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
                                
                                if state.params.media_autoselect(true) {
                                    
                                    for row in state.ui.widgets().window.watchlist.listbox.children() {
                                        if row.widget_name() == series.status.display() {
                                            row.activate();
                                            select_series(state, &store_iter);
                                            break;
                                        }
                                    }
                                    
                                }
                                
                                break;
                                
                            },
                            
                            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                            
                        }
                        
                    },
                    
                    // lookup
                    
                    gtk::ResponseType::Other(2) => {
                        
                        let title = state.ui.widgets().dialogs.watchlist.series.title_entry.text();
                        
                        if ! title.is_empty() {
                            let lookup = state.params.media_lookup(true);
                            let url = lookup.replace("%s", &crate::general::percent_encode(&title));
                            
                            if let Err(error) = crate::general::open(&url) {
                                state.ui.widgets().dialogs.watchlist.series.dialog.unrealize();
                                state.ui.widgets().dialogs.watchlist.series.dialog.hide();
                                
                                state.ui.dialogs_error_show(&error.to_string());
                            }
                        }
                        
                    },
                    
                    // cancel
                    
                    _ => break,
                    
                }
                
            }
            
        },
        
        None => state.ui.dialogs_error_show("Series not found"),
        
    }
}

pub fn delete(state: &mut State, sender: &Sender<Message>) {
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
    let id = SeriesId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        // delete related candidate, if any
        if let Err(error) = delete_related_candidate(state, id) {
            state.ui.dialogs_error_show(&error.to_string());
            return;
        }
        
        match state.database.series_remove(id) {
            
            Ok(_) => {
                
                let sort: gtk::TreeModelSort = treemodel.downcast().unwrap();
                let filter: gtk::TreeModelFilter = sort.model().downcast().unwrap();
                
                let filter_iter = sort.convert_iter_to_child_iter(&treeiter);
                let store_iter = filter.convert_iter_to_child_iter(&filter_iter);
                
                let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
                
                watchlist_store.remove(&store_iter);
                
                sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
                
            },
            
            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
            
        }
        
    }
}

pub fn copy_titles(state: &State) {
    let Some(treeview) = state.ui.watchlist_current_treeview() else {
        return;
    };
    
    let selection = treeview.selection();
    
    let count = selection.count_selected_rows();
    
    if count == 0 {
        return;
    }
    
    let mut titles = Vec::with_capacity(count as usize);
    
    selection.selected_foreach(|treemodel, _, treeiter| {
        let title = treemodel.value(treeiter, 3).get::<glib::GString>().unwrap();
        titles.push(title);
    });
    
    let text = titles.join("\n");
    
    state.ui.clipboard_set_text(&text);
}

fn delete_related_candidate(state: &mut State, id: SeriesId) -> Result<(), Box<dyn Error>> {
    let candidate_id = state.database.candidates_iter()
        .find(|(_, current)| current.series == id)
        .map(|(&candidate_id, _)| candidate_id);
    
    if let Some(candidate_id) = candidate_id {
        
        state.database.candidates_remove(candidate_id)?;
        
        let candidates_store = &state.ui.widgets().stores.preferences.candidates.store;
        
        candidates_store.foreach(|_, _, store_iter| {
            let current = CandidatesId::from(candidates_store.value(store_iter, 0).get::<u32>().unwrap());
            
            if current == candidate_id {
                candidates_store.remove(store_iter);
                return true;
            }
            
            false
        });
        
    }
    
    Ok(())
}

fn select_series(state: &State, store_iter: &gtk::TreeIter) {
    let Some(treeview) = state.ui.watchlist_current_treeview() else {
        return;
    };
    
    let sort: gtk::TreeModelSort = treeview.model().unwrap().downcast().unwrap();
    let filter: gtk::TreeModelFilter = sort.model().downcast().unwrap();
    
    if let Some(filter_iter) = filter.convert_child_iter_to_iter(store_iter) {
        if let Some(sort_iter) = sort.convert_child_iter_to_iter(&filter_iter) {
            
            let treepath = filter.path(&sort_iter).unwrap();
            treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
            treeview.grab_focus();
            
        }
    }
}
