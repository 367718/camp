use std::{
    env,
    ffi::OsString,
    fs::{ self, OpenOptions },
    path::{ MAIN_SEPARATOR_STR, PathBuf },
    process,
    str,
};

use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    APP_NAME,
    State, Message,
    PreferencesSection, FilesSection, WatchlistSection,
    PreferencesActions,
    Database,
};

pub enum GeneralActions {
    
    // ---------- config and database ----------
    
    ReloadDatabase,
    BackupDatabase,
    SaveAndQuit,
    
    // ---------- section ----------
    
    SectionFocusStart,
    SectionFocusEnd,
    SectionSwitchNext,
    SectionSwitchPrevious,
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
    
    let file_load_dialog = &state.ui.widgets().dialogs.general.file_load_error.dialog;
    
    file_load_dialog.set_title("File load error");
    file_load_dialog.set_position(gtk::WindowPosition::CenterOnParent);
    file_load_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
    let error_dialog = &state.ui.widgets().dialogs.general.error.dialog;
    
    error_dialog.set_title("Error");
    error_dialog.set_position(gtk::WindowPosition::CenterOnParent);
    error_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
    let file_chooser_dialog = &state.ui.widgets().dialogs.general.file_chooser.dialog;
    
    file_chooser_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
    
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
    app.set_accels_for_action("app.general.section.focus.start", &["<Primary>E"]);
    app.set_accels_for_action("app.general.section.switch.next", &["<Primary>Page_Down"]);
    app.set_accels_for_action("app.general.section.switch.previous", &["<Primary>Page_Up"]);
    
    // ---------- actions ----------
    
    let quit_action = gio::SimpleAction::new("general.save_and_quit", None);
    let backup_action = gio::SimpleAction::new("general.backup_database", None);
    let search_focus_action = gio::SimpleAction::new("general.search.focus", None);
    let section_focus_start_action = gio::SimpleAction::new("general.section.focus.start", None);
    let section_switch_next_action = gio::SimpleAction::new("general.section.switch.next", None);
    let section_switch_previous_action = gio::SimpleAction::new("general.section.switch.previous", None);
    
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
    
