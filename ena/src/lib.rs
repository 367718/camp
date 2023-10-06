mod marker;
mod walker;
mod watcher;

use std::{
    error::Error,
    ffi::{ OsStr, OsString },
    fs,
    mem,
    path::{ Path, PathBuf },
};

pub use marker::FilesMark;
pub use watcher::FilesWatcherEvent;
use walker::FilesWalker;
use watcher::FilesWatcher;

pub struct Files {
    root: Box<Path>,
    flag: Box<OsStr>,
    formats: Vec<Box<OsStr>>,
    entries: Vec<FilesEntry>,
    queue: Vec<usize>,
    watcher: Option<(FilesWatcher, Box<dyn Fn(FilesWatcherEvent)>)>,
}

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    path: Box<Path>,
    name: Box<OsStr>,
    container: Option<Box<OsStr>>,
    mark: FilesMark,
}

impl Files {
    
    // ---------- constructors ----------
    
    
    pub fn new<P: Into<PathBuf>, F: Into<OsString>, M: Into<OsString>>(root: P, flag: F, formats: impl Iterator<Item = M>) -> Self {
        let mut files = Files {
            root: root.into().into_boxed_path(),
            flag: flag.into().into_boxed_os_str(),
            formats: formats.map(Into::into).map(OsString::into_boxed_os_str).collect(),
            entries: Vec::new(),
            queue: Vec::new(),
            watcher: None,
        };
        
        files.entries = FilesWalker::new(files.root.to_path_buf())
            .filter_map(|path| files.build_entry(path))
            .collect();
        
        files
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&FilesEntry> {
        let path = path.as_ref();
        self.entries.iter().find(|entry| entry.path.as_ref() == path)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &FilesEntry> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    pub fn queue(&self) -> impl Iterator<Item = &FilesEntry> {
        self.queue.iter().filter_map(|&index| self.entries.get(index))
    }
    
    pub fn rename<P: AsRef<Path>, S: AsRef<OsStr>>(&self, path: P, name: S) -> Result<(), Box<dyn Error>> {
        let entry = self.get(&path).ok_or("Entry not found")?;
        
        let new_name = Path::new(&name).file_name().ok_or("Could not get new file name")?;
        let current_extension = entry.path.extension().ok_or("Could not get current file extension")?;
        
        let destination = entry.path.with_file_name(new_name).with_extension(current_extension);
        
        if entry.path.as_ref() != destination {
            
            if destination.exists() {
                return Err(chikuwa::concat_str!("File already exists: ", &destination.to_string_lossy()).into());
            }
            
            fs::rename(&entry.path, &destination)?;
            
        }
        
        Ok(())
    }
    
    pub fn move_to_folder<P: AsRef<Path>, S: AsRef<OsStr>>(&self, path: P, folder: Option<S>) -> Result<(), Box<dyn Error>> {
        let entry = self.get(&path).ok_or("Entry not found")?;
        
        let filename = entry.path.file_name().ok_or("Could not determine file name")?;
        
        let destination = match folder {
            
            // to subdirectory
            
            Some(folder) => {
                
                let folder_name = Path::new(&folder)
                    .file_name()
                    .ok_or("Could not determine folder name")?;
                
                let subdirectory = self.root.join(folder_name);
                
                if ! subdirectory.exists() {
                    fs::create_dir(&subdirectory)?;
                }
                
                subdirectory.join(filename)
                
            },
            
            // to root
            
            None => self.root.join(filename)
            
        };
        
        if entry.path.as_ref() != destination {
            
            if destination.exists() {
                return Err(chikuwa::concat_str!("File already exists: ", &destination.to_string_lossy()).into());
            }
            
            fs::rename(&entry.path, &destination)?;
            
        }
        
        Ok(())
    }
    
    pub fn delete<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let entry = self.get(&path).ok_or("Entry not found")?;
        
        fs::remove_file(&entry.path)?;
        
        Ok(())
    }
    
    pub fn mark<P: AsRef<Path>>(&self, path: P, mark: FilesMark) -> Result<(), Box<dyn Error>> {
        let entry = self.get(&path).ok_or("Entry not found")?;
        
        marker::set(&entry.path, &self.flag, mark)?;
        
        // since these kinds of changes are not picked up by the file watcher, they must be communicated manually
        // this means that an equivalent modification not made through this library will have no effect and a full reload should be performed to avoid loss of information
        if let Some((_, notify)) = self.watcher.as_ref() {
            notify(FilesWatcherEvent::FileRemoved(entry.path.to_path_buf()));
            notify(FilesWatcherEvent::FileAdded(entry.path.to_path_buf()));
        }
        
        Ok(())
    }
    
    pub fn perform_maintenance(&self) -> Result<(), Box<dyn Error>> {
        // updated files
        
        for file in self.entries.iter().filter(|entry| entry.mark == FilesMark::Updated) {
            fs::remove_file(&file.path)?;
        }
        
        // irrelevant files and directories
        
        for path in fs::read_dir(&self.root)?.flatten().map(|current| current.path()) {
            
            // symbolic links and junction points will be deleted
            if FilesWalker::new(path.clone()).find_map(|path| self.build_entry(path)).is_none() {
                
                if path.is_file() {
                    fs::remove_file(&path)?;
                } else {
                    fs::remove_dir_all(&path)?;
                }
                
            }
            
        }
        
        Ok(())
    }
    
    fn build_entry(&self, path: PathBuf) -> Option<FilesEntry> {
        let extension = path.extension()?;
        
        self.formats.iter().find(|format| format.eq_ignore_ascii_case(extension))?;
        
        let path = path.into_boxed_path();
        
        let name = path.file_stem()?
            .to_os_string()
            .into_boxed_os_str();
        
        let container = path.strip_prefix(&self.root).ok()?
            .parent()
            .map(Path::as_os_str)
            .filter(|parent| ! parent.is_empty())
            .map(ToOwned::to_owned)
            .map(OsString::into_boxed_os_str);
        
        let mark = marker::get(&path, &self.flag);
        
        Some(
            FilesEntry {
                path,
                name,
                container,
                mark,
            }
        )
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add<P: Into<PathBuf>>(&mut self, path: P) -> Result<&[FilesEntry], Box<dyn Error>> {
        let path = path.into();
        
        let mut to_be_added = FilesWalker::new(path)
            .filter_map(|path| self.build_entry(path))
            .filter(|entry| ! self.entries.contains(entry))
            .collect::<Vec<FilesEntry>>();
        
        if to_be_added.is_empty() {
            return Err("Could not find any entry to add for the specified path".into());
        }
        
        let previous = self.entries.len();
        
        self.entries.append(&mut to_be_added);
        
        Ok(&self.entries[previous..])
    }
    
    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<FilesEntry>, Box<dyn Error>> {
        let mut indexes = Vec::new();
        
        let path = path.as_ref();
        
        match self.entries.iter().position(|current| current.path.as_ref() == path) {
            
            // file
            Some(index) => indexes.push(index),
            
            // directory
            None => if path.starts_with(&self.root) {
                for (index, entry) in self.entries.iter().enumerate() {
                    if entry.path.ancestors().any(|ancestor| ancestor == path) {
                        indexes.push(index);
                    }
                }
            },
            
        }
        
        if indexes.is_empty() {
            return Err("Could not find any entry to remove for the specified path".into());
        }
        
        // indexes must be sorted in descending order since removing elements might invalidate them otherwise
        indexes.sort_unstable_by(|a, b| b.cmp(a));
        
        let mut result = Vec::with_capacity(indexes.len());
        
        for index in indexes {
            
            // remove entry
            result.push(self.entries.remove(index));
            
            if let Some(position) = self.queue.iter().position(|&current| current == index) {
                
                // remove index from queue
                self.queue.remove(position);
                
                // adjust remaining indexes
                for current in self.queue.iter_mut().filter(|current| **current >= index) {
                    *current -= 1;
                }
                
            }
            
        }
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        if self.queue.capacity() > self.queue.len().saturating_mul(2) {
            self.queue.shrink_to_fit();
        }
        
        Ok(result)
    }
    
    pub fn mount_watcher<N: Fn(FilesWatcherEvent) + Send + Clone + 'static>(&mut self, notify: N) -> Result<(), Box<dyn Error>> {
        self.unmount_watcher();
        
        let watcher = FilesWatcher::mount(&self.root, notify.clone())?;
        
        self.watcher = Some((watcher, Box::new(notify)));
        
        Ok(())
    }
    
    pub fn unmount_watcher(&mut self) {
        self.watcher = None;
    }
    
    pub fn refresh_queue<S: AsRef<OsStr>>(&mut self, selected: &[S]) {
        // indexes -> entries
        let mut queue = self.queue().collect::<Vec<&FilesEntry>>();
        
        // ---------- translate provided paths into entries and containers ----------
        
        let mut entries = Vec::with_capacity(selected.len());
        let mut containers = Vec::with_capacity(selected.len());
        
        for current in selected {
            
            let current = current.as_ref();
            
            if let Some(entry) = self.entries.iter().find(|entry| entry.path.as_ref() == current) {
                
                if ! entries.contains(&entry) {
                    entries.push(entry);
                }
                
                continue;
                
            }
            
            let wrapped = Some(current);
            
            if ! containers.contains(&wrapped) && self.entries.iter().any(|entry| entry.container.as_deref() == wrapped) {
                containers.push(wrapped);
            }
            
        }
        
        // ---------- remove unselected entries ----------
        
        queue.retain(|entry| entries.contains(entry) || containers.contains(&entry.container.as_deref()));
        
        // ---------- refresh subdirectories ----------
        
        for container in containers {
            
            if let Some(index) = queue.iter().position(|entry| entry.container.as_deref() == container) {
                
                // remove old entries from queue
                queue.retain(|entry| entry.container.as_deref() != container);
                
                let mut leftover = queue.split_off(index);
                
                // insert new entries from selected
                while let Some(index) = entries.iter().position(|entry| entry.container.as_deref() == container) {
                    queue.push(entries.remove(index));
                }
                
                queue.append(&mut leftover);
                
            }
            
        }
        
        // ---------- remove entries already in queue ----------
        
        entries.retain(|entry| ! queue.contains(entry));
        
        // ---------- add new entries ----------
        
        queue.append(&mut mem::take(&mut entries));
        
        // entries -> indexes
        self.queue = queue.iter()
            .filter_map(|&entry| self.entries.iter().position(|current| current == entry))
            .collect();
    }
    
}

impl FilesEntry {
    
