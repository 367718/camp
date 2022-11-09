use std::{
    collections::HashSet,
    error::Error,
};

use gtk::{
    gdk,
    gio,
    glib::Sender,
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, GeneralActions,
    CandidatesId, CandidatesEntry, CandidatesCurrent,
    SeriesId, SeriesEntry, SeriesStatus, SeriesGood,
    KindsId,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- widgets ----------
    
    let candidates_sort = &state.ui.widgets().stores.preferences.candidates.sort;
    let downloaded_sort = &state.ui.widgets().stores.preferences.candidates.downloaded_sort;
    
    // ---------- set sort ----------
    
    candidates_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set models ----------
    
    state.ui.widgets().window.preferences.candidates.candidates_treeview.set_model(Some(candidates_sort));
    state.ui.widgets().window.preferences.candidates.downloaded_treeview.set_model(Some(downloaded_sort));
}

fn fill(state: &State) {
    // 0 => id
    // 1 => title
    
    let candidates_store = &state.ui.widgets().stores.preferences.candidates.store;
    candidates_store.clear();
    
    for (id, entry) in state.database.candidates_iter() {
        candidates_store.insert_with_values(
            None,
            &[
                (0, &id.as_int()),
                (1, &entry.title()),
            ],
        );
    }
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    // candidates
    
    let candidates_add_action = gio::SimpleAction::new("preferences.candidates.candidates.add", None);
    let candidates_edit_action = gio::SimpleAction::new("preferences.candidates.candidates.edit", None);
    let candidates_delete_action = gio::SimpleAction::new("preferences.candidates.candidates.delete", None);
    
    // add candidate
    candidates_add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesAdd(None))).unwrap()
    });
    
    // edit candidate
    candidates_edit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesEdit)).unwrap()
    });
    
    // delete candidate
    candidates_delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesDelete)).unwrap()
    });
    
    app.add_action(&candidates_add_action);
    app.add_action(&candidates_edit_action);
    app.add_action(&candidates_delete_action);
    
    // downloaded
    
    let downloaded_add_action = gio::SimpleAction::new("preferences.candidates.downloaded.add", None);
    let downloaded_delete_action = gio::SimpleAction::new("preferences.candidates.downloaded.delete", None);
    
    // add downloaded
    downloaded_add_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::DownloadedAdd)).unwrap()
    });
    
    // delete downloaded
    downloaded_delete_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::DownloadedDelete)).unwrap()
    });
    
    app.add_action(&downloaded_add_action);
    app.add_action(&downloaded_delete_action);
    
    // ---------- treeviews ----------
    
    // candidates
    
    let treeview = &state.ui.widgets().window.preferences.candidates.candidates_treeview;
    
    // add candidate (Insert)
    // edit candidate (F2)
    // delete candidate (Delete)
    treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Insert => sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesAdd(None))).unwrap(),
                gdk::keys::constants::F2 => sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesEdit)).unwrap(),
                gdk::keys::constants::Delete => sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesDelete)).unwrap(),
                _ => (),
            }
            Inhibit(false)
        }
    });
    
    // edit candidate (Double-click, Return, Space)
    treeview.connect_row_activated({
        let sender_cloned = sender.clone();
        move |_, _, _| sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesEdit)).unwrap()
    });
    
    // show candidate information (row selected)
    treeview.connect_cursor_changed({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::Preferences(PreferencesActions::CandidatesShowInfo)).unwrap()
    });
    
    // downloaded
    
    let downloaded_treeview = &state.ui.widgets().window.preferences.candidates.downloaded_treeview;
    
    // add downloaded (Insert)
    // delete downloaded (Delete)
    downloaded_treeview.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Insert => sender_cloned.send(Message::Preferences(PreferencesActions::DownloadedAdd)).unwrap(),
                gdk::keys::constants::Delete => sender_cloned.send(Message::Preferences(PreferencesActions::DownloadedDelete)).unwrap(),
                _ => (),
            }
            Inhibit(false)
        }
    });
}