    // focus start of visible section
    section_focus_start_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionFocusStart)).unwrap()
    });
    
    // change visible section (down)
    section_switch_next_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionSwitchNext)).unwrap()
    });
    
    // change visible section (up)
    section_switch_previous_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::General(GeneralActions::SectionSwitchPrevious)).unwrap()
    });
    
    app.add_action(&quit_action);
    app.add_action(&backup_action);
    app.add_action(&search_focus_action);
    app.add_action(&section_focus_start_action);
    app.add_action(&section_switch_next_action);
    app.add_action(&section_switch_previous_action);
    
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
    
    // focus start of visible section (Tab)
    // focus end of visible section (SHIFT + Tab)
    search_entry.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            match eventkey.keyval() {
                gdk::keys::constants::Tab => sender_cloned.send(Message::General(GeneralActions::SectionFocusStart)).unwrap(),
                gdk::keys::constants::ISO_Left_Tab => sender_cloned.send(Message::General(GeneralActions::SectionFocusEnd)).unwrap(),
                _ => return Inhibit(false),
            }
            Inhibit(true)
        }
    });
    
    // prevent movement (Down Arrow)
    search_entry.connect_key_press_event({
        move |_, eventkey| {
            if eventkey.keyval() == gdk::keys::constants::Down {
                return Inhibit(true);
            }
            Inhibit(false)
        }
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
        
        // general -> bind x2
        SectionFocusStart => section_focus_start(state),
        
        // general -> bind x2
        SectionFocusEnd => section_focus_end(state),
        
        // general -> bind
        SectionSwitchNext => section_switch_next(state),
        
        // general -> bind
        SectionSwitchPrevious => section_switch_previous(state),
        
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
    let mut database = None;
    
    let mut dbpath = state.params.paths_database(false).to_owned();
    let mut error = String::new();
    
    // success
    
    match Database::load(&dbpath) {
        Ok(data) => database = Some(data),
        Err(err) => error = err.to_string(),
    }
    
    // error
    
    if database.is_none() {
        
        let file_chooser_dialog = &state.ui.widgets().dialogs.general.file_chooser.dialog;
        
        file_chooser_dialog.set_title("Choose database path");
        file_chooser_dialog.set_action(gtk::FileChooserAction::Save);
        
        let file_load_dialog = &state.ui.widgets().dialogs.general.file_load_error.dialog;
        
        // enable "Select another"
        file_load_dialog.set_response_sensitive(gtk::ResponseType::Other(1), true);
        
        state.ui.widgets().dialogs.general.file_load_error.message_label.set_text("The database file could not be loaded.");
        
        database = 'outer: loop {
            
            state.ui.widgets().dialogs.general.file_load_error.path_label.set_text(&dbpath.to_string_lossy());
            state.ui.widgets().dialogs.general.file_load_error.error_label.set_text(&error);
            
            let response = file_load_dialog.run();
            
            file_load_dialog.unrealize();
            file_load_dialog.hide();
            
            match response {
                
                // generate new
                
                gtk::ResponseType::Other(0) => match Database::new(&dbpath) {
                    Ok(generated) => break 'outer Some(generated),
                    Err(err) => error = err.to_string(),
                },
                
                // select another
                
                gtk::ResponseType::Other(1) => 'inner: loop {
                    
                    let response = file_chooser_dialog.run();
                    
                    file_chooser_dialog.hide();
                    
                    match response {
                        
                        // confirm
                        
                        gtk::ResponseType::Accept => {
                            
                            if let Some(chosen) = file_chooser_dialog.filename() {
                                
                                dbpath = chosen;
                                
                                let result = Database::load(&dbpath).and_then(|database| {
                                    state.params.paths_set_database(&dbpath)?;
                                    Ok(database)
                                });
                                
                                match result {
                                    Ok(database) => break 'outer Some(database),
                                    Err(err) => error = err.to_string(),
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
    let file_chooser_dialog = &state.ui.widgets().dialogs.general.file_chooser.dialog;
    
    file_chooser_dialog.set_title("Backup database");
    file_chooser_dialog.set_action(gtk::FileChooserAction::Save);
    file_chooser_dialog.set_current_name(&chikuwa::concat_str!(APP_NAME, "-", &chikuwa::current_date(), ".db"));
    
    loop {
        
        let response = file_chooser_dialog.run();
        
        file_chooser_dialog.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Accept => {
                
                if let Some(mut path) = file_chooser_dialog.filename() {
                    
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
                        .map_err(|_| chikuwa::concat_str!("File already exists or write error: ", &path.to_string_lossy()));
                    
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
    
    if let Err(err) = state.params.config_save(&cfgpath) {
        
        let mut error = err.to_string();
        let file_save_dialog = &state.ui.widgets().dialogs.general.file_save_error.dialog;
        
        state.ui.widgets().dialogs.general.file_save_error.message_label.set_text("The configuration file could not be saved.");
        
        let result = loop {
            
            state.ui.widgets().dialogs.general.file_save_error.path_label.set_text(&cfgpath.to_string_lossy());
            state.ui.widgets().dialogs.general.file_save_error.error_label.set_text(&error);
            
            let response = file_save_dialog.run();
            
            file_save_dialog.unrealize();
            file_save_dialog.hide();
            
            match response {
                
                // try again
                
                gtk::ResponseType::Ok => {
                    
                    if let Err(err) = state.params.config_save(&cfgpath) {
                        error = err.to_string();
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
    
    // exit
    
    process::exit(exit_code);
    
}


// ---------- section ----------


fn section_focus_start(state: &State) {
    
    fn focus_first_sensitive_child(parent_box: &gtk::Box) {
        if let Some(child) = parent_box.children().iter().find(|child| child.is_sensitive()) {
            child.grab_focus();
        }
    }
    
    // ---------- files ----------
    
    if let Some(treeview) = state.ui.files_current_treeview() {
        treeview.grab_focus();
        return;
    }
    
    // ---------- watchlist ----------
    
    if let Some(treeview) = state.ui.watchlist_current_treeview() {
        treeview.grab_focus();
        return;
    }
    
    // ---------- preferences ----------
    
    if let Some(selected) = state.ui.widgets().window.preferences.listbox.selected_row() {
        
        // ---------- candidates, feeds, kinds, formats ----------
        
        if let Some(treeview) = state.ui.preferences_current_treeview() {
            treeview.grab_focus();
            return;
        }
        
        let name = selected.widget_name();
        
        // ---------- media ----------
        
        if name == PreferencesSection::Media.to_str() {
            
            if state.ui.widgets().window.preferences.media.player_entry.is_sensitive() {
                state.ui.widgets().window.preferences.media.player_entry.grab_focus();
                return;
            }
            
            focus_first_sensitive_child(&state.ui.widgets().window.preferences.media.buttons_box);
            return;
            
        }
        
        // ---------- paths ----------
        
        if name == PreferencesSection::Paths.to_str() {
            
            if state.ui.widgets().window.preferences.paths.files_button.is_sensitive() {
                state.ui.widgets().window.preferences.paths.files_button.grab_focus();
                return;
            }
            
            focus_first_sensitive_child(&state.ui.widgets().window.preferences.paths.buttons_box);
            
        }
        
    }
    
}

fn section_focus_end(state: &State) {
    
    fn focus_last_sensitive_child(parent_box: &gtk::Box) {
        if let Some(child) = parent_box.children().iter().rev().find(|child| child.is_sensitive()) {
            child.grab_focus();
        }
    }
    
    // ---------- files ----------
    
    if state.ui.widgets().window.files.listbox.selected_row().is_some() {
        focus_last_sensitive_child(&state.ui.widgets().window.files.buttons_box);
        return;
    }
    
    // ---------- watchlist ----------
    
    if state.ui.widgets().window.watchlist.listbox.selected_row().is_some() {
        focus_last_sensitive_child(&state.ui.widgets().window.watchlist.buttons_box);
        return;
    }
    
    // ---------- preferences ----------
    
    if let Some(selected) = state.ui.widgets().window.preferences.listbox.selected_row() {
        match selected.widget_name() {
            
            // ---------- candidates ----------
            
            name if name == PreferencesSection::Candidates.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.candidates.downloaded_buttons_box);
            },
            
            // ---------- feeds ----------
            
            name if name == PreferencesSection::Feeds.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.feeds.buttons_box);
            },
            
            // ---------- kinds ----------
            
            name if name == PreferencesSection::Kinds.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.kinds.buttons_box);
            },
            
            // ---------- formats ----------
            
            name if name == PreferencesSection::Formats.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.formats.buttons_box);
            },
            
            // ---------- media ----------
            
            name if name == PreferencesSection::Media.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.media.buttons_box);
            },
            
            // ---------- paths ----------
            
            name if name == PreferencesSection::Paths.to_str() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.paths.buttons_box);
            },
            
            _ => (),
            
        }
    }
    
}

fn section_switch_next(state: &State) {
    
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
    
    // from files to watchlist
    
    if switch(&state.ui.widgets().window.files.listbox, &state.ui.widgets().window.watchlist.listbox) {
        return;
    }
    
    // from watchlist to preferences
    
    if switch(&state.ui.widgets().window.watchlist.listbox, &state.ui.widgets().window.preferences.listbox) {
        return;
    }
    
    // from preferences to files
    
    switch(&state.ui.widgets().window.preferences.listbox, &state.ui.widgets().window.files.listbox);
    
}

fn section_switch_previous(state: &State) {
    
    fn switch(current_listbox: &gtk::ListBox, next_listbox: &gtk::ListBox) -> bool {
        if let Some(selected) = current_listbox.selected_row() {
            
            let new = selected.index() - 1;
            
            if let Some(row) = current_listbox.row_at_index(new) {
                row.activate();
            } else {
                
                let new = i32::try_from(next_listbox.children().len() - 1).unwrap_or(0);
                
                if let Some(row) = next_listbox.row_at_index(new) {
                    row.activate();
                }
                
            }
            
            return true;
            
        }
        
        false
    }
    
    // from files to preferences
    
    if switch(&state.ui.widgets().window.files.listbox, &state.ui.widgets().window.preferences.listbox) {
        return;
    }
    
    // from watchlist to files
    
    if switch(&state.ui.widgets().window.watchlist.listbox, &state.ui.widgets().window.files.listbox) {
        return;
    }
    
    // from preferences to watchlist
    
    switch(&state.ui.widgets().window.preferences.listbox, &state.ui.widgets().window.watchlist.listbox);
    
}

fn section_switch_files(state: &State) {
    if let Some(selected) = state.ui.widgets().window.files.listbox.selected_row() {
        
        state.ui.widgets().window.files.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Files");
        
        state.ui.widgets().window.watchlist.listbox.unselect_all();
        state.ui.widgets().window.preferences.listbox.unselect_all();
        
        state.ui.widgets().menus.files.menu.show();
        state.ui.widgets().menus.watchlist.menu.hide();
        state.ui.widgets().menus.preferences.menu.hide();
        
        section_focus_start(state);
        
    }
}

fn section_switch_watchlist(state: &State) {
    if let Some(selected) = state.ui.widgets().window.watchlist.listbox.selected_row() {
        
        state.ui.widgets().window.watchlist.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Watchlist");
        
        state.ui.widgets().window.files.listbox.unselect_all();
        state.ui.widgets().window.preferences.listbox.unselect_all();
        
        state.ui.widgets().menus.files.menu.hide();
        state.ui.widgets().menus.watchlist.menu.show();
        state.ui.widgets().menus.preferences.menu.hide();
        
        section_focus_start(state);
        
    }
}

fn section_switch_preferences(state: &State) {
    if let Some(selected) = state.ui.widgets().window.preferences.listbox.selected_row() {
        
        state.ui.widgets().window.preferences.stack.set_visible_child_name(&selected.widget_name());
        
        state.ui.widgets().window.general.sections_stack.set_visible_child_name("Preferences");
        
        state.ui.widgets().window.files.listbox.unselect_all();
        state.ui.widgets().window.watchlist.listbox.unselect_all();
        
        state.ui.widgets().menus.files.menu.hide();
        state.ui.widgets().menus.watchlist.menu.hide();
        state.ui.widgets().menus.preferences.menu.show();
        
        section_focus_start(state);
        
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
        
        let needles = input
            .split_whitespace()
            .collect::<Vec<&str>>();
        
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
                        if ! chikuwa::insensitive_contains(&container, &needles) && ! chikuwa::insensitive_contains(&file_stem, &needles) {
                            return false;
                        }
                        
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let section = FilesSection::from(watched);
                        
                        chikuwa::concat_str!(&container, MAIN_SEPARATOR_STR, &file_stem, " (", section.to_str(), ") (f)")
                        
                    },
                    
                    None => {
                        
                        let file_stem = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        // skip element if its file_stem does not seem relevant
                        if ! chikuwa::insensitive_contains(&file_stem, &needles) {
                            return false;
                        }
                        
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let section = FilesSection::from(watched);
                        
                        chikuwa::concat_str!(&file_stem, " (", section.to_str(), ") (f)")
                        
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
            
            if chikuwa::insensitive_contains(&title, &needles) {
                let status = watchlist_store.value(store_iter, 2).get::<i64>().unwrap();
                let section = WatchlistSection::try_from(status).unwrap();
                
                let display = chikuwa::concat_str!(&title, " (", section.to_str(), ") (w)");
                
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
    
    let Some(search_iter) = search_sort.iter_from_string(string_iter) else {
        return;
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
                
                if let Some(row) = state.ui.widgets().window.files.listbox.children().iter().find(|child| child.widget_name() == section.to_str()) {
                    
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
                
                let status = watchlist_store.value(store_iter, 2).get::<i64>().unwrap();
                let section = WatchlistSection::try_from(status).unwrap();
                
                if let Some(row) = state.ui.widgets().window.watchlist.listbox.children().iter().find(|child| child.widget_name() == section.to_str()) {
                    
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
