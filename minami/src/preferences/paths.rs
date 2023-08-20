use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, FilesActions, GeneralActions,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    state.ui.widgets().window.preferences.paths.files_entry.set_text(&state.params.paths_files(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.downloads_entry.set_text(&state.params.paths_downloads(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.pipe_entry.set_text(&state.params.paths_pipe(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.database_entry.set_text(&state.params.paths_database(false).to_string_lossy());
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let unlock_action = gio::SimpleAction::new("preferences.paths.unlock", None);
    let confirm_action = gio::SimpleAction::new("preferences.paths.confirm", None);
    let discard_action = gio::SimpleAction::new("preferences.paths.discard", None);
    
    // sensitivize fields and buttons
    unlock_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::PathsUnlock)).unwrap()
    });
    
    // commit changes and desensitivize fields and buttons
    confirm_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::PathsConfirm)).unwrap()
    });
    
    confirm_action.set_enabled(false);
    
    // discard changes and desensitivize fields and buttons
    discard_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::PathsDiscard)).unwrap()
    });
    
    discard_action.set_enabled(false);
    
    app.add_action(&unlock_action);
    app.add_action(&confirm_action);
    app.add_action(&discard_action);
    
    // ---------- entries ----------
    
    let entries = [
        &state.ui.widgets().window.preferences.paths.files_entry,
        &state.ui.widgets().window.preferences.paths.downloads_entry,
        &state.ui.widgets().window.preferences.paths.pipe_entry,
        &state.ui.widgets().window.preferences.paths.database_entry,
    ];
    
    for entry in entries {
        
        // prevent movement (Up Arrow)
        // prevent movement (Down Arrow)
        entry.connect_key_press_event({
            move |_, eventkey| {
                match eventkey.keyval() {
                    gdk::keys::constants::Up => glib::Propagation::Stop,
                    gdk::keys::constants::Down => glib::Propagation::Stop,
                    _ => glib::Propagation::Proceed,
                }
            }
        });
        
    }
    
    // ---------- buttons ----------
    
    // open chooser for files field
    state.ui.widgets().window.preferences.paths.files_button.connect_clicked({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::Preferences(PreferencesActions::PathsChooseFiles)).unwrap()
    });
    
    // open chooser for downloads field
    state.ui.widgets().window.preferences.paths.downloads_button.connect_clicked({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::Preferences(PreferencesActions::PathsChooseDownloads)).unwrap()
    });
    
    // open chooser for database field
    state.ui.widgets().window.preferences.paths.database_button.connect_clicked({
        let sender_cloned = sender.clone();
        move |_| sender_cloned.send(Message::Preferences(PreferencesActions::PathsChooseDatabase)).unwrap()
    });
    
    // focus global search entry (SHIFT + Tab)
    state.ui.widgets().window.preferences.paths.files_button.connect_key_press_event({
        let sender_cloned = sender.clone();
        move |_, eventkey| {
            if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
                sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    });
    
    let choosers = [
        &state.ui.widgets().window.preferences.paths.files_button,
        &state.ui.widgets().window.preferences.paths.downloads_button,
        &state.ui.widgets().window.preferences.paths.database_button,
    ];
    
    for chooser in choosers {
        
        // prevent movement (Up Arrow)
        // prevent movement (Right Arrow)
        // prevent movement (Down Arrow)
        // prevent movement (Left Arrow)
        chooser.connect_key_press_event({
            move |_, eventkey| {
                match eventkey.keyval() {
                    gdk::keys::constants::Up => glib::Propagation::Stop,
                    gdk::keys::constants::Right => glib::Propagation::Stop,
                    gdk::keys::constants::Down => glib::Propagation::Stop,
                    gdk::keys::constants::Left => glib::Propagation::Stop,
                    _ => glib::Propagation::Proceed,
                }
            }
        });
        
    }
    
    for button in &state.ui.widgets().window.preferences.paths.buttons_box.children() {
        
        // prevent selection of last media field (Up Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Up {
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        });
        
    }
    
    if let Some(button) = state.ui.widgets().window.preferences.paths.buttons_box.children().first() {
        
        // prevent selection of first paths chooser button (Left Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Left {
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        });
        
    }
    
    if let Some(button) = state.ui.widgets().window.preferences.paths.buttons_box.children().iter().find(|button| button.is_sensitive()) {
        
        // focus global search entry (SHIFT + Tab)
        button.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
                    sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        });
        
    }
}

pub fn choose_files(state: &State) {
    let entry = &state.ui.widgets().window.preferences.paths.files_entry;
    let file_chooser = &state.ui.widgets().dialogs.general.file_chooser.dialog;
    
    file_chooser.set_title("Select files directory");
    file_chooser.set_action(gtk::FileChooserAction::SelectFolder);
    
    show_chooser(file_chooser, entry);
}

