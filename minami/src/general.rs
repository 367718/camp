use std::{
    cmp::Ordering,
    env,
    error::Error,
    ffi::OsString,
    fs::{ self, OpenOptions },
    io,
    iter::Peekable,
    mem,
    os::raw::*,
    path::{ MAIN_SEPARATOR_STR, PathBuf },
    process,
    ptr,
    str::{ self, Chars },
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
    // CAUTION: this will discard uncommited changes in current database
    
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
    file_chooser_dialog.set_current_name(&concat_str!(APP_NAME, "-", &current_date(), ".db"));
    
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
        
        if name == PreferencesSection::Media.display() {
            
            if state.ui.widgets().window.preferences.media.player_entry.is_sensitive() {
                state.ui.widgets().window.preferences.media.player_entry.grab_focus();
                return;
            }
            
            focus_first_sensitive_child(&state.ui.widgets().window.preferences.media.buttons_box);
            return;
            
        }
        
        // ---------- paths ----------
        
        if name == PreferencesSection::Paths.display() {
            
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
            
            name if name == PreferencesSection::Candidates.display() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.candidates.downloaded_buttons_box);
            },
            
            // ---------- feeds ----------
            
            name if name == PreferencesSection::Feeds.display() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.feeds.buttons_box);
            },
            
            // ---------- kinds ----------
            
            name if name == PreferencesSection::Kinds.display() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.kinds.buttons_box);
            },
            
            // ---------- formats ----------
            
            name if name == PreferencesSection::Formats.display() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.formats.buttons_box);
            },
            
            // ---------- media ----------
            
            name if name == PreferencesSection::Media.display() => {
                focus_last_sensitive_child(&state.ui.widgets().window.preferences.media.buttons_box);
            },
            
            // ---------- paths ----------
            
            name if name == PreferencesSection::Paths.display() => {
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
                        if ! case_insensitive_contains(&container, &needles) && ! case_insensitive_contains(&file_stem, &needles) {
                            return false;
                        }
                        
                        let watched = files_store.value(store_iter, 2).get::<bool>().unwrap();
                        let section = FilesSection::from(watched);
                        
                        concat_str!(&container, MAIN_SEPARATOR_STR, &file_stem, " (", section.display(), ") (f)")
                        
                    },
                    
                    None => {
                        
                        let file_stem = files_store.value(store_iter, 3).get::<glib::GString>().unwrap();
                        
                        // skip element if its file_stem does not seem relevant
                        if ! case_insensitive_contains(&file_stem, &needles) {
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
            
            if case_insensitive_contains(&title, &needles) {
                let status = watchlist_store.value(store_iter, 2).get::<i64>().unwrap();
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
                
                let status = watchlist_store.value(store_iter, 2).get::<i64>().unwrap();
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


pub fn open(file: &str) -> Result<(), Box<dyn Error>> {
    let encoded_operation: Vec<c_ushort> = "open".encode_utf16()
        .chain(Some(0))
        .collect();
    
    let encoded_file: Vec<c_ushort> = file.encode_utf16()
        .chain(Some(0))
        .collect();
    
    let result = unsafe {
        
        crate::ffi::ShellExecuteW(
            ptr::null_mut(),
            encoded_operation.as_ptr(),
            encoded_file.as_ptr(),
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
    let mut st = unsafe {
        
        mem::zeroed::<crate::ffi::SYSTEMTIME>()
        
    };
    
    unsafe {
        
        crate::ffi::GetLocalTime(
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
        
        while let Some(digit) = chars.peek().and_then(|curr| curr.to_digit(10)) {
            number = number * 10 + digit;
            chars.next();
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

pub fn case_insensitive_contains(haystack: &str, needles: &[&str]) -> bool {
    
    fn contains(haystack: &str, needle: &str) -> bool {
        if haystack.is_ascii() && needle.is_ascii() {
            
            // windows method panics on zero size
            if needle.is_empty() {
                return false;
            }
            
            return haystack.as_bytes()
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()));
            
        }
        
        haystack.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
    }
    
    needles.iter().all(|needle| contains(haystack, needle))
    
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