pub fn show_info(state: &State) {
    let candidates_treeview = &state.ui.widgets().window.preferences.candidates.candidates_treeview;
    let downloaded_store = &state.ui.widgets().stores.preferences.candidates.downloaded_store;
    
    downloaded_store.clear();
    
    if let Some((candidates_treemodel, candidates_treeiter)) = candidates_treeview.selection().selected() {
        
        let id = CandidatesId::from(candidates_treemodel.value(&candidates_treeiter, 0).get::<u32>().unwrap());
        
        if let Some(candidate) = state.database.candidates_get(id) {
            
            if candidate.current() == CandidatesCurrent::Yes {
                let downloaded_sort = &state.ui.widgets().stores.preferences.candidates.downloaded_sort;
                
                downloaded_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Descending);
                
                for download in candidate.downloaded() {
                    downloaded_store.insert_with_values(
                        None,
                        &[
                            (0, download),
                        ],
                    );
                }
                
                downloaded_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Descending);
            }
            
        }
    }
}

pub fn reload(state: &State) {
    // ---------- widget ----------
    
    let candidates_sort = &state.ui.widgets().stores.preferences.candidates.sort;
    
    // ---------- unset model ----------
    
    state.ui.widgets().window.preferences.candidates.candidates_treeview.set_model(None::<&gtk::TreeModel>);
    
    // ---------- unset sort ----------
    
    candidates_sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
    
    // ---------- fill ----------
    
    fill(state);
    
    // ---------- set sort ----------
    
    candidates_sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
    
    // ---------- set model ----------
    
    state.ui.widgets().window.preferences.candidates.candidates_treeview.set_model(Some(candidates_sort));
}


// ---------- candidates ----------


