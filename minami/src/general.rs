use std::{
    cmp::Ordering,
    env,
    error::Error,
    ffi::{ c_void, OsString },
    fs::{ self, OpenOptions },
    io,
    iter::Peekable,
    mem,
    path::{ MAIN_SEPARATOR, PathBuf },
    process,
    ptr,
    str::{ self, Chars },
};

use gtk::{
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    APP_NAME,
    State, Message, FilesSection, WatchlistSection,
    PreferencesActions,
    Database,
};

pub enum GeneralActions {
    
    // ---------- config and database ----------
    
    ReloadDatabase,
    BackupDatabase,
    SaveAndQuit,
    
    // ---------- section ----------
    
    SectionFocus,
    SectionNext,
    SectionPrevious,
    SectionSwitchFiles,
    SectionSwitchWatchlist,
    SectionSwitchPreferences,
    
    // ---------- search ----------
    
    SearchCompute,
    SearchShouldRecompute,
    SearchShow,
    SearchFocus,
    SearchSelect(String),
    
}

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(app, state);
    bind(app, state, sender);
}

fn build(app: &gtk::Application, state: &State) {
    // ---------- dialogs ----------
    
    let database_dialog = &state.ui.widgets().dialogs.general.database_load_error.dialog;
    
    database_dialog.set_title("Database load error");
    database_dialog.set_position(gtk::WindowPosition::CenterOnParent);
    database_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
    if let Some(widget) = database_dialog.widget_for_response(gtk::ResponseType::Cancel) {
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            button.set_label("Cancel");
        }
    }
    
    let error_dialog = &state.ui.widgets().dialogs.general.error.dialog;
    
    error_dialog.set_title("Error");
    error_dialog.set_position(gtk::WindowPosition::CenterOnParent);
    error_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
    let chooser_dialog = &state.ui.widgets().dialogs.general.chooser.dialog;
    
    chooser_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
    // ---------- search ----------
    
    state.ui.widgets().window.general.search_completion.set_model(Some(&state.ui.widgets().stores.general.search.sort));
    
    // ---------- listboxes ----------
    
    if let Some(row) = state.ui.widgets().window.files.listbox.row_at_index(0) {
        row.activate();
    }
    
    // ---------- entries ----------
    
    state.ui.widgets().window.general.search_entry.grab_focus();
    
    // ---------- windows ----------
    
    let app_window = &state.ui.widgets().window.general.window;
    
    let width = state.params.window_width(true);
    let height = state.params.window_height(true);
    app_window.set_default_size(width, height);
    
    let x = state.params.window_x(true);
    let y = state.params.window_y(true);
    app_window.move_(x, y);
    
    if state.params.window_maximized(true) {
        app_window.maximize();
    }
    
    app_window.set_application(Some(app));
    
    app_window.show_all();
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- global hotkeys ----------
    
    app.set_accels_for_action("app.general.save_and_quit", &["<Primary>Q"]);
    app.set_accels_for_action("app.general.search.focus", &["<Primary>F"]);
    app.set_accels_for_action("app.general.section.focus", &["<Primary>E"]);
    app.set_accels_for_action("app.general.section.next", &["<Primary>Page_Down"]);
    app.set_accels_for_action("app.general.section.previous", &["<Primary>Page_Up"]);
    
    // ---------- actions ----------
    
    let quit_action = gio::SimpleAction::new("general.save_and_quit", None);
    let backup_action = gio::SimpleAction::new("general.backup_database", None);
    
    let search_focus_action = gio::SimpleAction::new("general.search.focus", None);
    let section_focus_action = gio::SimpleAction::new("general.section.focus", None);
    let section_next_action = gio::SimpleAction::new("general.section.next", None);
    let section_previous_action = gio::SimpleAction::new("general.section.previous", None);
    
    // save config and database and terminate app
    quit_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SaveAndQuit)).unwrap()
    });
    
    // backup database
    backup_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::BackupDatabase)).unwrap()
    });
    
    // focus search entry
    search_focus_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap()
    });
    
    // focus visible section list (files and watchlist only)
    section_focus_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionFocus)).unwrap()
    });
    
    // change visible section (down)
    section_next_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionNext)).unwrap()
    });
    
    // change visible section (up)
    section_previous_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionPrevious)).unwrap()
    });
    
    app.add_action(&quit_action);
    app.add_action(&backup_action);
    
    app.add_action(&search_focus_action);
    app.add_action(&section_focus_action);
    app.add_action(&section_next_action);
    app.add_action(&section_previous_action);
    
    // ---------- entries ----------
    
    let search_entry = &state.ui.widgets().window.general.search_entry;
    let search_completion = &state.ui.widgets().window.general.search_completion;
    
    // compute and show search result list
    search_entry.connect_search_changed({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::General(GeneralActions::SearchCompute)).unwrap()
    });
    
    // show search result list on focus
    search_entry.connect_grab_focus({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::General(GeneralActions::SearchShow)).unwrap()
    });
    
    // search file / series on search result selection
    search_completion.connect_match_selected({
        let sender_cloned = sender.clone();
        move |_, model, iter| {
            if let Some(string_iter) = model.string_from_iter(iter).map(|iter| iter.to_string()) {
                sender_cloned.send(Message::General(GeneralActions::SearchSelect(string_iter))).unwrap();
            }
            Inhibit(true)
        }
    });
    
    // ---------- listboxes ----------
    
    state.ui.widgets().window.files.listbox.connect_row_selected({
        let sender_cloned = sender.clone();
        move |_, selected| if selected.is_some() {
            sender_cloned.send(Message::General(GeneralActions::SectionSwitchFiles)).unwrap();
        }
    });
    
    state.ui.widgets().window.watchlist.listbox.connect_row_selected({
        let sender_cloned = sender.clone();
        move |_, selected| if selected.is_some() {
            sender_cloned.send(Message::General(GeneralActions::SectionSwitchWatchlist)).unwrap();
        }
    });
    
    state.ui.widgets().window.preferences.listbox.connect_row_selected({
        let sender_cloned = sender.clone();
        move |_, selected| if selected.is_some() {
            sender_cloned.send(Message::General(GeneralActions::SectionSwitchPreferences)).unwrap();
        }
    });
    
    // ---------- windows ----------
    
    // save config and database and terminate app
    state.ui.widgets().window.general.window.connect_delete_event({
        let sender_cloned = sender.clone();
        move |_, _| {
            sender_cloned.send(Message::General(GeneralActions::SaveAndQuit)).unwrap();
            Inhibit(true)
        }
    });
}

