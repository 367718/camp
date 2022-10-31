#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;
mod general;
mod preferences;
mod files;
mod watchlist;

use std::{
    env,
    error::Error,
    ffi::c_void,
    io::{ self, Write },
    path::{ Path, PathBuf },
    process,
    ptr,
};

use gtk::{
    gio,
    glib,
    prelude::*,
};

use rin::{ Params, Args, Config };
use ena::{ Files, FilesEntry, FilesMark, FilesWatcherEvent };
use chiaki::{
    Database,
    FeedsId, FeedsEntry,
    FormatsId, FormatsEntry,
    KindsId, KindsEntry,
    SeriesId, SeriesEntry, SeriesStatus, SeriesGood,
    CandidatesId, CandidatesEntry, CandidatesCurrent,
};
use aoi::RemoteControlServer;
use akari::Client as HttpClient;

use ui::Ui;
use general::GeneralActions;
use preferences::{ PreferencesActions, PreferencesSection };
use files::{ FilesActions, FilesSection };
use watchlist::{ WatchlistActions, WatchlistSection };

const APP_ID: &str = concat!("app.", env!("CARGO_PKG_NAME"));
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const APP_ICON: &[u8] = include_bytes!("../rsc/app.png");
const STYLESHEET: &[u8] = include_bytes!("../rsc/stylesheet.css");

pub struct State {
    ui: Ui,
    params: Params,
    database: Database,
    files: Files,
}

pub enum Message {
    General(GeneralActions),
    Preferences(PreferencesActions),
    Files(FilesActions),
    Watchlist(WatchlistActions),
}

fn main() {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    
    app.connect_activate(init_app);
    
    // disable gtk's command-line arguments handling
    app.run_with_args(&[] as &[&str]);
}

fn init_app(app: &gtk::Application) {
    // ---------- command-line arguments ----------
    
    let args = Args::new(
        None, // use values provided when the application was called
        &["--help", "--version", "--dlonly"], // additional flags
        &["--config", "--stylesheet"], // additional key-value pairs
    );
    
    if args.free_flag("--help") {
        show_help();
        process::exit(0);
    }
    
    if args.free_flag("--version") {
        println!("{} {}", APP_NAME, APP_VERSION);
        process::exit(0);
    }
    
    // ---------- ui ----------
    
    let ui = Ui::new(args.free_value("--stylesheet"));
    
    // ---------- enforce process uniqueness ----------
    
    if let Err(error) = register_app() {
        ui.dialogs_error_show(&error.to_string());
        process::exit(1);
    }
    
    // ---------- config ----------
    
    let config = init_config(args.free_value("--config"), &ui)
        .unwrap_or_else(|| process::exit(1));
    
    // ---------- parameters ----------
    
    let mut params = Params::new(args, config);
    
    // ---------- database ----------
    
    let database = init_database(&mut params, &ui)
        .unwrap_or_else(|| process::exit(1));
    
    // ---------- files ----------
    
    let files = if params.args_free_flag("--dlonly") {
        Files::new(
            Path::new(""),
            params.media_flag(true),
            &[] as &[&str],
        )
    } else {
        Files::new(
            params.paths_files(true),
            params.media_flag(true),
            &database.formats_iter()
                .map(|(_, entry)| entry.name.as_str())
                .collect::<Vec<&str>>(),
        )
    };
    
    // ---------- state ----------
    
    let mut state = State { ui, params, database, files };
    
    // ---------- channel ----------
    
    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    
    // ---------- modules ----------
    
    if state.params.args_free_flag("--dlonly") {
        files::handle_action(&mut state, &sender, FilesActions::Download);
        // SaveAndQuit must be called indirectly since DownloadedUpdate action wouldn't be handled otherwise
        sender.send(Message::General(GeneralActions::SaveAndQuit)).unwrap();
    } else {
        general::init(app, &state, &sender);
        preferences::init(app, &mut state, &sender);
        files::init(app, &mut state, &sender);
        watchlist::init(app, &state, &sender);
    }
    
    // ---------- actions ----------
    
    receiver.attach(None, move |message| {
        use Message::*;
        
        match message {
            General(action) => general::handle_action(&mut state, &sender, action),
            Preferences(action) => preferences::handle_action(&mut state, &sender, action),
            Files(action) => files::handle_action(&mut state, &sender, action),
            Watchlist(action) => watchlist::handle_action(&mut state, &sender, action),
        }
        
        glib::Continue(true)
    });
}

