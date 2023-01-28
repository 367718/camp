use std::time::Duration;

use gtk::{
	gdk,
    gio,
    glib::Sender,
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
    state.ui.widgets().window.preferences.media.player_entry.set_text(state.params.media_player(false));
    state.ui.widgets().window.preferences.media.iconify_switch.set_state(state.params.media_iconify(false));
    state.ui.widgets().window.preferences.media.flag_entry.set_text(state.params.media_flag(false));
    state.ui.widgets().window.preferences.media.timeout_spin.set_value(state.params.media_timeout(false).as_secs_f64());
    state.ui.widgets().window.preferences.media.autoselect_switch.set_state(state.params.media_autoselect(false));
    state.ui.widgets().window.preferences.media.lookup_entry.set_text(state.params.media_lookup(false));
    state.ui.widgets().window.preferences.media.bind_entry.set_text(state.params.media_bind(false));
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let unlock_action = gio::SimpleAction::new("preferences.media.unlock", None);
    let confirm_action = gio::SimpleAction::new("preferences.media.confirm", None);
    let discard_action = gio::SimpleAction::new("preferences.media.discard", None);
    
    // sensitivize fields and buttons
    unlock_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::MediaUnlock)).unwrap()
    });
    
    // commit changes and desensitivize fields and buttons
    confirm_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::MediaConfirm)).unwrap()
    });
    
    confirm_action.set_enabled(false);
    
    // discard changes and desensitivize fields and buttons
    discard_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Preferences(PreferencesActions::MediaDiscard)).unwrap()
    });
    
    discard_action.set_enabled(false);
    
    app.add_action(&unlock_action);
    app.add_action(&confirm_action);
    app.add_action(&discard_action);
	
	// ---------- entries ----------
	
	let entries = [
		&state.ui.widgets().window.preferences.media.player_entry,
		&state.ui.widgets().window.preferences.media.flag_entry,
		&state.ui.widgets().window.preferences.media.lookup_entry,
		&state.ui.widgets().window.preferences.media.bind_entry,
	];
	
	for entry in entries {
		
		// prevent movement (Up Arrow)
		// prevent movement (Down Arrow)
        entry.connect_key_press_event({
            move |_, eventkey| {
                match eventkey.keyval() {
					gdk::keys::constants::Up => Inhibit(true),
					gdk::keys::constants::Down => Inhibit(true),
					_ => Inhibit(false),
				}
            }
        });
		
	}
	
	// focus global search entry (SHIFT + Tab)
	state.ui.widgets().window.preferences.media.player_entry.connect_key_press_event({
		let sender_cloned = sender.clone();
		move |_, eventkey| {
			if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
				sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
				return Inhibit(true);
			}
			Inhibit(false)
		}
	});
	
	// ---------- switches ----------
	
	let switches = [
		&state.ui.widgets().window.preferences.media.iconify_switch,
		&state.ui.widgets().window.preferences.media.autoselect_switch,
	];
	
	for switch in switches {
		
		// prevent movement (Up Arrow)
		// prevent movement (Down Arrow)
		// prevent movement (Left Arrow)
        switch.connect_key_press_event({
            move |_, eventkey| {
                match eventkey.keyval() {
					gdk::keys::constants::Up => Inhibit(true),
					gdk::keys::constants::Down => Inhibit(true),
					gdk::keys::constants::Left => Inhibit(true),
					_ => Inhibit(false),
				}
            }
        });
		
	}
	
	// ---------- buttons ----------
    
    for button in &state.ui.widgets().window.preferences.media.buttons_box.children() {
        
        // prevent selection of last media field (Up Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Up {
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
	
	if let Some(button) = state.ui.widgets().window.preferences.media.buttons_box.children().first() {
        
        // prevent selection of first media field (Left Arrow)
        button.connect_key_press_event({
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::Left {
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
	
	if let Some(button) = state.ui.widgets().window.preferences.media.buttons_box.children().iter().find(|button| button.is_sensitive()) {
        
        // focus global search entry (SHIFT + Tab)
        button.connect_key_press_event({
			let sender_cloned = sender.clone();
            move |_, eventkey| {
                if eventkey.keyval() == gdk::keys::constants::ISO_Left_Tab {
					sender_cloned.send(Message::General(GeneralActions::SearchFocus)).unwrap();
                    return Inhibit(true);
                }
                Inhibit(false)
            }
        });
        
    }
}

pub fn unlock(state: &State) {
    sensitivize_fields_and_buttons(state, true);
}

pub fn confirm(state: &mut State, sender: &Sender<Message>) {
    let mut success = true;
    
    success &= commit_player(state);
    success &= commit_iconify(state);
    success &= commit_flag(state, sender);
    success &= commit_timeout(state);
    success &= commit_autoselect(state);
    success &= commit_lookup(state);
    success &= commit_bind(state);
    
    if success {
        sensitivize_fields_and_buttons(state, false);
    }
}

pub fn discard(state: &State) {
    state.ui.widgets().window.preferences.media.player_entry.set_text(state.params.media_player(false));
    state.ui.widgets().window.preferences.media.iconify_switch.set_state(state.params.media_iconify(false));
    state.ui.widgets().window.preferences.media.flag_entry.set_text(state.params.media_flag(false));
    state.ui.widgets().window.preferences.media.timeout_spin.set_value(state.params.media_timeout(false).as_secs_f64());
    state.ui.widgets().window.preferences.media.autoselect_switch.set_state(state.params.media_autoselect(false));
    state.ui.widgets().window.preferences.media.lookup_entry.set_text(state.params.media_lookup(false));
    state.ui.widgets().window.preferences.media.bind_entry.set_text(state.params.media_bind(false));
    
    sensitivize_fields_and_buttons(state, false);
}

fn sensitivize_fields_and_buttons(state: &State, sensitive: bool) {
    state.ui.widgets().window.preferences.media.player_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.iconify_switch.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.flag_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.timeout_spin.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.autoselect_switch.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.lookup_entry.set_sensitive(sensitive);
    state.ui.widgets().window.preferences.media.bind_entry.set_sensitive(sensitive);
    
    if let Some(application) = state.ui.widgets().window.general.window.application() {
        
        // unlock action is turned sensitive when confirm and discard aren't and vice versa
        if let Some(unlock_action) = application.lookup_action("preferences.media.unlock") {
            if let Some(unlock_action) = unlock_action.downcast_ref::<gio::SimpleAction>() {
                unlock_action.set_enabled(! sensitive);
            }
        }
        
        if let Some(confirm_action) = application.lookup_action("preferences.media.confirm") {
            if let Some(confirm_action) = confirm_action.downcast_ref::<gio::SimpleAction>() {
                confirm_action.set_enabled(sensitive);
            }
        }
        
        if let Some(discard_action) = application.lookup_action("preferences.media.discard") {
            if let Some(discard_action) = discard_action.downcast_ref::<gio::SimpleAction>() {
                discard_action.set_enabled(sensitive);
            }
        }
        
    }
    
    let children = state.ui.widgets().window.preferences.media.buttons_box.children();
    
    if let Some(child) = children.iter().find(|child| child.is_sensitive()) {
        child.grab_focus();
    }
    
}

fn commit_player(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.media.player_entry.text();
    
    if let Err(error) = state.params.media_set_player(&input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}

fn commit_iconify(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.media.iconify_switch.state();
    
    state.params.media_set_iconify(input);
    
    true
}

fn commit_flag(state: &mut State, sender: &Sender<Message>) -> bool {
    let input = state.ui.widgets().window.preferences.media.flag_entry.text();
    
    match state.params.media_set_flag(&input) {
        
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

fn commit_timeout(state: &mut State) -> bool {
    let input = Duration::from_secs(u64::try_from(state.ui.widgets().window.preferences.media.timeout_spin.value_as_int()).unwrap_or(0));
    
    if let Err(error) = state.params.media_set_timeout(input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}

fn commit_autoselect(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.media.autoselect_switch.state();
    
    state.params.media_set_autoselect(input);
    
    true
}

fn commit_lookup(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.media.lookup_entry.text();
    
    if let Err(error) = state.params.media_set_lookup(&input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}

fn commit_bind(state: &mut State) -> bool {
    let input = state.ui.widgets().window.preferences.media.bind_entry.text();
    
    if let Err(error) = state.params.media_set_bind(&input) {
        state.ui.dialogs_error_show(&error.to_string());
        return false;
    }
    
    true
}
