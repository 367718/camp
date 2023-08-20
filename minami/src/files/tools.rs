use std::{
    error::Error,
    ffi::OsString,
    fs,
    path::{ MAIN_SEPARATOR_STR, Path },
    str,
    thread,
};

use gtk::{
    gdk,
    gio,
    glib::{ self, Sender },
    prelude::*,
};

use crate::{
    State, Message,
    PreferencesActions, FilesActions,
    FeedsId, FeedsEntry,
    CandidatesEntry, CandidatesCurrent,
    FilesMark,
    DownloadsEntries, DownloadsEntry, UpdatesEntries, UpdatesEntry,
    RemoteControlServer,
    HttpClient,
};

pub fn init(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    build(state);
    bind(app, state, sender);
}

fn build(state: &State) {
    let job_dialog = &state.ui.widgets().dialogs.files.job.dialog;
    
    // override dlonly mode setup
    
    job_dialog.set_position(gtk::WindowPosition::CenterOnParent);
    job_dialog.set_transient_for(Some(&state.ui.widgets().window.general.window));
}

fn bind(app: &gtk::Application, state: &State, sender: &Sender<Message>) {
    // ---------- actions ----------
    
    let lookup_action = gio::SimpleAction::new("files.tools.lookup", None);
    let remote_action = gio::SimpleAction::new("files.tools.remote", None);
    let download_action = gio::SimpleAction::new("files.tools.download", None);
    let update_action = gio::SimpleAction::new("files.tools.update", None);
    
    // lookup selected name
    lookup_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Lookup)).unwrap()
    });
    
    // start remote control
    remote_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Remote)).unwrap()
    });
    
    // download new releases
    download_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Download)).unwrap()
    });
    
    // update watched releases
    update_action.connect_activate({
        let sender_cloned = sender.clone();
        move |_, _| sender_cloned.send(Message::Files(FilesActions::Update)).unwrap()
    });
    
    app.add_action(&lookup_action);
    app.add_action(&remote_action);
    app.add_action(&download_action);
    app.add_action(&update_action);
    
    // ---------- treeviews ----------
    
    for treeview in &state.ui.widgets().window.files.treeviews {
        
        // lookup selected name (CONTROL + L/l)
        treeview.connect_key_press_event({
            let sender_cloned = sender.clone();
            move |_, eventkey| {
                match eventkey.keyval() {
                    
                    key if (key == gdk::keys::constants::L || key == gdk::keys::constants::l) && eventkey.state().contains(gdk::ModifierType::CONTROL_MASK) => {
                        sender_cloned.send(Message::Files(FilesActions::Lookup)).unwrap();
                        glib::Propagation::Stop
                    },
                    
                    _ => glib::Propagation::Proceed,
                    
                }
            }
        });
        
    }
}

pub fn lookup(state: &State) {
    let Some(treeview) = state.ui.files_current_treeview() else {
        return;
    };
    
    let (treepaths, treemodel) = treeview.selection().selected_rows();
    
    if treepaths.is_empty() {
        return;
    }
    
    if treepaths.len() > 1 {
        treeview.set_cursor(treepaths.first().unwrap(), None::<&gtk::TreeViewColumn>, false);
    }
    
    let treeiter = treemodel.iter(treepaths.first().unwrap()).unwrap();
    
    let name = match treemodel.iter_parent(&treeiter) {
        
        Some(parent_iter) => {
            
            let container = treemodel.value(&parent_iter, 3).get::<glib::GString>().unwrap();
            let file_stem = treemodel.value(&treeiter, 3).get::<glib::GString>().unwrap();
            
            chikuwa::concat_str!(&container, MAIN_SEPARATOR_STR, &file_stem)
            
        },
        
        None => treemodel.value(&treeiter, 3).get::<String>().unwrap()
        
    };
    
    let lookup = state.params.media_lookup(true);
    let url = lookup.replace("%s", &chikuwa::percent_encode(&name));
    
    if let Err(error) = chikuwa::execute_app(&url) {
        state.ui.dialogs_error_show(&error.to_string());
    }
}