fn show_help() {
    let mut stdout = io::stdout().lock();
    
    writeln!(&mut stdout, "{}", APP_DESCRIPTION).unwrap();
    writeln!(&mut stdout).unwrap();
    
    writeln!(&mut stdout, "Available command-line arguments:").unwrap();
    writeln!(&mut stdout).unwrap();
    
    writeln!(&mut stdout, "--help        Prints available command-line arguments and exits").unwrap();
    writeln!(&mut stdout, "--version     Prints version information and exits").unwrap();
    writeln!(&mut stdout, "--dlonly      Downloads new releases and exits").unwrap();
    writeln!(&mut stdout).unwrap();
    
    writeln!(&mut stdout, "--maximized   Sets whether the application window should start in a maximized state (yes/no)").unwrap();
    writeln!(&mut stdout, "--width       Sets application window width (number)").unwrap();
    writeln!(&mut stdout, "--height      Sets application window height (number)").unwrap();
    writeln!(&mut stdout, "--x           Sets application window position in the horizontal axis (number)").unwrap();
    writeln!(&mut stdout, "--y           Sets application window position in the vertical axis (number)").unwrap();
    writeln!(&mut stdout).unwrap();
    
    writeln!(&mut stdout, "--player      Sets the command to run for playing files").unwrap();
    writeln!(&mut stdout, "--iconify     Sets whether the application should iconify when a file is played (yes/no)").unwrap();
    writeln!(&mut stdout, "--flag        Sets the flag to use for marking files").unwrap();
    writeln!(&mut stdout, "--timeout     Sets the timeout (in seconds) for download requests").unwrap();
    writeln!(&mut stdout, "--autoselect  Sets whether series should be autoselected when modified (yes/no)").unwrap();
    writeln!(&mut stdout, "--lookup      Sets the URL of the lookup site ('%s' will be replaced by query)").unwrap();
    writeln!(&mut stdout, "--bind        Sets the address to bind for remote control").unwrap();
    writeln!(&mut stdout).unwrap();
    
    writeln!(&mut stdout, "--config      Sets the path of the configuration file").unwrap();
    writeln!(&mut stdout, "--stylesheet  Sets the path of the stylesheet file").unwrap();
    writeln!(&mut stdout, "--files       Sets the path of the files directory").unwrap();
    writeln!(&mut stdout, "--downloads   Sets the path of the directory to download torrent files in").unwrap();
    writeln!(&mut stdout, "--pipe        Sets the path of the named pipe to use for remote control").unwrap();
    writeln!(&mut stdout, "--database    Sets the path of the database file").unwrap();
}

fn register_app() -> Result<(), Box<dyn Error>> {
    
    extern "system" {
        
        fn CreateMutexW(
            lpMutexAttributes: *mut c_void, // LPSECURITY_ATTRIBUTES -> *mut SECURITY_ATTRIBUTES
            bInitialOwner: i32, // BOOL
            lpName: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
        ) -> *mut c_void; // HANDLE
        
    }
    
    let name: Vec<u16> = APP_ID.encode_utf16()
        .chain(Some(0))
        .collect();
    
    unsafe {
        
        // mutex will be automatically released on application shutdown
        CreateMutexW(
            ptr::null_mut(),
            0,
            name.as_ptr(),
        )
        
    };
    
    // allow app to run even if mutex could not be created, but stop if it already exists
    if io::Error::last_os_error().kind() == io::ErrorKind::AlreadyExists {
        return Err("Only one instance of the application can be running at one time".into());
    }
    
    Ok(())
    
}

fn init_config(path: Option<&str>, ui: &Ui) -> Option<Config> {
    let cfgpath = path.map_or_else(||
        env::current_exe().unwrap().with_extension("cfg"),
        PathBuf::from
    );
    
    // success
    
    let mut message = match Config::load(&cfgpath) {
        Ok(data) => return Some(data),
        Err(error) => error.to_string(),
    };
    
    // error
    
    ui.widgets().dialogs.general.config_load_error.path_label.set_text(&cfgpath.to_string_lossy());
    
    let config_dialog = &ui.widgets().dialogs.general.config_load_error.dialog;
    
    loop {
        
        ui.widgets().dialogs.general.config_load_error.message_label.set_text(&message);
        
        let response = config_dialog.run();
        
        config_dialog.unrealize();
        config_dialog.hide();
        
        match response {
            
            // generate new
            
            gtk::ResponseType::Ok => {
                
                let result = Config::new(&cfgpath).and_then(|mut config| {
                    // update database path
                    config.paths_set_database(cfgpath.with_extension("db"))?;
                    Ok(config)
                });
                
                match result {
                    Ok(config) => return Some(config),
                    Err(error) => message = error.to_string(),
                }
                
            },
            
            // exit
            _ => return None,
            
        }
        
    }
}

