use std::path::MAIN_SEPARATOR;

use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, FilesActions, WatchlistActions,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    bind(app, state, sender);
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let candidate_action = gio::SimpleAction::new("files.edit.candidate", None);
    let series_action = gio::SimpleAction::new("files.edit.series", None);
    let copy_action = gio::SimpleAction::new("files.edit.copy", None);
    
    // add candidate
    candidate_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::AddCandidate)).unwrap()
    });
    
    // add series
    series_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::AddSeries)).unwrap()
    });
    
    // copy names to clipboard
    copy_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::CopyNames)).unwrap()
    });
    
    app.add_action(&candidate_action);
    app.add_action(&series_action);
    app.add_action(&copy_action);
    
    // ---------- treeviews ----------
    
    let treeviews = [
        &state.ui.widgets().window.files.new_treeview,
        &state.ui.widgets().window.files.watched_treeview,
    ];
    
    for treeview in treeviews {
        
        // add candidate (Insert)
        // add series (SHIFT + Insert)
        // copy names to clipboard (CONTROL + C/c)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                match eventkey.keyval() {
                    
                    key if key == gdk::keys::constants::Insert && ! eventkey.state().contains(gdk::ModifierType::SHIFT_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::AddCandidate)).unwrap();
                    },
                    
                    key if key == gdk::keys::constants::Insert && eventkey.state().contains(gdk::ModifierType::SHIFT_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::AddSeries)).unwrap();
                    },
                    
                    key if (key == gdk::keys::constants::C || key == gdk::keys::constants::c) && eventkey.state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::CopyNames)).unwrap();
                    },
                    
                    _ => return Inhibit(false),
                    
                }
                
                Inhibit(true)
            }
        });
        
    }
}

pub fn add_candidate(state: &State, sender: &Sender<Message>) {
    if let Some(name) = selected_name(state) {
        sender.send(Message::Preferences(PreferencesActions::CandidatesAdd(Some(name)))).unwrap();
    }
}

pub fn add_series(state: &State, sender: &Sender<Message>) {
    if let Some(name) = selected_name(state) {
        sender.send(Message::Watchlist(WatchlistActions::Add(Some(name)))).unwrap();
    }
}

fn selected_name(state: &State) -> Option<String> {
    let treeview = state.ui.files_current_treeview()?;
    
    let (treepaths, treemodel) = treeview.selection().selected_rows();
    
    if treepaths.len() > 1 {
        treeview.set_cursor(treepaths.first()?, None::<&gtk::TreeViewColumn>, false);
    }
    
    let treeiter = treemodel.iter(treepaths.first()?)?;
    
    let name = match treemodel.iter_parent(&treeiter) {
        
        Some(parent_iter) => {
            
            let container = treemodel.value(&parent_iter, 3).get::<glib::GString>().unwrap();
            let file_stem = treemodel.value(&treeiter, 3).get::<glib::GString>().unwrap();
            
            let mut composite = String::with_capacity(container.len() + 1 + file_stem.len());
            
            composite.push_str(&container);
            composite.push(MAIN_SEPARATOR);
            composite.push_str(&file_stem);
            
            composite
            
        },
        
        None => treemodel.value(&treeiter, 3).get::<String>().unwrap()
        
    };
    
    Some(name)
}

pub fn copy_names(state: &State) {
    
    fn include_container(container: &str, file_stem: &str) -> String {
        let mut composite = String::with_capacity(container.len() + 1 + file_stem.len());
        
        composite.push_str(container);
        composite.push(MAIN_SEPARATOR);
        composite.push_str(file_stem);
        
        composite
    }
    
    let Some(treeview) = state.ui.files_current_treeview() else {
        return;
    };
    
    let selection = treeview.selection();
    
    let count = selection.count_selected_rows();
    
    if count == 0 {
        return;
    }
    
    let mut names = Vec::with_capacity(count as usize);
    
    selection.selected_foreach(|treemodel, _, treeiter| {
        match treemodel.iter_children(Some(treeiter)) {
            
            // subdirectory
            
            Some(iter_child) => {
                
                names.reserve(treemodel.iter_n_children(Some(treeiter)) as usize);
                
                let container = treemodel.value(treeiter, 3).get::<glib::GString>().unwrap();
                
                loop {
                    
                    let file_stem = treemodel.value(&iter_child, 3).get::<glib::GString>().unwrap();
                    names.push(include_container(&container, &file_stem));
                    
                    if ! treemodel.iter_next(&iter_child) {
                        break;
                    }
                    
                }
                
            },
            
            // file
            
            None => match treemodel.iter_parent(treeiter) {
                
                // in subdirectory
                
                Some(parent_iter) => {
                    
                    // skip if parent is selected
                    if selection.iter_is_selected(&parent_iter) {
                        return;
                    }
                    
                    let container = treemodel.value(&parent_iter, 3).get::<glib::GString>().unwrap();
                    let file_stem = treemodel.value(treeiter, 3).get::<glib::GString>().unwrap();
                    
                    names.push(include_container(&container, &file_stem));
                    
                },
                
                // in root
                
                None => {
                    
                    let file_stem = treemodel.value(treeiter, 3).get::<glib::GString>().unwrap();
                    names.push(file_stem.to_string());
                    
                },
                
            },
            
        }
    });
    
    let text = names.join("\n");
    
    state.ui.clipboard_set_text(&text);
    
}