    // ---------- accessors ----------
    
    
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    
    pub fn container(&self) -> Option<&OsStr> {
        self.container.as_deref()
    }
    
    pub fn mark(&self) -> FilesMark {
        self.mark
    }
    
}

#[cfg(test)]
mod lib {
    
    use std::fs::File;
    
    use super::*;
    
    mod add {
        
        use super::*;
        
        mod file {
            
            use super::*;
            
            #[test]
            fn valid() {
                // setup
                
                // root
                //  |-> tempfile
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(Some(&root), ".mkv");
                
                let entry = FilesEntry {
                    path: file_path.to_path_buf().into_boxed_path(),
                    name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(&file_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(&file_path), Some(&entry));
            }
            
            #[test]
            fn invalid() {
                // setup
                
                // root
                //  |-> tempfile
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(Some(&root), ".pdf");
                
                // operation
                
                let output = files.add(&file_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path = build_file(Some(&dir_path), ".mkv");
                
                let container = dir_path
                    .strip_prefix(&root)
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry = FilesEntry {
                    path: file_path.to_path_buf().into_boxed_path(),
                    name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(&file_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(&file_path), Some(&entry));
            }
            
            #[test]
            fn duplicated() {
                // setup
                
                // root
                //  |-> tempfile
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(Some(&root), ".mkv");
                
                let entry = FilesEntry {
                    path: file_path.to_path_buf().into_boxed_path(),
                    name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(&file_path).unwrap();
                
                // operation
                
                let output = files.add(&file_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.count(), 1);
                
                assert_eq!(files.get(&file_path), Some(&entry));
            }
            
            #[test]
            fn outside_of_root() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(None::<PathBuf>, ".mkv");
                
                // operation
                
                let output = files.add(&file_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path), None);
            }
            
        }
        
        mod directory {
            
            use super::*;
            
            #[test]
            fn valid() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".mkv");
                
                let entry_first = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let file_path_second = build_file(Some(&dir_path), ".mp4");
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(&dir_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 2);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                
                assert_eq!(files.get(&file_path_first), Some(&entry_first));
                assert_eq!(files.get(&file_path_second), Some(&entry_second));
            }
            