pub fn handle_action(state: &mut State, sender: &Sender<Message>, action: GeneralActions) {
    use GeneralActions::*;
    
    match action {
        
        // ---------- config and database ----------
        
        // preferences -> paths -> commit_database
        ReloadDatabase => reload_database(state, sender),
        
        // general -> bind
        BackupDatabase => backup_database(state),
        
        // general -> bind x2
        SaveAndQuit => save_and_quit(state),
        
        // ---------- section ----------
        
        // general -> bind
        SectionFocus => section_focus(state),
        
        // general -> bind
        SectionNext => section_next(state),
        
        // general -> bind
        SectionPrevious => section_previous(state),
        
        // general -> bind
        SectionSwitchFiles => section_switch_files(state),
        
        // general -> bind
        SectionSwitchWatchlist => section_switch_watchlist(state),
        
        // general -> bind
        SectionSwitchPreferences => section_switch_preferences(state),
        
        // ---------- search ----------
        
        // general -> bind
        SearchCompute => search_compute(state),
        
        // files -> general -> add
        // files -> general -> remove
        // files -> general -> reload
        // watchlist -> edit -> add
        // watchlist -> edit -> edit
        // watchlist -> edit -> delete
        // watchlist -> general -> reload
        // preferences -> candidates -> candidates_series_add
        // preferences -> candidates -> candidates_series_edit
        SearchShouldRecompute => search_should_recompute(state),
        
        // general -> bind
        SearchShow => search_show(state),
        
        // general -> bind
        SearchFocus => search_focus(state),
        
        // general -> bind
        SearchSelect(string_iter) => search_select(state, &string_iter),
        
    }
}


// ---------- config and database ----------