fn init_database(params: &mut Params, ui: &Ui) -> Option<Database> {
    let mut dbpath = params.paths_database(true).to_owned();
    
    // success
    
    let mut message = match Database::load(&dbpath) {
        Ok(database) => return Some(database),
        Err(error) => error.to_string(),
    };
    
    // error
    
    let save_chooser = &ui.widgets().dialogs.general.chooser.dialog;
    
    save_chooser.set_title("Choose database path");
    save_chooser.set_action(gtk::FileChooserAction::Save);
    
    let database_dialog = &ui.widgets().dialogs.general.database_load_error.dialog;
    
    loop {
        
        ui.widgets().dialogs.general.database_load_error.path_label.set_text(&dbpath.to_string_lossy());
        ui.widgets().dialogs.general.database_load_error.message_label.set_text(&message);
        
        let response = database_dialog.run();
        
        database_dialog.unrealize();
        database_dialog.hide();
        
        match response {
            
            // generate new
            
            gtk::ResponseType::Other(0) => match Database::new(&dbpath) {
                Ok(database) => return Some(database),
                Err(error) => message = error.to_string(),
            },
            
            // select another
            
            gtk::ResponseType::Other(1) => 'inner: loop {
                
                let response = save_chooser.run();
                
                save_chooser.hide();
                
                match response {
                    
                    // confirm
                    
                    gtk::ResponseType::Accept => if let Some(chosen) = save_chooser.filename() {
                        
                        dbpath = chosen;
                        
                        let result = Database::load(&dbpath).and_then(|database| {
                            if params.args_paths_database().is_none() {
                                params.paths_set_database(&dbpath)?;
                            }
                            Ok(database)
                        });
                        
                        match result {
                            Ok(database) => return Some(database),
                            Err(error) => message = error.to_string(),
                        }
                        
                        break 'inner;
                        
                    },
                    
                    // cancel
                    
                    _ => break 'inner,
                    
                }
                
            },
            
            // exit
            
            _ => return None,
            
        }
        
    }
}

#[cfg(test)]
mod bin {
    
    use super::*;
    
    #[cfg(test)]
    mod sections {
        
        use super::*;
        
        #[test]
        fn files_count() {
            // setup
            
            let count = FilesSection::iter().count();
            
            // operation
            
            let output = count == 2;
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn watchlist_count() {
            // setup
            
            let count = WatchlistSection::iter().count();
            
            // operation
            
            let output = count == 4;
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn preferences_count() {
            // setup
            
            let count = PreferencesSection::iter().count();
            
            // operation
            
            let output = count == 6;
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn files_display_unique() {
            // setup
            
            let count = FilesSection::iter().count();
            let mut collected = Vec::new();
            
            for display in FilesSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&display) {
                    collected.push(display);
                }
            }
            
            // operation
            
            let output = count == collected.len();
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn watchlist_display_unique() {
            // setup
            
            let count = WatchlistSection::iter().count();
            let mut collected = Vec::new();
            
            for display in WatchlistSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&display) {
                    collected.push(display);
                }
            }
            
            // operation
            
            let output = count == collected.len();
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn preference_display_unique() {
            // setup
            
            let count = PreferencesSection::iter().count();
            let mut collected = Vec::new();
            
            for section in PreferencesSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&section) {
                    collected.push(section);
                }
            }
            
            // operation
            
            let output = count == collected.len();
            
            // control
            
            assert!(output);
        }
        
        #[test]
        fn display_unique() {
            // setup
            
            let count = FilesSection::iter().count() + WatchlistSection::iter().count() + PreferencesSection::iter().count();
            let mut collected = Vec::new();
            
            for display in FilesSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&display) {
                    collected.push(display);
                }
            }
            
            for display in WatchlistSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&display) {
                    collected.push(display);
                }
            }
            
            for display in PreferencesSection::iter().map(|section| section.display().to_string()) {
                if ! collected.contains(&display) {
                    collected.push(display);
                }
            }
            
            // operation
            
            let output = count == collected.len();
            
            // control
            
            assert!(output);
        }
        
    }
    
    #[cfg(test)]
    mod file_marks {
        
        use super::*;
        
        #[test]
        fn count() {
            // setup
            
            let count = FilesMark::iter().count();
            
            // operation
            
            let output = count == 3;
            
            // control
            
            assert!(output);
        }
        
    }
    
}