pub fn choose_downloads(state: &State) {
    let entry = &state.ui.widgets().window.preferences.paths.downloads_entry;
    let file_chooser = &state.ui.widgets().dialogs.general.file_chooser.dialog;
    
    file_chooser.set_title("Select downloads directory");
    file_chooser.set_action(gtk::FileChooserAction::SelectFolder);
    
    show_chooser(file_chooser, entry);
}

pub fn choose_database(state: &State) {
    let entry = &state.ui.widgets().window.preferences.paths.database_entry;
    let file_chooser = &state.ui.widgets().dialogs.general.file_chooser.dialog;
    
    file_chooser.set_title("Select database file");
    file_chooser.set_action(gtk::FileChooserAction::Save);
    
    show_chooser(file_chooser, entry);
}

fn show_chooser(chooser: &gtk::FileChooserNative, entry: &gtk::Entry) {
    loop {
        
        let response = chooser.run();
        
        chooser.hide();
        
        match response {
            
            // confirm
            
            gtk::ResponseType::Accept => {
                
                if let Some(chosen) = chooser.filename() {
                    entry.set_text(&chosen.to_string_lossy());
                    break;
                }
                
            },
            
            // cancel
            
            _ => break,
            
        }
        
    }
}

pub fn unlock(state: &State) {
    sensitivize_fields_and_buttons(state, true);
}

pub fn confirm(state: &mut State, sender: &Sender<Message>) {
    let mut success = true;
    
    success &= commit_files(state, sender);
    success &= commit_downloads(state);
    success &= commit_pipe(state);
    success &= commit_database(state, sender);
    
    if success {
        sensitivize_fields_and_buttons(state, false);
    }
}

pub fn discard(state: &State) {
    state.ui.widgets().window.preferences.paths.files_entry.set_text(&state.params.paths_files(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.downloads_entry.set_text(&state.params.paths_downloads(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.pipe_entry.set_text(&state.params.paths_pipe(false).to_string_lossy());
    state.ui.widgets().window.preferences.paths.database_entry.set_text(&state.params.paths_database(false).to_string_lossy());
    
    sensitivize_fields_and_buttons(state, false);
}

fn sensitivize_fields_and_buttons(state: &State, sensitive: bool) {
    state.ui.widgets().window.preferences.paths.files_button.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.paths.downloads_button.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.paths.database_button.set_sensitive(sensitive);
    
    state.ui.widgets().window.preferences.paths.files_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.paths.downloads_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.paths.pipe_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.paths.database_entry.set_sensitive(sensitive);
    
    if let Some(application) = state.ui.widgets().window.general.window.application() {
        
        // unlock action is turned sensitive when confirm and discard aren't and vice versa
        if let Some(unlock_action) = application.lookup_action("preferences.paths.unlock").and_downcast::<gio::SimpleAction>() {
            unlock_action.set_enabled(! sensitive);
        }
        
        if let Some(confirm_action) = application.lookup_action("preferences.paths.confirm").and_downcast::<gio::SimpleAction>() {
            confirm_action.set_enabled(sensitive);
        }
        
        if let Some(discard_action) = application.lookup_action("preferences.paths.discard").and_downcast::<gio::SimpleAction>() {
            discard_action.set_enabled(sensitive);
        }
        
    }
    
    let children = state.ui.widgets().window.preferences.paths.buttons_box.children();
    
    if let Some(child) = children.iter().find(|child| child.is_sensitive()) {
        child.grab_focus();
    }
    
}

fn commit_files(state: &mut State, sender: &Sender<Message>) -> bool {
    let input = state.ui.widgets().window.preferences.paths.files_entry.text();
    
    match state.params.paths_set_files(&input) {
        
        Ok(changed) => if changed {
            sender.send(Message::Files(FilesActions::Reload)).unwrap();
        },
        
        Err(error) => {
            state.ui.dialogs_error_show(&error.to_string());
            return false;
        },
        
    }
    
    true
}

fn commit_downloads(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.paths.downloads_entry.text();
    
    if let Err(error) = state.params.paths_set_downloads(&input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}

fn commit_pipe(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.paths.pipe_entry.text();
    
    if let Err(error) = state.params.paths_set_pipe(&input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}

fn commit_database(state: &mut State, sender: &Sender<Message>) -> bool {
    let path = state.ui.widgets().window.preferences.paths.database_entry.text();
    
    match state.params.paths_set_database(&path) {
        
        Ok(changed) => if changed && state.params.args_paths_database().is_none() {
            sender.send(Message::General(GeneralActions::ReloadDatabase)).unwrap();
        },
        
        Err(error) => {
            state.ui.dialogs_error_show(&error.to_string());
            return false;
        },
        
    }
    
    true
}