fn reload_database(state: &mut State, sender: &Sender<Message>) {
    // CAUTION: this will discard uncommited changes in current database
    
    let mut database = None;
    
    let mut dbpath = state.params.paths_database(false).to_owned();
    let mut message = String::new();
    
    // success
    
    match Database::load(&dbpath) {
        Ok(data) => database = Some(data),
        Err(error) => message = error.to_string(),
    }
    
    // error
    
    if database.is_none() {
        
        let save_chooser = &state.ui.widgets().dialogs.general.chooser.dialog;
        
        save_chooser.set_title("Choose database path");
        save_chooser.set_action(gtk::FileChooserAction::Save);
        
        let database_dialog = &state.ui.widgets().dialogs.general.database_load_error.dialog;
        
        database = 'outer: loop {
            
            state.ui.widgets().dialogs.general.database_load_error.path_label.set_text(&dbpath.to_string_lossy());
            state.ui.widgets().dialogs.general.database_load_error.message_label.set_text(&message);
            
            let response = database_dialog.run();
            
            database_dialog.unrealize();
            database_dialog.hide();
            
            match response {
                
                // generate new
                
                gtk::ResponseType::Other(0) => match Database::new(&dbpath) {
                    Ok(generated) => break 'outer Some(generated),
                    Err(error) => message = error.to_string(),
                },
                
                // select another
                
                gtk::ResponseType::Other(1) => 'inner: loop {
                    
                    let response = save_chooser.run();
                    
                    save_chooser.hide();
                    
                    match response {
                        
                        // confirm
                        
                        gtk::ResponseType::Accept => {
                            
                            if let Some(chosen) = save_chooser.filename() {
                                
                                dbpath = chosen;
                                
                                let result = Database::load(&dbpath).and_then(|database| {
                                    state.params.paths_set_database(&dbpath)?;
                                    Ok(database)
                                });
                                
                                match result {
                                    Ok(database) => break 'outer Some(database),
                                    Err(error) => message = error.to_string(),
                                }
                                
                                break 'inner;
                                
                            }
                            
                        },
                        
                        // cancel
                        
                        _ => break 'inner,
                        
                    }
                    
                },
                
                // cancel
                
                _ => {
                    
                    let database_entry = &state.ui.widgets().window.preferences.paths.database_entry;
                    database_entry.set_text(&state.params.paths_database(false).to_string_lossy());
                    database_entry.grab_focus();
                    
                    break 'outer None;
                    
                },
                
            }
            
        };
        
    }
    
    // database changed
    
    if let Some(database) = database {
        state.database = database;
        
        state.ui.widgets().window.preferences.paths.database_entry.set_text(&dbpath.to_string_lossy());
        
        sender.send(Message::Preferences(PreferencesActions::CandidatesReload)).unwrap();
        sender.send(Message::Preferences(PreferencesActions::FeedsReload)).unwrap();
        
        // this will also reload files
        sender.send(Message::Preferences(PreferencesActions::FormatsReload)).unwrap();
        
        // this will also reload watchlist
        sender.send(Message::Preferences(PreferencesActions::KindsReload)).unwrap();
    }
}