pub fn candidates_add(state: &mut State, sender: &Sender<Message>, prefill: &Option<String>) {
    state.ui.widgets().dialogs.preferences.candidates.dialog.set_title("Add candidate");
    
    let title_entry = &state.ui.widgets().dialogs.preferences.candidates.title_entry;
    let group_entry = &state.ui.widgets().dialogs.preferences.candidates.group_entry;
    let quality_entry = &state.ui.widgets().dialogs.preferences.candidates.quality_entry;
    let series_entry = &state.ui.widgets().dialogs.preferences.candidates.series_entry;
    let offset_spin = &state.ui.widgets().dialogs.preferences.candidates.offset_spin;
    let current_switch = &state.ui.widgets().dialogs.preferences.candidates.current_switch;
    let downloaded_spin = &state.ui.widgets().dialogs.preferences.candidates.downloaded_spin;
    
    title_entry.set_text(prefill.as_deref().unwrap_or(""));
    group_entry.set_text("");
    quality_entry.set_text("");
    series_entry.set_text("");
    offset_spin.set_value(0.0);
    current_switch.set_active(true);
    downloaded_spin.set_sensitive(true);
    downloaded_spin.set_value(1.0);
    
    let mut series = SeriesId::from(0);
    
    loop {
        
        state.ui.widgets().dialogs.preferences.candidates.title_entry.grab_focus();
        
        let response = state.ui.widgets().dialogs.preferences.candidates.dialog.run();
        
        state.ui.widgets().dialogs.preferences.candidates.dialog.unrealize();
        state.ui.widgets().dialogs.preferences.candidates.dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Other(0) => {
                
                // change selected series status, if necessary
                if let Err(error) = candidates_series_edit(state, sender, series) {
                    state.ui.dialogs_error_show(&error.to_string());
                    continue;
                }
                
                let current = CandidatesCurrent::from(state.ui.widgets().dialogs.preferences.candidates.current_switch.is_active());
                
                let cap = state.ui.widgets().dialogs.preferences.candidates.downloaded_spin.text().parse().unwrap_or(0);
                
                let downloaded = if current == CandidatesCurrent::Yes && cap > 0 {
                    (1..=cap).collect()
                } else {
                    HashSet::new()
                };
                
                let entry = CandidatesEntry::new()
                    .with_series(series)
                    .with_title(state.ui.widgets().dialogs.preferences.candidates.title_entry.text().to_string())
                    .with_group(state.ui.widgets().dialogs.preferences.candidates.group_entry.text().to_string())
                    .with_quality(state.ui.widgets().dialogs.preferences.candidates.quality_entry.text().to_string())
                    .with_offset(state.ui.widgets().dialogs.preferences.candidates.offset_spin.text().parse().unwrap_or(0))
                    .with_current(current)
                    .with_downloaded(downloaded);
                
                match state.database.candidates_add(entry) {
                    
                    Ok(id) => {
                        
                        let candidate = state.database.candidates_get(id).unwrap();
                        
                        let candidates_treeview = &state.ui.widgets().window.preferences.candidates.candidates_treeview;
                        
                        let candidates_sort = &state.ui.widgets().stores.preferences.candidates.sort;
                        let candidates_store = &state.ui.widgets().stores.preferences.candidates.store;
                        
                        let store_iter = candidates_store.insert_with_values(
                            None,
                            &[
                                (0, &id.as_int()),
                                (1, &candidate.title()),
                            ],
                        );
                        
                        if prefill.is_none() {
                            
                            let sort_iter = candidates_sort.convert_child_iter_to_iter(&store_iter).unwrap();
                            let treepath = candidates_sort.path(&sort_iter).unwrap();
                            candidates_treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
                            candidates_treeview.grab_focus();
                            
                        }
                        
                        break;
                        
                    },
                    
                    Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                    
                }
                
            },
            
            // select series
            
            gtk::ResponseType::Other(1) => {
                
                if let Some(id) = candidates_series_select(state, sender, &state.ui.widgets().dialogs.preferences.candidates.title_entry.text()) {                    
                    if let Some(retrieved) = state.database.series_get(id) {
                        state.ui.widgets().dialogs.preferences.candidates.series_entry.set_text(retrieved.title());
                        series = id;
                        continue;
                    }
                }
                
                state.ui.widgets().dialogs.preferences.candidates.series_entry.set_text("");
                series = SeriesId::from(0);
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

pub fn candidates_edit(state: &mut State, sender: &Sender<Message>) {
    let Some((treemodel, treeiter)) = state.ui.widgets().window.preferences.candidates.candidates_treeview.selection().selected() else {
        return;
    };
    
    let id = CandidatesId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
    
    match state.database.candidates_get(id) {
        
        Some(previous) => {
            
            let Some(series_title) = state.database.series_get(previous.series()).map(SeriesEntry::title) else {
                state.ui.dialogs_error_show("Series not found");
                return;
            };
            
            state.ui.widgets().dialogs.preferences.candidates.dialog.set_title("Edit candidate");
            
            let title_entry = &state.ui.widgets().dialogs.preferences.candidates.title_entry;
            let group_entry = &state.ui.widgets().dialogs.preferences.candidates.group_entry;
            let quality_entry = &state.ui.widgets().dialogs.preferences.candidates.quality_entry;
            let series_entry = &state.ui.widgets().dialogs.preferences.candidates.series_entry;
            let offset_spin = &state.ui.widgets().dialogs.preferences.candidates.offset_spin;
            let current_switch = &state.ui.widgets().dialogs.preferences.candidates.current_switch;
            let downloaded_spin = &state.ui.widgets().dialogs.preferences.candidates.downloaded_spin;
            
            title_entry.set_text(previous.title());
            group_entry.set_text(previous.group());
            quality_entry.set_text(previous.quality());
            series_entry.set_text(series_title);
            offset_spin.set_value(f64::from(previous.offset()));
            current_switch.set_active(previous.current() == CandidatesCurrent::Yes);
            downloaded_spin.set_sensitive(false);
            downloaded_spin.set_value(0.0);
            
            let mut series = previous.series();
            
            let previous_downloaded = previous.downloaded().clone();
            
            loop {
                
                state.ui.widgets().dialogs.preferences.candidates.title_entry.grab_focus();
                
                let response = state.ui.widgets().dialogs.preferences.candidates.dialog.run();
                
                state.ui.widgets().dialogs.preferences.candidates.dialog.unrealize();
                state.ui.widgets().dialogs.preferences.candidates.dialog.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Other(0) => {
                        
                        // change selected series status, if necessary
                        if let Err(error) = candidates_series_edit(state, sender, series) {
                            state.ui.dialogs_error_show(&error.to_string());
                            continue;                    
                        }
                        
                        let current = CandidatesCurrent::from(state.ui.widgets().dialogs.preferences.candidates.current_switch.is_active());
                        
                        let downloaded = if current == CandidatesCurrent::Yes {
                            previous_downloaded.clone()
                        } else {
                            HashSet::new()
                        };
                        
                        let entry = CandidatesEntry::new()
                            .with_series(series)
                            .with_title(state.ui.widgets().dialogs.preferences.candidates.title_entry.text().to_string())
                            .with_group(state.ui.widgets().dialogs.preferences.candidates.group_entry.text().to_string())
                            .with_quality(state.ui.widgets().dialogs.preferences.candidates.quality_entry.text().to_string())
                            .with_offset(state.ui.widgets().dialogs.preferences.candidates.offset_spin.text().parse().unwrap_or(0))
                            .with_current(current)
                            .with_downloaded(downloaded);
                        
                        match state.database.candidates_edit(id, entry) {
                            
                            Ok(_) => {
                                
                                let candidate = state.database.candidates_get(id).unwrap();
                                
                                let candidates_sort = &state.ui.widgets().stores.preferences.candidates.sort;
                                let candidates_store = &state.ui.widgets().stores.preferences.candidates.store;
                                
                                let store_iter = candidates_sort.convert_iter_to_child_iter(&treeiter);
                                
                                candidates_store.set(
                                    &store_iter,
                                    &[
                                        (1, &candidate.title()),
                                    ],
                                );
                                
                                show_info(state);
                                
                                state.ui.widgets().window.preferences.candidates.candidates_treeview.grab_focus();
                                
                                break;
                                
                            },
                            
                            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                            
                        }
                        
                    }
                    
                    // select series
                    
                    gtk::ResponseType::Other(1) => {
                        
                        if let Some(id) = candidates_series_select(state, sender, &state.ui.widgets().dialogs.preferences.candidates.title_entry.text()) {                    
                            if let Some(retrieved) = state.database.series_get(id) {
                                state.ui.widgets().dialogs.preferences.candidates.series_entry.set_text(retrieved.title());
                                series = id;
                                continue;
                            }
                        }
                        
                        state.ui.widgets().dialogs.preferences.candidates.series_entry.set_text("");
                        series = SeriesId::from(0);
                        
                    },
                    
                    // cancel
                    
                    _ => break,
                    
                }
                
            }
            
        },
        
        None => state.ui.dialogs_error_show("Candidate not found"),
        
    }
}

pub fn candidates_delete(state: &mut State) {
    let Some((treemodel, treeiter)) = state.ui.widgets().window.preferences.candidates.candidates_treeview.selection().selected() else {
        return;
    };
    
    let delete_dialog = &state.ui.widgets().dialogs.general.delete.dialog;
    
    let response = delete_dialog.run();
    
    delete_dialog.unrealize();
    delete_dialog.hide();
    
    // confirm
    
    if response == gtk::ResponseType::Ok {
        
        let id = CandidatesId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap());
        
        match state.database.candidates_remove(id) {
            
            Ok(_) => {
                
                let candidates_sort = &state.ui.widgets().stores.preferences.candidates.sort;
                let candidates_store = &state.ui.widgets().stores.preferences.candidates.store;
                
                let store_iter = candidates_sort.convert_iter_to_child_iter(&treeiter);
                
                candidates_store.remove(&store_iter);
                
            },
            
            Err(error) => state.ui.dialogs_error_show(&error.to_string()),
            
        }
        
    }
}

fn candidates_series_select(state: &mut State, sender: &Sender<Message>, prefill: &str) -> Option<SeriesId> {
    let candidates_series = &state.ui.widgets().dialogs.preferences.candidates_series;
    
    candidates_series.notebook.set_current_page(Some(0));
    
    candidates_series.watching_treeview.realize();
    candidates_series.on_hold_treeview.realize();
    candidates_series.plan_to_watch_treeview.realize();
    
    candidates_series.watching_treeview.set_cursor(&gtk::TreePath::new_first(), None::<&gtk::TreeViewColumn>, false);
    candidates_series.on_hold_treeview.set_cursor(&gtk::TreePath::new_first(), None::<&gtk::TreeViewColumn>, false);
    candidates_series.plan_to_watch_treeview.set_cursor(&gtk::TreePath::new_first(), None::<&gtk::TreeViewColumn>, false);
    
    candidates_series.watching_treeview.selection().unselect_all();
    candidates_series.on_hold_treeview.selection().unselect_all();
    candidates_series.plan_to_watch_treeview.selection().unselect_all();
    
    candidates_series.watching_treeview.grab_focus();
    
    loop {
        
        let response = state.ui.widgets().dialogs.preferences.candidates_series.dialog.run();
        
        state.ui.widgets().dialogs.preferences.candidates_series.dialog.unrealize();
        state.ui.widgets().dialogs.preferences.candidates_series.dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Other(0) => {
                
                let current_tree = match state.ui.widgets().dialogs.preferences.candidates_series.notebook.current_page() {
                    Some(0) => &state.ui.widgets().dialogs.preferences.candidates_series.watching_treeview,
                    Some(1) => &state.ui.widgets().dialogs.preferences.candidates_series.on_hold_treeview,
                    Some(2) => &state.ui.widgets().dialogs.preferences.candidates_series.plan_to_watch_treeview,
                    _ => unreachable!(),
                };
                
                if let Some((treemodel, treeiter)) = current_tree.selection().selected() {
                    break Some(SeriesId::from(treemodel.value(&treeiter, 0).get::<u32>().unwrap()));
                }
                
            },
            
            // add new
            
            gtk::ResponseType::Other(1) => {
                
                let id = candidates_series_add(state, sender, prefill);
                
                if id.is_some() {
                    break id;
                }
                
            },
            
            // cancel
            
            _ => break None,
            
        }
        
    }
}

