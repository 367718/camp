mod general;
mod edit;
mod tools;

use gtk::glib::Sender;

use crate::{
    State, Message,
    SeriesStatus,
};

pub enum WatchlistActions {
    
    // ---------- general ----------
    
    Reload,
    
    // ---------- edit ----------
    
    Add(Option<String>),
    Edit,
    Delete,
    CopyTitles,
    
    // ---------- tools ----------
    
    Lookup,
    
}

pub type WatchlistSection = SeriesStatus;

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    general::init(state);
    edit::init(app, state, sender);
    tools::init(app, state, sender);
}

pub fn handle_action(state: &mut State, sender: &Sender<Message>, action: WatchlistActions) {
    use WatchlistActions::*;
    
    match action {
        
        // ---------- general ----------
        
        // preferences -> kinds -> edit
        // preferences -> kinds -> reload
        Reload => general::reload(state, sender),
        
        // ---------- edit ----------
        
        // files -> edit -> add_series
        // watchlist -> edit -> bind x2
        Add(prefill) => edit::add(state, sender, &prefill),
        
        // watchlist -> edit -> bind x3
        Edit => edit::edit(state, sender),
        
        // watchlist -> edit -> bind x2
        Delete => edit::delete(state, sender),
        
        // watchlist -> edit -> bind x2
        CopyTitles => edit::copy_titles(state),
        
        // ---------- tools ----------
        
        // watchlist -> tools -> bind x2
        Lookup => tools::lookup(state),
        
    }
}