            #[test]
            fn invalid() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".txt");
                let file_path_second = build_file(Some(&dir_path), ".pdf");
                
                // operation
                
                let output = files.add(&dir_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path_first), None);
                assert_eq!(files.get(&file_path_second), None);
            }
            
            #[test]
            fn mixed() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".txt");
                let file_path_second = build_file(Some(&dir_path), ".mp4");
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(&dir_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry_second);
                
                assert_eq!(files.get(&file_path_first), None);
                assert_eq!(files.get(&file_path_second), Some(&entry_second));
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                // root
                //  |-> depth_one
                //      |-> depth_two
                //          |-> depth_three
                //              |-> tempfile_first
                //          |-> tempfile_second
                //          |-> tempfile_third
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path_first = build_dir(Some(&root));
                let dir_path_second = build_dir(Some(&dir_path_first));
                let dir_path_third = build_dir(Some(&dir_path_second));
                
                let file_path_first = build_file(Some(&dir_path_third), ".mkv");
                
                let container = dir_path_third
                    .strip_prefix(&root)
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_first = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                let file_path_second = build_file(Some(&dir_path_second), ".mp4");
                
                let container = dir_path_second
                    .strip_prefix(&root)
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                let file_path_third = build_file(Some(&dir_path_second), ".mp4");
                
                let container = dir_path_second
                    .strip_prefix(&root)
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_third = FilesEntry {
                    path: file_path_third.to_path_buf().into_boxed_path(),
                    name: file_path_third.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(&dir_path_second);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 3);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                assert!(output.contains(&entry_third));
                
                assert_eq!(files.get(&file_path_first), Some(&entry_first));
                assert_eq!(files.get(&file_path_second), Some(&entry_second));
                assert_eq!(files.get(&file_path_third), Some(&entry_third));
            }
            
            #[test]
            fn duplicated() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".mkv");
                
                let entry_first = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let file_path_second = build_file(Some(&dir_path), ".mp4");
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                files.add(&dir_path).unwrap();
                
                // operation
                
                let output = files.add(&dir_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.count(), 2);
                
                assert_eq!(files.get(&file_path_first), Some(&entry_first));
                assert_eq!(files.get(&file_path_second), Some(&entry_second));
            }
            
            #[test]
            fn outside_of_root() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(None::<PathBuf>);
                
                let file_path_first = build_file(Some(&dir_path), ".mkv");
                let file_path_second = build_file(Some(&dir_path), ".mp4");
                
                // operation
                
                let output = files.add(&dir_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path_first), None);
                assert_eq!(files.get(&file_path_second), None);
            }
            
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        mod file {
            
            use super::*;
            
            #[test]
            fn valid() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(Some(&root), ".mkv");
                
                let entry = FilesEntry {
                    path: file_path.to_path_buf().into_boxed_path(),
                    name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(&file_path).unwrap();
                
                // operation
                
                let output = files.remove(&file_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(&file_path), None);
            }
            
            #[test]
            fn not_found() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path = build_file(Some(&root), ".mkv");
                
                // operation
                
                let output = files.remove(&file_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path = build_file(Some(&dir_path), ".mkv");
                
                let container = dir_path
                    .strip_prefix(&root)
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry = FilesEntry {
                    path: file_path.to_path_buf().into_boxed_path(),
                    name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                files.add(&dir_path).unwrap();
                
                // operation
                
                let output = files.remove(&file_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(&file_path), None);
            }
            
            #[test]
            fn in_queue() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let file_path_first = build_file(Some(&root), ".mkv");
                
                let entry_first = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(&file_path_first).unwrap();
                
                let file_path_second = build_file(Some(&root), ".mkv");
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(&file_path_second).unwrap();
                
                let file_path_third = build_file(Some(&root), ".mkv");
                    
                let entry_third = FilesEntry {
                    path: file_path_third.to_path_buf().into_boxed_path(),
                    name: file_path_third.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(&file_path_third).unwrap();
                
                files.refresh_queue(&[
                    &file_path_first,
                    &file_path_second,
                    &file_path_third,
                ]);
                
                // operation
                
                let output = files.remove(&file_path_second);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry_second);
                
                assert_eq!(files.get(&file_path_second), None);
                
                assert_eq!(files.queue().collect::<Vec<&FilesEntry>>(), [&entry_first, &entry_third]);
            }
            
        }
        
        mod directory {
            
            use super::*;
            
            #[test]
            fn valid() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".mkv");
                
                let entry_first = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let file_path_second = build_file(Some(&dir_path), ".mp4");
                
                let entry_second = FilesEntry {
                    path: file_path_second.to_path_buf().into_boxed_path(),
                    name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                files.add(&dir_path).unwrap();
                
                // operation
                
                let output = files.remove(&dir_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 2);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                
                assert_eq!(files.get(&file_path_first), None);
                assert_eq!(files.get(&file_path_second), None);
            }
            
            #[test]
            fn not_found() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path = build_file(Some(&dir_path), ".xls");
                
                // operation
                
                let output = files.remove(&dir_path);
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(&file_path), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                let root = build_dir(None::<PathBuf>);
                
                let mut files = new(&root);
                
                let dir_path = build_dir(Some(&root));
                
                let file_path_first = build_file(Some(&dir_path), ".mkv");
                
                let entry = FilesEntry {
                    path: file_path_first.to_path_buf().into_boxed_path(),
                    name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(dir_path.file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let file_path_second = build_file(Some(&dir_path), ".pdf");
                
                files.add(&dir_path).unwrap();
                
                // operation
                
                let output = files.remove(&dir_path);
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(&file_path_first), None);
                assert_eq!(files.get(&file_path_second), None);
            }
            
        }
        
    }
    
    mod mark {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            let entry = FilesEntry {
                path: file_path.to_path_buf().into_boxed_path(),
                name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::Watched,
            };
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.mark(&file_path, FilesMark::Watched);
            
            // control
            
            assert!(output.is_ok());
            
            files.remove(&file_path).unwrap();
            files.add(&file_path).unwrap();
            
            assert_eq!(files.get(&file_path), Some(&entry));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            let entry = FilesEntry {
                path: file_path.to_path_buf().into_boxed_path(),
                name: file_path.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.mark(&file_path, FilesMark::Watched);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(files.get(&file_path), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let files = new(&root);
            
            let file_path = build_file(Some(&root), ".pdf");
            
            // operation
            
            let output = files.mark(&file_path, FilesMark::Watched);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod refresh_queue {
        
        use super::*;
        
        #[test]
        fn file() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            let entry = FilesEntry {
                path: file_path_first.to_path_buf().into_boxed_path(),
                name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&root), ".txt");
            
            // operation
            
            files.refresh_queue(&[
                &file_path_first,
                &file_path_second,
                &file_path_first,
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 1);
            
            assert_eq!(queue, [&entry]);
        }
        
        #[test]
        fn directory() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let dir_path = build_dir(Some(&root));
            
            let file_path_first = build_file(Some(&dir_path), ".mkv");
            
            let container = dir_path
                .strip_prefix(&root)
                .unwrap()
                .as_os_str()
                .to_owned()
                .into_boxed_os_str();
            
            let entry_first = FilesEntry {
                path: file_path_first.to_path_buf().into_boxed_path(),
                name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: Some(container),
                mark: FilesMark::None,
            };
            
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&dir_path), ".mkv");
            
            let container = dir_path
                .strip_prefix(&root)
                .unwrap()
                .as_os_str()
                .to_owned()
                .into_boxed_os_str();
            
            let entry_second = FilesEntry {
                path: file_path_second.to_path_buf().into_boxed_path(),
                name: file_path_second.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: Some(container),
                mark: FilesMark::None,
            };
            
            files.add(&file_path_second).unwrap();
            
            let file_path_third = build_file(Some(&root), ".mp4");
            
            let entry_third = FilesEntry {
                path: file_path_third.to_path_buf().into_boxed_path(),
                name: file_path_third.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(&file_path_third).unwrap();
            
            files.refresh_queue(&[
                &file_path_first,
                &file_path_third,
                &file_path_second,
            ]);
            
            // operation
            
            let container = dir_path
                .strip_prefix(&root)
                .unwrap()
                .as_os_str();
            
            files.refresh_queue(&[
                file_path_first.as_os_str(),
                container,
                file_path_first.as_os_str(),
                file_path_second.as_os_str(),
                file_path_third.as_os_str(),
                file_path_second.as_os_str(),
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 3);
            
            assert_eq!(queue, [&entry_first, &entry_second, &entry_third]);
        }
        
        #[test]
        fn deselection() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            let entry_first = FilesEntry {
                path: file_path_first.to_path_buf().into_boxed_path(),
                name: file_path_first.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&root), ".mp4");
            
            files.add(&file_path_second).unwrap();
            
            let file_path_third = build_file(Some(&root), ".mp4");
            
            let entry_third = FilesEntry {
                path: file_path_third.to_path_buf().into_boxed_path(),
                name: file_path_third.file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(&file_path_third).unwrap();
            
            files.refresh_queue(&[
                &file_path_first,
                &file_path_second,
                &file_path_third,
            ]);
            
            // operation
            
            files.refresh_queue(&[
                &file_path_first,
                &file_path_third,
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 2);
            
            assert_eq!(queue, [&entry_first, &entry_third]);
        }
        
        #[test]
        fn empty() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&root), ".mp4");
            
            files.refresh_queue(&[
                &file_path_first,
                &file_path_second,
            ]);
            
            // operation
            
            files.refresh_queue(&[PathBuf::new()]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert!(queue.is_empty());
        }
        
    }
    
    mod rename {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            
            // operation
            
            let output = files.rename(&file_path_first, "testing.txt");
            
            // control
            
            assert!(output.is_ok());
            
            assert!(file_path_first.with_file_name("testing").with_extension("mkv").exists());
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mp4");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.rename(&file_path, "..");
            
            // control
            
            assert!(output.is_err());
            
            assert!(file_path.exists());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let files = new(&root);
            
            let file_path = build_file(Some(&root), ".mp4");
            
            // operation
            
            let output = files.rename(&file_path, "testing.txt");
            
            // control
            
            assert!(output.is_err());
            
            assert!(file_path.exists());
        }
        
        #[test]
        fn overwrite() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mp4");
            
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&root), ".mp4");
            
            // operation
            
            let output = files.rename(&file_path_first, file_path_second.file_name().unwrap());
            
            // control
            
            assert!(output.is_err());
            
            assert!(&file_path_first.exists());
            assert!(&file_path_second.exists());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.rename(&file_path, file_path.file_name().unwrap());
            
            // control
            
            assert!(output.is_ok());
            
            assert!(file_path.exists());
        }
        
    }
    
    mod move_to_folder {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.move_to_folder(&file_path, Some("../../test"));
            
            // control
            
            assert!(output.is_ok());
            
            assert!(root.join("test").join(file_path.file_name().unwrap()).exists());
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.move_to_folder(&file_path, Some(".."));
            
            // control
            
            assert!(output.is_err());
            
            assert!(file_path.exists());
        }
        
        #[test]
        fn to_root() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let dir_path = build_dir(Some(&root));
            
            let file_path = build_file(Some(&dir_path), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.move_to_folder(&file_path, None::<&OsStr>);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(root.join(file_path.file_name().unwrap()).exists());
        }
        
        #[test]
        fn overwrite() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            
            let dir_path = build_dir(Some(&root));
            
            let file_path_second = build_file(Some(&dir_path), ".mkv");
            
            files.rename(&file_path_first, file_path_second.file_stem().unwrap()).unwrap();
            
            file_path_first.unmanage();
            
            let file_path_first = chikuwa::EphemeralPath::from(file_path_second.to_path_buf());
            
            // operation
            
            let output = files.move_to_folder(&file_path_first, dir_path.file_name());
            
            // control
            
            assert!(output.is_err());
            
            assert!(file_path_first.exists());
            assert!(file_path_second.exists());
        }
        
        #[test]
        fn already_in_root() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            
            // operation
            
            let output = files.move_to_folder(&file_path_first, None::<&OsStr>);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(file_path_first.exists());
        }
        
        #[test]
        fn already_in_subdirectory() {
            // setup
            
            // root
            //  |-> subdirectory
            //      |-> tempfile
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let dir_path = build_dir(Some(&root));
            
            let file_path = build_file(Some(&dir_path), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.move_to_folder(&file_path, dir_path.file_stem());
            
            // control
            
            assert!(output.is_ok());
            
            assert!(file_path.exists());
        }
        
    }
    
    mod delete {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path = build_file(Some(&root), ".mkv");
            
            files.add(&file_path).unwrap();
            
            // operation
            
            let output = files.delete(&file_path);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(! file_path.exists());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let files = new(&root);
            
            let file_path = build_file(Some(&root), ".mp4");
            
            // operation
            
            let output = files.delete(&file_path);
            
            // control
            
            assert!(output.is_err());
            
            assert!(file_path.exists());
        }
        
    }
    
    mod perform_maintenance {
        
        use super::*;
        
        #[test]
        fn updated_files() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            files.mark(&file_path_first, FilesMark::Updated).unwrap();
            files.remove(&file_path_first).unwrap();
            files.add(&file_path_first).unwrap();
            
            let file_path_second = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_second).unwrap();
            files.mark(&file_path_second, FilesMark::Updated).unwrap();
            files.remove(&file_path_second).unwrap();
            files.add(&file_path_second).unwrap();
            
            let file_path_third = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_third).unwrap();
            
            // operation
            
            let output = files.perform_maintenance();
            
            // control
            
            assert!(output.is_ok());
            
            assert!(! file_path_first.exists());
            assert!(! file_path_second.exists());
            assert!(file_path_third.exists());
        }
        
        #[test]
        fn empty_directories() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let mut files = new(&root);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            
            files.add(&file_path_first).unwrap();
            
            let dir_path_first = build_dir(Some(&root));
            
            let file_path_second = build_file(Some(&dir_path_first), ".mkv");
            
            files.add(&file_path_second).unwrap();
            files.mark(&file_path_second, FilesMark::Updated).unwrap();
            files.remove(&file_path_second).unwrap();
            files.add(&file_path_second).unwrap();
            
            let dir_path_second = build_dir(Some(&root));
            
            let file_path_third = build_file(Some(&dir_path_second), ".mkv");
            
            files.add(&file_path_third).unwrap();
            files.mark(&file_path_third, FilesMark::Updated).unwrap();
            files.remove(&file_path_third).unwrap();
            files.add(&file_path_third).unwrap();
            
            let file_path_fourth = build_file(Some(&dir_path_second), ".mkv");
            
            files.add(&file_path_fourth).unwrap();
            
            let dir_path_third = build_dir(Some(&root));
            
            // operation
            
            let output = files.perform_maintenance();
            
            // control
            
            assert!(output.is_ok());
            
            assert!(file_path_first.exists());
            assert!(! file_path_second.exists());
            assert!(! dir_path_first.exists());
            assert!(! file_path_third.exists());
            assert!(file_path_fourth.exists());
            assert!(dir_path_second.exists());
            assert!(! dir_path_third.exists());
        }
        
    }
    
    mod new {
        
        use super::*;
        
        #[test]
        fn pre_existing() {
            // setup
            
            let root = build_dir(None::<PathBuf>);
            
            let file_path_first = build_file(Some(&root), ".mkv");
            let file_path_second = build_file(Some(&root), ".mkv");
            let file_path_third = build_file(Some(&root), ".pdf");
            
            // operation
            
            let output = new(&root);
            
            // control
            
            assert_eq!(output.count(), 2);
            
            assert!(output.get(&file_path_first).is_some());
            assert!(output.get(&file_path_second).is_some());
            assert!(output.get(&file_path_third).is_none());
            
            assert!(output.queue().next().is_none());
        }
        
    }
    
    fn new<P: Into<PathBuf>>(path: P) -> Files {
        let flag = "user.app.ena";
        let formats = Vec::from(["mkv", "mp4", "avi"]);
        
        Files::new(path, flag, formats.iter())
    }
    
    fn build_dir<P: Into<PathBuf>>(base: Option<P>) -> chikuwa::EphemeralPath {
        let path = build_path(base, None);
        
        fs::create_dir(&path).unwrap();
        
        path
    }
    
    fn build_file<P: Into<PathBuf>>(base: Option<P>, extension: &str) -> chikuwa::EphemeralPath {
        let path = build_path(base, Some(extension));
        
        File::create(&path).unwrap();
        
        path
    }
    
    fn build_path<P: Into<PathBuf>>(base: Option<P>, suffix: Option<&str>) -> chikuwa::EphemeralPath {
        let mut builder = chikuwa::EphemeralPath::builder();
        
        if let Some(base) = base {
            builder = builder.with_base(base);
        }
        
        if let Some(suffix) = suffix {
            builder = builder.with_suffix(suffix);
        }
        
        builder.build()
    }
    
}
