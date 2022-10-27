mod general;
mod edit;
mod tools;

use gtk::glib::Sender;

use crate::{
    State, Message,
    SeriesStatus,
};

use edit::ProgressModification;

pub enum WatchlistActions {
    
    // ---------- general ----------
    
    MenuPopup(Option<(f64, f64)>),
    Reload,
    
    // ---------- edit ----------
    
    Add(Option<String>),
    Edit(bool),
    Delete,
    ChangeProgress(ProgressModification),
    CopyTitles,
    
    // ---------- tools ----------
    
    Lookup,
    
}

pub type WatchlistSection = SeriesStatus;

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    general::init(app, state, sender);
    edit::init(app, state, sender);
    tools::init(app, state, sender);
}

pub fn handle_action(state: &mut State, sender: &Sender<Message>, action: WatchlistActions) {
    use WatchlistActions::*;
    
    match action {
        
        // ---------- general ----------
        
        // watchlist -> general -> bind x2
        MenuPopup(coords) => general::menu_popup(state, coords),
        
        // preferences -> kinds -> edit
        // preferences -> kinds -> reload
        Reload => general::reload(state, sender),
        
        // ---------- edit ----------
        
        // files -> edit -> add_series
        // watchlist -> edit -> bind x2
        Add(prefill) => edit::add(state, sender, &prefill),
        
        // watchlist -> edit -> bind x6
        Edit(completed) => edit::edit(state, sender, completed),
        
        // watchlist -> edit -> bind x2
        Delete => edit::delete(state, sender),
        
        // watchlist -> edit -> bind x4
        ChangeProgress(modification) => edit::change_progress(state, &modification),
        
        // watchlist -> edit -> bind x2
        CopyTitles => edit::copy_titles(state),
        
        // ---------- tools ----------
        
        // watchlist -> tools -> bind x2
        Lookup => tools::lookup(state),
        
    }
}
