mod candidates;
mod feeds;
mod formats;
mod kinds;
mod media;
mod paths;

use std::iter;

use gtk::glib::Sender;

use crate::{
    State, Message,
    SeriesId,
};

pub enum PreferencesActions {
    
    // ---------- candidates ----------
    
    CandidatesAdd(Option<String>),
    CandidatesEdit,
    CandidatesDelete,
    CandidatesShowInfo,
    CandidatesReload,
    DownloadedAdd,
    DownloadedEdit,
    DownloadedDelete,
    DownloadedUpdate(Vec<(SeriesId, u32)>),
    
    // ---------- feeds ----------
    
    FeedsAdd,
    FeedsEdit,
    FeedsDelete,
    FeedsReload,
    
    // ---------- formats ----------
    
    FormatsAdd,
    FormatsEdit,
    FormatsDelete,
    FormatsReload,
    
    // ---------- kinds ----------
    
    KindsAdd,
    KindsEdit,
    KindsDelete,
    KindsReload,
    
    // ---------- media ----------
    
    MediaUnlock,
    MediaConfirm,    
    MediaDiscard,
    
    // ---------- paths ----------
    
    PathsChooseFiles,
    PathsChooseDownloads,
    PathsChooseDatabase,
    PathsUnlock,
    PathsConfirm,
    PathsDiscard,
    
}

#[derive(Clone, Copy)]
pub enum PreferencesSection {
    Candidates,
    Feeds,
    Kinds,
    Formats,
    Media,
    Paths,
}

impl PreferencesSection {
    
    pub fn display(&self) -> &str {
        match self {
            Self::Candidates => "Candidates",
            Self::Feeds => "Feeds",
            Self::Kinds => "Kinds",
            Self::Formats => "Formats",
            Self::Media => "Media",
            Self::Paths => "Paths",
        }
    }
    
    pub fn iter() -> impl iter::Iterator<Item = Self> {
        [
            Self::Candidates,
            Self::Feeds,
            Self::Kinds,
            Self::Formats,
            Self::Media,
            Self::Paths,
        ].iter().copied()
    }
    
}

pub fn init(app: &gtk::Application, state: &mut State, sender: &Sender<Message>) {
    candidates::init(app, state, sender);
    feeds::init(app, state, sender);
    formats::init(app, state, sender);
    kinds::init(app, state, sender);
    media::init(app, state, sender);
    paths::init(app, state, sender);
}

pub fn handle_action(state: &mut State, sender: &Sender<Message>, action: PreferencesActions) {
    use PreferencesActions::*;
    
    match action {
        
        // ---------- candidates ----------
        
        // files -> edit -> add_candidate
        // preferences -> candidates -> bind x2
        CandidatesAdd(prefill) => candidates::candidates_add(state, sender, &prefill),
        
        // preferences -> candidates -> bind x3
        CandidatesEdit => candidates::candidates_edit(state, sender),
        
        // preferences -> candidates -> bind x2
        CandidatesDelete => candidates::candidates_delete(state),
        
        // preferences -> candidates -> bind
        // preferences -> candidates -> downloaded_update
        CandidatesShowInfo => candidates::show_info(state),
        
        // general -> reload_database
        CandidatesReload => candidates::reload(state),
        
        // preferences -> candidates -> bind x2
        DownloadedAdd => candidates::downloaded_add(state),
        
        // preferences -> candidates -> bind x2
        DownloadedEdit => candidates::downloaded_edit(state),
        
        // preferences -> candidates -> bind x2
        DownloadedDelete => candidates::downloaded_delete(state),
        
        // files -> tools -> download
        DownloadedUpdate(downloads) => candidates::downloaded_update(state, sender, downloads),
        
        // ---------- feeds ----------
        
        // preferences -> feeds -> bind x2
        FeedsAdd => feeds::add(state),
        
        // preferences -> feeds -> bind x3
        FeedsEdit => feeds::edit(state),
        
        // preferences -> feeds -> bind x2
        FeedsDelete => feeds::delete(state),
        
        // general -> reload_database
        FeedsReload => feeds::reload(state),
        
        // ---------- formats ----------
        
        // preferences -> formats -> bind x2
        FormatsAdd => formats::add(state, sender),
        
        // preferences -> formats -> bind x3
        FormatsEdit => formats::edit(state, sender),
        
        // preferences -> formats -> bind x2
        FormatsDelete => formats::delete(state, sender),
        
        // general -> reload_database
        FormatsReload => formats::reload(state, sender),
        
        // ---------- kinds ----------
        
        // preferences -> kinds -> bind x2
        KindsAdd => kinds::add(state),
        
        // preferences -> kinds -> bind x3
        KindsEdit => kinds::edit(state, sender),
        
        // preferences -> kinds -> bind x2
        KindsDelete => kinds::delete(state),
        
        // general -> reload_database
        KindsReload => kinds::reload(state, sender),
        
        // ---------- media ----------
        
        // preferences -> media -> bind
        MediaUnlock => media::unlock(state),
        
        // preferences -> media -> bind
        MediaConfirm => media::confirm(state, sender),
        
        // preferences -> media -> bind
        MediaDiscard => media::discard(state),
        
        // ---------- paths ----------
        
        // preferences -> paths -> bind
        PathsChooseFiles => paths::choose_files(state),
        
        // preferences -> paths -> bind
        PathsChooseDownloads => paths::choose_downloads(state),
        
        // preferences -> paths -> bind
        PathsChooseDatabase => paths::choose_database(state),
        
        // preferences -> paths -> bind
        PathsUnlock => paths::unlock(state),
        
        // preferences -> paths -> bind
        PathsConfirm => paths::confirm(state, sender),
        
        // preferences -> paths -> bind
        PathsDiscard => paths::discard(state),
        
    }
}