fn candidates_series_add(state: &mut State, sender: &Sender<Message>, prefill: &str) -> Option<SeriesId> {
    let series_dialog = &state.ui.widgets().dialogs.watchlist.series.dialog;
    
    series_dialog.set_title("Add series");
    
    // confirm and add another
    series_dialog.set_response_sensitive(gtk::ResponseType::Other(1), false);
    
    let title_entry = &state.ui.widgets().dialogs.watchlist.series.title_entry;
    let kind_combo = &state.ui.widgets().dialogs.watchlist.series.kind_combo;
    let status_combo = &state.ui.widgets().dialogs.watchlist.series.status_combo;
    let progress_spin = &state.ui.widgets().dialogs.watchlist.series.progress_spin;
    let good_switch = &state.ui.widgets().dialogs.watchlist.series.good_switch;
    
    title_entry.set_text(prefill);
    kind_combo.set_active(Some(0));
    status_combo.set_sensitive(false);
    status_combo.set_active(Some(0));
    progress_spin.set_value(1.0);
    good_switch.set_sensitive(false);
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
            
            // confirm
            
            gtk::ResponseType::Other(0) => {
                
                let status = match SeriesStatus::try_from(status_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))) {
                    Ok(value) => value,
                    Err(error) => {
                        state.ui.dialogs_error_show(&error.to_string());
                        continue;
                    }
                };
                
                let entry = SeriesEntry::new()
                    .with_title(title_entry.text().to_string())
                    .with_kind(KindsId::from(kind_combo.active_id().map_or(0, |id| id.parse().unwrap_or(0))))
                    .with_status(status)
                    .with_progress(progress_spin.text().parse().unwrap_or(0))
                    .with_good(SeriesGood::from(good_switch.is_active()));
                
                match state.database.series_add(entry) {
                    
                    Ok(id) => {
                        
                        let series = state.database.series_get(id).unwrap();
                        
                        let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
                        
                        watchlist_store.insert_with_values(
                            None,
                            &[
                                (0, &id.as_int()),
                                
                                (1, &(u32::from(series.good().as_int()) * 400)),
                                (2, &series.status().as_int()),
                                
                                (3, &series.title()),
                                
                                (4, &series.good().display()),
                                (5, &state.database.kinds_get(series.kind()).map_or("", |kind| kind.name())),
                                (6, &series.progress()),
                            ],
                        );
                        
                        sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
                        
                        break Some(id);
                        
                    },
                    
                    Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                    
                }
                
            },
            
            // lookup
            
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
            
            _ => break None,
            
        }
        
    }
}

