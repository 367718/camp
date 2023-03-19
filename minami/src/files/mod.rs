mod general;
mod file;
mod edit;
mod view;
mod tools;

use std::{
    iter,
    path::PathBuf,
};

use gtk::glib::Sender;

use crate::{
    State, Message,
    SeriesId,
};

pub enum FilesActions {
    
    // ---------- general ----------
    
    Add(PathBuf),
    Remove(PathBuf),
    Reload,
    ShowFrame(bool),
    
    // ---------- file ----------
    
    Play,
    Rename,
    MoveToFolder,
    Delete,
    MarkAsWatched,
    MarkAsUpdated(Vec<(SeriesId, u32, PathBuf)>),
    RefreshQueue,
    Maintenance,
    OpenDirectory,
    
    // ---------- edit ----------
    
    AddCandidate,
    AddSeries,
    CopyNames,
    
    // ---------- tools ----------
    
    Lookup,
    Remote,
    Download,
    Update,
    
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilesSection {
    New,
    Watched,
}

impl FilesSection {
    
    pub fn display(&self) -> &str {
        match self {
            Self::New => "New",
            Self::Watched => "Watched",
        }
    }
    
    pub fn iter() -> impl iter::Iterator<Item = Self> {
        [
            Self::New,
            Self::Watched,
        ].iter().copied()
    }
    
}

impl From<bool> for FilesSection {
    
    fn from(value: bool) -> Self {
        if value {
            Self::Watched
        } else {
            Self::New
        }
    }
    
}

pub fn init(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    general::init(app, state, sender);
    file::init(app, state, sender);
    edit::init(app, state, sender);
    view::init(state, sender);
    tools::init(app, state, sender);
}

pub fn handle_action(state: &mut State, sender: &Sender<Message>, action: FilesActions) {
    use FilesActions::*;
    
    match action {
        
        // ---------- general ----------
        
        // files -> general -> mount_watcher
        Add(path) => general::add(state, sender, &path),
        
        // files -> general -> mount_watcher
        Remove(path) => general::remove(state, sender, &path),
        
        // files -> general -> bind
        // preferences -> formats -> add
        // preferences -> formats -> edit
        // preferences -> formats -> delete
        // preferences -> formats -> reload
        // preferences -> media -> commit_flag
        // preferences -> media -> commit_maxdepth
        // preferences -> paths -> commit_files
        Reload => general::reload(state, sender),
        
        // files -> general -> mount_watcher
        ShowFrame(show) => general::show_frame(state, show),
        
        // ---------- file ----------
        
        // files -> file -> bind x2
        Play => file::play(state),
        
        // files -> file -> bind x2
        Rename => file::rename(state),
        
        // files -> file -> bind x2
        MoveToFolder => file::move_to_folder(state),
        
        // files -> file -> bind x2
        Delete => file::delete(state),
        
        // files -> file -> bind x3
        MarkAsWatched => file::mark_as_watched(state),
        
        // files -> tools -> update
        MarkAsUpdated(updates) => file::mark_as_updated(state, updates),
        
        // files -> file -> bind
        RefreshQueue => file::refresh_queue(state),
        
        // files -> file -> bind
        Maintenance => file::maintenance(state),
        
        // files -> file -> bind
        OpenDirectory => file::open_directory(state),
        
        // ---------- edit ----------
        
        // files -> edit -> bind x2
        CopyNames => edit::copy_names(state),
        
        // files -> edit -> bind x2
        AddCandidate => edit::add_candidate(state, sender),
        
        // files -> edit -> bind x2
        AddSeries => edit::add_series(state, sender),
        
        // ---------- tools ----------
        
        // files -> tools -> bind x2
        Lookup => tools::lookup(state),
        
        // files -> tools -> bind
        Remote => tools::remote(state),
        
        // files -> tools -> bind
        Download => tools::download(state, sender),
        
        // files -> tools -> bind
        Update => tools::update(state, sender),
        
    }
}