fn backup_database(state: &mut State) {
    // make sure uncommitted changes are saved
    if let Err(error) = state.database.save(state.params.paths_database(true)) {
        
        let mut message = error.to_string();
        let config_dialog = &state.ui.widgets().dialogs.general.database_save_error.dialog;
        
        loop {
            
            state.ui.widgets().dialogs.general.database_save_error.path_label.set_text(&state.params.paths_database(true).to_string_lossy());
            state.ui.widgets().dialogs.general.database_save_error.message_label.set_text(&message);
            
            let response = config_dialog.run();
            
            config_dialog.unrealize();
            config_dialog.hide();
            
            match response {
                
                // try again
                
                gtk::ResponseType::Ok => {
                    
                    if let Err(error) = state.database.save(state.params.paths_database(true)) {
                        message = error.to_string();
                        continue;
                    }
                    
                },
                
                // give up
                
                _ => return,
                
            }
            
        }
        
    }
    
    let save_chooser = &state.ui.widgets().dialogs.general.chooser.dialog;
    
    save_chooser.set_title("Backup database");
    save_chooser.set_action(gtk::FileChooserAction::Save);
    save_chooser.set_current_name(&concat_str!(APP_NAME, "-", &current_date(), ".db"));
    
    loop {
        
        let response = save_chooser.run();
        
        save_chooser.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Accept => {
                
                if let Some(mut path) = save_chooser.filename() {
                    
                    if let Some(current) = path.extension() {
                        if current != "db" {
                            let mut composite = OsString::with_capacity(current.len() + 3);
                            composite.push(current);
                            composite.push(".db");
                            path.set_extension(composite);
                        }
                    } else {
                        path.set_extension("db");
                    }
                    
                    let creation = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)
                        .map_err(|_| concat_str!("File already exists or write error: ", &path.to_string_lossy()));
                    
                    if let Err(error) = creation {
                        state.ui.dialogs_error_show(&error);
                    } else if let Err(error) = fs::copy(state.params.paths_database(true), path) {
                        state.ui.dialogs_error_show(&error.to_string());
                    }
                    
                    break;
                    
                }
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

fn save_and_quit(state: &mut State) {
    // commit dimensions, coordinates and maximized window state
    
    let app_window = &state.ui.widgets().window.general.window;
    
    if app_window.is_visible() {
        
        if ! app_window.is_maximized() {
            
            let size = app_window.size();
            
            state.params.window_set_width(size.0).ok();
            state.params.window_set_height(size.1).ok();
            
            let position = app_window.position();
            
            state.params.window_set_x(position.0).ok();
            state.params.window_set_y(position.1).ok();
            
        }
        
        let new_maximized = app_window.is_maximized();
        
        state.params.window_set_maximized(new_maximized);
        
    }
    
    let mut exit_code = 0;
    
    // save config
    
    let cfgpath = state.params.args_free_value("--config").map_or_else(||
        env::current_exe().unwrap().with_extension("cfg"),
        PathBuf::from
    );
    
    if let Err(error) = state.params.config_save(&cfgpath) {
        
        let mut message = error.to_string();
        let config_dialog = &state.ui.widgets().dialogs.general.config_save_error.dialog;
        
        let result = loop {
            
            state.ui.widgets().dialogs.general.config_save_error.path_label.set_text(&cfgpath.to_string_lossy());
            state.ui.widgets().dialogs.general.config_save_error.message_label.set_text(&message);
            
            let response = config_dialog.run();
            
            config_dialog.unrealize();
            config_dialog.hide();
            
            match response {
                
                // try again
                
                gtk::ResponseType::Ok => {
                    
                    if let Err(error) = state.params.config_save(&cfgpath) {
                        message = error.to_string();
                        continue;
                    }
                    
                    break 0;
                    
                },
                
                // give up
                
                _ => break 1,
                
            }
            
        };
        
        exit_code = result;
        
    }
    
    // save database
    
    if let Err(error) = state.database.save(state.params.paths_database(true)) {
        
        let mut message = error.to_string();
        let database_dialog = &state.ui.widgets().dialogs.general.database_save_error.dialog;
        
        let result = loop {
            
            state.ui.widgets().dialogs.general.database_save_error.path_label.set_text(&state.params.paths_database(true).to_string_lossy());
            state.ui.widgets().dialogs.general.database_save_error.message_label.set_text(&message);
            
            let response = database_dialog.run();
            
            database_dialog.unrealize();
            database_dialog.hide();
            
            match response {
                
                // try again
                
                gtk::ResponseType::Ok => {
                    
                    if let Err(error) = state.database.save(state.params.paths_database(true)) {
                        message = error.to_string();
                        continue;
                    }
                    
                    break 0;
                    
                },
                
                // give up
                
                _ => break 1,
                
            }
            
        };
        
        exit_code |= result;
        
    }
    
    // exit
    
    process::exit(exit_code);
    
}


// ---------- section ----------


fn section_focus(state: &State) {
    if let Some(treeview) = state.ui.files_current_treeview() {
        treeview.grab_focus();
    } else if let Some(treeview) = state.ui.watchlist_current_treeview() {
        treeview.grab_focus();
    }
}

fn section_next(state: &State) {
    
    fn switch(current_listbox: &gtk::ListBox, next_listbox: &gtk::ListBox) -> bool {
        if let Some(selected) = current_listbox.selected_row() {
            
            let new = selected.index() + 1;
            
            if let Some(row) = current_listbox.row_at_index(new) {
                row.activate();
            } else if let Some(row) = next_listbox.row_at_index(0) {
                row.activate();
            }
            
            return true;
            
        }
        
        false
    }
    
    // files -> watchlist
    
    if switch(&state.ui.widgets().window.files.listbox, &state.ui.widgets().window.watchlist.listbox) {
        return;
    }
    
    // watchlist -> preferences
    
    if switch(&state.ui.widgets().window.watchlist.listbox, &state.ui.widgets().window.preferences.listbox) {
        return;
    }
    
    // preferences -> files
    
    switch(&state.ui.widgets().window.preferences.listbox, &state.ui.widgets().window.files.listbox);
    
}

fn section_previous(state: &State) {
    
    fn switch(current_listbox: &gtk::ListBox, next_listbox: &gtk::ListBox) -> bool {
        if let Some(selected) = current_listbox.selected_row() {
            
            let new = selected.index() - 1;
            
            if let Some(row) = current_listbox.row_at_index(new) {
                row.activate();
            } else {
                
                let new = next_listbox.children().len() - 1;
                
                if let Some(row) = next_listbox.row_at_index(new as i32) {
                    row.activate();
                }
                
            }
            
            return true;
            
        }
        
        false
    }
    
    // files -> preferences
    
    if switch(&state.ui.widgets().window.files.listbox, &state.ui.widgets().window.preferences.listbox) {
        return;
    }
    
    // watchlist -> files
    
    if switch(&state.ui.widgets().window.watchlist.listbox, &state.ui.widgets().window.files.listbox) {
        return;
    }
    
    // preferences -> watchlist
    
    switch(&state.ui.widgets().window.preferences.listbox, &state.ui.widgets().window.watchlist.listbox);
    
}

fn section_switch_files(state: &State) {
    if let Some(selected) = state.ui.widgets().window.files.listbox.selected_row() {
        state.ui.widgets().window.files.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Files");
        
        state.ui.widgets().window.watchlist.listbox.unselect_all();
        state.ui.widgets().window.preferences.listbox.unselect_all();
        
        state.ui.widgets().menus.files.bar.menu.show();
        state.ui.widgets().menus.watchlist.bar.menu.hide();
        state.ui.widgets().menus.preferences.bar.menu.hide();
    }
}

fn section_switch_watchlist(state: &State) {
    if let Some(selected) = state.ui.widgets().window.watchlist.listbox.selected_row() {
        state.ui.widgets().window.watchlist.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Watchlist");
        
        state.ui.widgets().window.files.listbox.unselect_all();
        state.ui.widgets().window.preferences.listbox.unselect_all();
        
        state.ui.widgets().menus.files.bar.menu.hide();
        state.ui.widgets().menus.watchlist.bar.menu.show();
        state.ui.widgets().menus.preferences.bar.menu.hide();
    }
}

fn section_switch_preferences(state: &State) {
    if let Some(selected) = state.ui.widgets().window.preferences.listbox.selected_row() {
        state.ui.widgets().window.preferences.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Preferences");
        
        state.ui.widgets().window.files.listbox.unselect_all();
        state.ui.widgets().window.watchlist.listbox.unselect_all();
        
        state.ui.widgets().menus.files.bar.menu.hide();
        state.ui.widgets().menus.watchlist.bar.menu.hide();
        state.ui.widgets().menus.preferences.bar.menu.show();
    }
}


// ---------- search ----------


fn search_compute(state: &mut State) {
    let search_store = &state.ui.widgets().stores.general.search.store;
    let search_entry = &state.ui.widgets().window.general.search_entry;
    
    let input = search_entry.text();
    
    if input.is_empty() {
        search_store.clear();
        return;
    }
    
    let previous = unsafe {
        search_entry.data::<gtk::glib::GString>("previous-input")
        .filter(|previous| previous.as_ref() == &input)
    };
    
    if previous.is_none() {
        
        search_store.clear();
        
        state.ui.widgets().stores.general.search.sort.set_sort_column_id(gtk::SortColumn::Default, gtk::SortType::Ascending);
        
        // files
        
        // 0 => true
        // 1 => path
        // 2 => container (if any) + file stem + (status) + (f)
        
        let files_store = &state.ui.widgets().stores.files.entries.store;
        
        files_store.foreach(|_, _, store_iter| {
            // skip containers
            if ! files_store.iter_has_child(store_iter) {
                
                let display = match files_store.iter_parent(store_iter) {
                    
                    Some(parent_iter) => {
                        
                        let container = files_store.value(&parent_iter, 3).get::<glib::GString>().unwrap();
                        let file_stem = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        // skip element if neither its container nor its file_stem seem relevant
                        if ! case_insensitive_contains(&container, &input) && ! case_insensitive_contains(&file_stem, &input) {
                            return false;
                        }
                        
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let section = FilesSection::from(watched);
                        let display = section.display();
                        
                        let mut composite = String::with_capacity(container.len() + 1 + file_stem.len() + 7 + display.len());
                        
                        composite.push_str(&container);
                        composite.push(MAIN_SEPARATOR);
                        composite.push_str(&file_stem);
                        composite.push_str(" (");
                        composite.push_str(display);
                        composite.push_str(") (f)");
                        
                        composite
                        
                    },
                    
                    None => {
                        
                        let file_stem = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        // skip element if its file_stem does not seem relevant
                        if ! case_insensitive_contains(&file_stem, &input) {
                            return false;
                        }
                        
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let section = FilesSection::from(watched);
                        
                        concat_str!(&file_stem, " (", section.display(), ") (f)")
                        
                    },
                    
                };
                
                let filepath = files_store.value(store_iter, 0).get::<glib::GString>().unwrap();
                
                search_store.insert_with_values(
                    None,
                    &[
                        (0, &true),
                        (1, &filepath),
                        (2, &display),
                    ],
                );
                
            }
            
            false
        });
        
        // watchlist
        
        // 0 => false
        // 1 => title
        // 2 => title + (status) + (w)
        
        let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
        
        watchlist_store.foreach(|_, _, store_iter| {
            let title = watchlist_store.value(store_iter, 3).get::<glib::GString>().unwrap();
            
            if case_insensitive_contains(&title, &input) {
                let status = watchlist_store.value(store_iter, 2).get::<u8>().unwrap();
                let section = WatchlistSection::try_from(status).unwrap();
                
                let display = concat_str!(&title, " (", section.display(), ") (w)");
                
                search_store.insert_with_values(
                    None,
                    &[
                        (0, &false),
                        (1, &title),
                        (2, &display),
                    ],
                );
            }
            
            false
        });
        
        state.ui.widgets().stores.general.search.sort.set_sort_column_id(gtk::SortColumn::Index(0), gtk::SortType::Ascending);
        
        unsafe { search_entry.set_data("previous-input", input) };
        
    }
    
    state.ui.widgets().window.general.search_completion.complete();
}

fn search_should_recompute(state: &State) {
    unsafe { state.ui.widgets().window.general.search_entry.steal_data::<gtk::glib::GString>("previous-input") };
}

fn search_show(state: &State) {
    state.ui.widgets().window.general.search_entry.emit_by_name::<()>("search_changed", &[]);
}

fn search_focus(state: &State) {
    state.ui.widgets().window.general.search_entry.grab_focus();
}

fn search_select(state: &State, string_iter: &str) {
    let search_sort = &state.ui.widgets().stores.general.search.sort;
    
    let search_iter = match search_sort.iter_from_string(string_iter) {
        Some(search_iter) => search_iter,
        None => return,
    };
    
    let is_file = search_sort.value(&search_iter, 0).get::<bool>().unwrap();
    
    if is_file {
        
        let files_store = &state.ui.widgets().stores.files.entries.store;
        
        let file_path = search_sort.value(&search_iter, 1).get::<glib::GString>().unwrap();
        
        files_store.foreach(|_, store_path, store_iter| {
            let current = files_store.value(store_iter, 0).get::<glib::GString>().unwrap();
            
            if current == file_path {
                
                let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                let section = FilesSection::from(watched);
                
                if let Some(row) = state.ui.widgets().window.files.listbox.children().iter().find(|child| child.widget_name() == section.display()) {
                    
                    row.activate();
                    
                    if let Some(treeview) = state.ui.files_current_treeview() {
                        
                        let sort: gtk::TreeModelSort = treeview.model().unwrap().downcast().unwrap();
                        let filter: gtk::TreeModelFilter = sort.model().downcast().unwrap();
                        
                        let filter_path = filter.convert_child_path_to_path(store_path).unwrap();
                        let sort_path = sort.convert_child_path_to_path(&filter_path).unwrap();
                        
                        treeview.expand_to_path(&sort_path);
                        treeview.set_cursor(&sort_path, None::<&gtk::TreeViewColumn>, false);
                        treeview.grab_focus();
                        
                    }
                    
                }
                
                return true;
                
            }
            
            false
        });
        
    } else {
        
        let watchlist_store = &state.ui.widgets().stores.watchlist.entries.store;
        
        let title = search_sort.value(&search_iter, 1).get::<glib::GString>().unwrap();
        
        watchlist_store.foreach(|_, store_path, store_iter| {
            let current = watchlist_store.value(store_iter, 3).get::<glib::GString>().unwrap();
            
            if current == title {
                
                let status = watchlist_store.value(store_iter, 2).get::<u8>().unwrap();
                let section = WatchlistSection::try_from(status).unwrap();
                
                if let Some(row) = state.ui.widgets().window.watchlist.listbox.children().iter().find(|child| child.widget_name() == section.display()) {
                    
                    row.activate();
                    
                    if let Some(treeview) = state.ui.watchlist_current_treeview() {
                        
                        let sort: gtk::TreeModelSort = treeview.model().unwrap().downcast().unwrap();
                        let filter: gtk::TreeModelFilter = sort.model().downcast().unwrap();
                        
                        let filter_path = filter.convert_child_path_to_path(store_path).unwrap();
                        let sort_path = sort.convert_child_path_to_path(&filter_path).unwrap();
                        
                        treeview.expand_to_path(&sort_path);
                        treeview.set_cursor(&sort_path, None::<&gtk::TreeViewColumn>, false);
                        treeview.grab_focus();
                        
                    }
                    
                }
                
                return true;
                
            }
            
            false
        });
        
    }
}


// ---------- misc ----------


pub fn open(resource: &str) -> Result<(), Box<dyn Error>> {
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew
        fn ShellExecuteW(
            hwnd: *mut c_void, // HWND -> *mut HWND__
            lpOperation: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
            lpFile: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
            lpParameters: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
            lpDirectory: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
            nShowCmd: i32, // c_int
        ) -> *mut c_void; // HINSTANCE -> *mut HINSTANCE__
        
    }
    
    let operation: Vec<u16> = "open".encode_utf16()
        .chain(Some(0))
        .collect();
    
    let file: Vec<u16> = resource.encode_utf16()
        .chain(Some(0))
        .collect();
    
    let result = unsafe {
        
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            ptr::null(),
            ptr::null(),
            5, // SW_SHOW
        )
        
    };
    
    if result as isize <= 32 {
        return Err(io::Error::last_os_error().into());
    }
    
    Ok(())
    
}