fn candidates_series_edit(state: &mut State, sender: &Sender<Message>, series: SeriesId) -> Result<(), Box<dyn Error>> {
    let previous = state.database.series_get(series).ok_or("Series not found")?;
    
    if previous.status() != SeriesStatus::Watching {
        
        let mut new = previous.clone()
            .with_status(SeriesStatus::Watching);
        
        if new.progress() == 0 {
            new = new.with_progress(1);
        }
        
        state.database.series_edit(series, new)?;
        
        let new = state.database.series_get(series).unwrap();
        
        let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
        
        watchlist_store.foreach(|_, _, store_iter| {
            let current = SeriesId::from(watchlist_store.value(store_iter, 0).get::<u32>().unwrap());
            
            if current == series {
                watchlist_store.set(
                    store_iter,
                    &[
                        (2, &new.status().as_int()),
                        (6, &new.progress()),
                    ],
                );
                
                return true;
            }
            
            false
        });
        
        sender.send(Message::General(GeneralActions::SearchShouldRecompute)).unwrap();
        
    }
    
    Ok(())
}


// ---------- downloaded ----------


pub fn downloaded_add(state: &mut State) {
    let Some((candidates_treemodel, candidates_treeiter)) = state.ui.widgets().window.preferences.candidates.candidates_treeview.selection().selected() else {
        return;
    };
    
    let id = CandidatesId::from(candidates_treemodel.value(&candidates_treeiter, 0).get::<u32>().unwrap());
    
    match state.database.candidates_get(id) {
        
        Some(candidate) => {
            
            let candidate = candidate.clone();
            
            let downloaded_dialog = &state.ui.widgets().dialogs.preferences.candidates_downloaded.dialog;
            
            downloaded_dialog.set_title("Add download");
            
            let title_label = &state.ui.widgets().dialogs.preferences.candidates_downloaded.title_label;
            let download_spin = &state.ui.widgets().dialogs.preferences.candidates_downloaded.download_spin;
            
            title_label.set_text(candidate.title());
            download_spin.set_value(f64::from(candidate.downloaded().iter().max().unwrap_or(&0).saturating_add(1)));
            
            download_spin.grab_focus();
            
            loop {
                
                let response = downloaded_dialog.run();
                
                downloaded_dialog.unrealize();
                downloaded_dialog.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Ok => {
                        
                        let download = download_spin.text().parse().unwrap_or(0);
                        
                        let mut downloaded = candidate.downloaded().clone();
                        downloaded.insert(download);
                        
                        let new = candidate.clone()
                            .with_downloaded(downloaded);
                        
                        match state.database.candidates_edit(id, new) {
                            
                            Ok(previous) => {
                                
                                let downloaded_treeview = &state.ui.widgets().window.preferences.candidates.downloaded_treeview;
                                let downloaded_sort = &state.ui.widgets().stores.preferences.candidates.downloaded_sort;
                                
                                // only select
                                if previous.downloaded().contains(&download) {
                                    
                                    downloaded_sort.foreach(|_, sort_path, sort_iter| {
                                        let current = downloaded_sort.value(sort_iter, 0).get::<u32>().unwrap();
                                        
                                        if current == download {
                                            downloaded_treeview.set_cursor(sort_path, None::<&gtk::TreeViewColumn>, false);
                                            return true;
                                        }
                                        
                                        false
                                    });
                                    
                                
                                // insert and select
                                } else {
                                    
                                    let downloaded_store = &state.ui.widgets().stores.preferences.candidates.downloaded_store;
                                    
                                    let store_iter = downloaded_store.insert_with_values(
                                        None,
                                        &[
                                            (0, &download),
                                        ],
                                    );
                                    
                                    let sort_iter = downloaded_sort.convert_child_iter_to_iter(&store_iter).unwrap();
                                    let treepath = downloaded_sort.path(&sort_iter).unwrap();
                                    downloaded_treeview.set_cursor(&treepath, None::<&gtk::TreeViewColumn>, false);
                                    
                                }
                                
                                downloaded_treeview.grab_focus();
                                
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
        
        None => state.ui.dialogs_error_show("Candidate not found"),
        
    }
}

pub fn downloaded_delete(state: &mut State) {
    let candidates_treeview = &state.ui.widgets().window.preferences.candidates.candidates_treeview;
    let downloaded_treeview = &state.ui.widgets().window.preferences.candidates.downloaded_treeview;
    
    let Some((candidates_treemodel, candidates_treeiter)) = candidates_treeview.selection().selected() else {
        return;
    };
    
    let Some((downloaded_treemodel, downloaded_treeiter)) = downloaded_treeview.selection().selected() else {
        return;
    };
    
    let id = CandidatesId::from(candidates_treemodel.value(&candidates_treeiter, 0).get::<u32>().unwrap());
    
    match state.database.candidates_get(id) {
        
        Some(candidate) => {
            
            let download = downloaded_treemodel.value(&downloaded_treeiter, 0).get::<u32>().unwrap();
            
            let mut downloaded = candidate.downloaded().clone();
            downloaded.remove(&download);
            
            let new = candidate.clone()
                .with_downloaded(downloaded);
            
            match state.database.candidates_edit(id, new) {
                
                Ok(_) => {
                    
                    let downloaded_sort = &state.ui.widgets().stores.preferences.candidates.downloaded_sort;
                    let downloaded_store = &state.ui.widgets().stores.preferences.candidates.downloaded_store;
                    
                    let downloaded_store_iter = downloaded_sort.convert_iter_to_child_iter(&downloaded_treeiter);
                    downloaded_store.remove(&downloaded_store_iter);
                    
                },
                
                Err(error) => state.ui.dialogs_error_show(&error.to_string()),
                
            }
            
        },
        
        None => state.ui.dialogs_error_show("Candidate not found"),
        
    }
}

pub fn downloaded_update(state: &mut State, sender: &Sender<Message>, downloads: Vec<(SeriesId, u32)>) {
    if downloads.is_empty() {
        return;
    }
    
    for (series, download) in downloads {
        
        let Some((id, candidate)) = state.database.candidates_iter().find(|(_, current)| current.series() == series) else {
            continue;
        };
        
        if ! candidate.downloaded().contains(&download) {
            
            let mut downloaded = candidate.downloaded().clone();
            downloaded.insert(download);
            
            let new = candidate.clone()
                .with_downloaded(downloaded);
            
            state.database.candidates_edit(*id, new).ok();
            
        }
        
    }
    
    // make sure shown downloads are up to date
    sender.send(Message::Preferences(PreferencesActions::CandidatesShowInfo)).unwrap();
}