pub fn remote(state: &mut State) {
    // ---------- parameters ----------
    
    let pipe = state.params.paths_pipe(true);
    let bind = state.params.media_bind(true);
    
    // ---------- dialog ----------
    
    let job_dialog = state.ui.widgets().dialogs.files.job.dialog.clone();
    job_dialog.set_title("Remote control");
    
    let progress_buffer = state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap();
    progress_buffer.set_text("");
    
    // ---------- startup ----------
    
    progress_buffer.insert(&mut progress_buffer.end_iter(), &chikuwa::concat_str!("Pipe: ", &pipe.to_string_lossy(), "\n"));
    progress_buffer.insert(&mut progress_buffer.end_iter(), &chikuwa::concat_str!("Bind: ", bind, "\n"));
    progress_buffer.insert(&mut progress_buffer.end_iter(), "\nConnecting to player instance and starting HTTP server...\n\n");
    
    // ---------- channel ----------
    
    let (job_sender, job_receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    
    // ---------- server ----------
    
    let mut server = None;
    
    match RemoteControlServer::start(pipe, bind, move |error| job_sender.send(error).unwrap()) {
        
        Ok(started) => {
            
            server = Some(started);
            
            progress_buffer.insert(&mut progress_buffer.end_iter(), "Success, listening for commands...\n\n");
            
            // ---------- error ----------
            
            // this can end up printing a message to the buffer after the dialog has been closed
            job_receiver.attach(None, move |error| {
                progress_buffer.insert(&mut progress_buffer.end_iter(), &chikuwa::concat_str!("ERROR: ", &error.to_string()));
                glib::ControlFlow::Continue
            });
            
        },
        
        Err(error) => progress_buffer.insert(&mut progress_buffer.end_iter(), &chikuwa::concat_str!("ERROR: ", &error.to_string())),
        
    }
    
    // ---------- dialog ----------
    
    job_dialog.run();
    job_dialog.unrealize();
    job_dialog.hide();
    
    // ---------- stop ----------
    
    if let Some(mut server) = server {
        server.stop();
    }
    
    // ---------- cleanup ----------
    
    state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap().set_text("");
}

pub fn download(state: &mut State, sender: &Sender<Message>) {
    
    fn get_torrent(client: &mut HttpClient, name: &str, url: &str, directory: &Path) -> Result<(), Box<dyn Error>> {
        let filename = Path::new(name).file_name().ok_or("Invalid file name")?;
        let mut destination = directory.join(filename);
        
        if let Some(current) = destination.extension() {
            if ! current.eq_ignore_ascii_case("torrent") {
                let mut composite = OsString::with_capacity(current.len() + 8);
                composite.push(current);
                composite.push(".torrent");
                destination.set_extension(composite);
            }
        } else {
            destination.set_extension("torrent");
        }
        
        fs::create_dir_all(directory)?;
        
        if destination.exists() {
            return Err(chikuwa::concat_str!("File already exists: ", &destination.to_string_lossy()).into());
        }
        
        let content = client.get(url)?;
        
        fs::write(destination, content)?;
        
        Ok(())
    }
    
    // ---------- parameters ----------
    
    let candidates = state.database.candidates_iter()
        .filter(|(_, entry)| entry.current() == CandidatesCurrent::Yes)
        .map(|(_, entry)| entry)
        .collect::<Vec<&CandidatesEntry>>();
    
    let mut feeds = state.database.feeds_iter()
        .collect::<Vec<(FeedsId, &FeedsEntry)>>();
    
    feeds.sort_unstable_by(|a, b| a.0.to_int().cmp(&b.0.to_int()));
    
    let feeds = feeds.drain(..)
        .map(|(_, entry)| entry)
        .collect::<Vec<&FeedsEntry>>();
    
    let timeout = state.params.media_timeout(true);
    let directory = state.params.paths_downloads(true);
    
    // ---------- dialog ----------
    
    let job_dialog = state.ui.widgets().dialogs.files.job.dialog.clone();
    job_dialog.set_title("Download new releases");
    job_dialog.set_response_sensitive(gtk::ResponseType::Close, false);
    
    let progress_buffer = state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap();
    progress_buffer.set_text("");
    
    // ---------- channel ----------
    
    let (job_sender, job_receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    
    // ---------- thread ----------
    
    thread::scope(|scope| {
        
        // ---------- downloads ----------
        
        let result = scope.spawn(|| {
            
            let mut result = Vec::with_capacity(25);
            let mut client = HttpClient::new(timeout);
            
            for feed in feeds {
                
                job_sender.send(Some(feed.url().to_string())).unwrap();
                job_sender.send(Some(String::from("\n------------------------------------------------------------\n"))).unwrap();
                
                match client.get(feed.url()) {
                    
                    Ok(content) => {
                        
                        let mut found_releases = false;
                        
                        let mut downloads: Vec<DownloadsEntry> = DownloadsEntries::get(&content, &candidates).collect();
                        downloads.sort_unstable_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
                        
                        result.reserve(downloads.len());
                        
                        for download in downloads {
                            
                            let current = (download.candidate.series(), download.episode);
                            
                            if ! result.contains(&current) {
                                
                                job_sender.send(Some(chikuwa::concat_str!(download.title, "\n"))).unwrap();
                                
                                found_releases = true;
                                
                                // commit to disk
                                if let Err(error) = get_torrent(&mut client, download.title, download.link, directory) {
                                    job_sender.send(Some(chikuwa::concat_str!("ERROR: ", &error.to_string(), "\n"))).unwrap();
                                    continue;
                                }
                                
                                result.push(current);
                                
                            }
                            
                        }
                        
                        if ! found_releases {
                            job_sender.send(Some(String::from("No releases found\n"))).unwrap();
                        }
                        
                    },
                    
                    Err(error) => job_sender.send(Some(chikuwa::concat_str!("ERROR: ", &error.to_string(), "\n"))).unwrap(),
                    
                }
                
                job_sender.send(Some(String::from("\n\n"))).unwrap();
                
            }
            
            job_sender.send(None).unwrap();
            
            result
            
        });
        
        // ---------- progress ----------
        
        job_receiver.attach(None, move |message| {
            match message {
                Some(message) => progress_buffer.insert(&mut progress_buffer.end_iter(), &message),
                None => job_dialog.set_response_sensitive(gtk::ResponseType::Close, true),
            }
            
            glib::ControlFlow::Continue
        });
        
        // ---------- dialog ----------
        
        let job_dialog = &state.ui.widgets().dialogs.files.job.dialog;
        
        job_dialog.run();
        job_dialog.unrealize();
        job_dialog.hide();
        
        // ---------- downloaded update ----------
        
        if let Ok(result) = result.join() {
            sender.send(Message::Preferences(PreferencesActions::DownloadedUpdate(result))).unwrap();
        }
        
    });
    
    // ---------- cleanup ----------
    
    state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap().set_text("");
    
}

pub fn update(state: &mut State, sender: &Sender<Message>) {
    // ---------- parameters ----------
    
    let candidates = state.database.candidates_iter()
        .map(|(_, entry)| entry)
        .collect::<Vec<&CandidatesEntry>>();
    
    let files = state.files.iter()
        .filter(|entry| entry.mark() == FilesMark::Watched)
        .filter_map(|entry| Some((entry.name().to_str()?, entry.path())))
        .collect::<Vec<(&str, &Path)>>();
    
    // ---------- dialog ----------
    
    let job_dialog = &state.ui.widgets().dialogs.files.job.dialog;
    job_dialog.set_title("Update watched releases");
    
    let progress_buffer = state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap();
    progress_buffer.set_text("");
    
    // ---------- updates and progress ----------
    
    let mut updates: Vec<UpdatesEntry> = UpdatesEntries::get(&files, &candidates).collect();
    updates.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    let mut result = Vec::with_capacity(updates.len());
    
    for update in updates {
        
        let episode = update.episode.saturating_sub(update.candidate.offset());
        
        if episode > 0 {
            progress_buffer.insert(&mut progress_buffer.end_iter(), &chikuwa::concat_str!(update.name, "\n"));
            let id = update.candidate.series();
            result.push((id, episode, update.path.to_owned()));
        }
        
    }
    
    if result.is_empty() {
        progress_buffer.insert(&mut progress_buffer.end_iter(), "No updates found");
    } else {
        sender.send(Message::Files(FilesActions::MarkAsUpdated(result))).unwrap();
    }
    
    // ---------- dialog ----------
    
    job_dialog.run();
    job_dialog.unrealize();
    job_dialog.hide();
    
    // ---------- cleanup ----------
    
    state.ui.widgets().dialogs.files.job.progress_textview.buffer().unwrap().set_text("");    
}