pub fn current_date() -> String {
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
        fn GetLocalTime(
            lpSystemTime: *mut SYSTEMTIME, // LPSYSTEMTIME
        );
        
    }
    
    #[repr(C)]
    #[allow(non_snake_case)]
    // https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime
    struct SYSTEMTIME {
        wYear: u16, // WORD -> c_ushort
        wMonth: u16, // WORD -> c_ushort
        wDayOfWeek: u16, // WORD -> c_ushort
        wDay: u16, // WORD -> c_ushort
        wHour: u16, // WORD -> c_ushort
        wMinute: u16, // WORD -> c_ushort
        wSecond: u16, // WORD -> c_ushort
        wMilliseconds: u16, // WORD -> c_ushort
    }
    
    let mut st = unsafe {
        
        mem::zeroed::<SYSTEMTIME>()
        
    };
    
    unsafe {
        
        GetLocalTime(
            &mut st,
        );
        
    }
    
    format!("{:04}{:02}{:02}", st.wYear, st.wMonth, st.wDay)
    
}

pub fn percent_encode(value: &str) -> String {
	
	const ENCODED: &str = concat!(
        "%00%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F",
        "%10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F",
        "%20%21%22%23%24%25%26%27%28%29%2A%2B%2C%2D%2E%2F",
        "%30%31%32%33%34%35%36%37%38%39%3A%3B%3C%3D%3E%3F",
        "%40%41%42%43%44%45%46%47%48%49%4A%4B%4C%4D%4E%4F",
        "%50%51%52%53%54%55%56%57%58%59%5A%5B%5C%5D%5E%5F",
        "%60%61%62%63%64%65%66%67%68%69%6A%6B%6C%6D%6E%6F",
        "%70%71%72%73%74%75%76%77%78%79%7A%7B%7C%7D%7E%7F",
        "%80%81%82%83%84%85%86%87%88%89%8A%8B%8C%8D%8E%8F",
        "%90%91%92%93%94%95%96%97%98%99%9A%9B%9C%9D%9E%9F",
        "%A0%A1%A2%A3%A4%A5%A6%A7%A8%A9%AA%AB%AC%AD%AE%AF",
        "%B0%B1%B2%B3%B4%B5%B6%B7%B8%B9%BA%BB%BC%BD%BE%BF",
        "%C0%C1%C2%C3%C4%C5%C6%C7%C8%C9%CA%CB%CC%CD%CE%CF",
        "%D0%D1%D2%D3%D4%D5%D6%D7%D8%D9%DA%DB%DC%DD%DE%DF",
        "%E0%E1%E2%E3%E4%E5%E6%E7%E8%E9%EA%EB%EC%ED%EE%EF",
        "%F0%F1%F2%F3%F4%F5%F6%F7%F8%F9%FA%FB%FC%FD%FE%FF",
    );
	
	let to_be_replaced = value.bytes()
        .filter(|byte| ! byte.is_ascii_alphanumeric())
        .count();
    
    let mut result = String::with_capacity(value.len() + (to_be_replaced * 2));
    
    for byte in value.bytes() {
		
        if ! byte.is_ascii_alphanumeric() {
            let index = usize::from(byte) * 3;
            result.push_str(&ENCODED[index..index + 3]);
            continue;
        }
        
        result.push(char::from(byte));
        
    }
    
    result
	
}

pub fn natural_cmp(first: &str, second: &str) -> Ordering {
    
    fn extract_number(chars: &mut Peekable<Chars>) -> u32 {
        let mut number = 0;
        
        while let Some(curr) = chars.peek() {
            
            if let Some(digit) = curr.to_digit(10) {
                number = number * 10 + digit;
                chars.next();
                continue;
            }
            
            break;
            
        }
        
        number
    }
    
    // this will case-insensitively consider "a2" as less than "a10"
    
    let mut fr_chars = first.chars().peekable();
    let mut sd_chars = second.chars().peekable();
    
    while let (Some(fr_curr), Some(sd_curr)) = (fr_chars.peek(), sd_chars.peek()) {
        
        let fr_curr = fr_curr.to_ascii_lowercase();
        let sd_curr = sd_curr.to_ascii_lowercase();
        
        if fr_curr != sd_curr {
            
            if fr_curr.is_numeric() && sd_curr.is_numeric() {
                
                let order = extract_number(&mut fr_chars).cmp(&extract_number(&mut sd_chars));
                
                if order != Ordering::Equal {
                    return order;
                }
                
                fr_chars.next();
                sd_chars.next();
                
                continue;
                
            }
            
            return fr_curr.cmp(&sd_curr);
            
        }
        
        fr_chars.next();
        sd_chars.next();
        
    }
    
    first.len().cmp(&second.len())
    
}

pub fn case_insensitive_contains(base: &str, search: &str) -> bool {
    // windows method panics on zero size
    if search.is_empty() {
        return true;
    }
    
    if base.is_ascii() && search.is_ascii() {
        return base.as_bytes()
            .windows(search.len())
            .any(|window| window.eq_ignore_ascii_case(search.as_bytes()));
    }
    
    base.to_ascii_lowercase().contains(&search.to_ascii_lowercase())
}

macro_rules! concat_str {
    
    ( ) => { String::new() };
    
    ( $( $component:expr ),+ ) => {{
        
        let mut capacity = 0;
        $( capacity += $component.len(); )+
        
        let mut string = String::with_capacity(capacity);
        $( string.push_str($component); )+
        
        string
        
    }};
    
}

pub(crate) use concat_str;
