mod marks;
mod watcher;

use std::{
    error::Error,
    ffi::{ OsStr, OsString },
    fs,
    path::{ Path, PathBuf },
};

pub use marks::FilesMark;
pub use watcher::FilesWatcherEvent;
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
        
        if let Some(entries) = files.walk_path(&files.root) {
            files.entries = entries;
        }
        
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
                return Err(format!("File already exists: {}", destination.to_string_lossy()).into());
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
                return Err(format!("File already exists: {}", destination.to_string_lossy()).into());
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
    
    pub fn mark<P: AsRef<Path>>(&self, path: P, mark: FilesMark) -> Result<bool, Box<dyn Error>> {
        let entry = self.get(&path).ok_or("Entry not found")?;
        
        let changed = marks::set(&self.flag, entry, mark)?;
        
        if changed {
            
            // since these kinds of changes are not picked up by the file watcher, they must be communicated manually
            // this means that an equivalent modification not made through this library will have no effect and a full reload should be performed to avoid loss of information
            if let Some((_, notify)) = self.watcher.as_ref() {
                notify(FilesWatcherEvent::FileRemoved(entry.path.to_path_buf()));
                notify(FilesWatcherEvent::FileAdded(entry.path.to_path_buf()));
            }
            
        }
        
        Ok(changed)
    }
    
    pub fn perform_maintenance(&self) -> Result<(), Box<dyn Error>> {
        // updated files
        
        for file in self.entries.iter().filter(|entry| entry.mark == FilesMark::Updated) {
            fs::remove_file(&file.path)?;
        }
        
        // irrelevant files in root and empty directories
        
        for path in fs::read_dir(&self.root)?.flatten().map(|current| current.path()) {
            
            // this will delete symbolic links and junction points
            if self.walk_path(&path).is_none() {
                
                if path.is_file() {
                    fs::remove_file(&path)?;
                } else {
                    fs::remove_dir_all(&path)?;
                }
                
            }
            
        }
        
        Ok(())
    }
    
    fn walk_path(&self, path: &Path) -> Option<Vec<FilesEntry>> {
        let metadata = path.symlink_metadata().ok()?;
        
        // disallow symbolic links and junction points
        if metadata.is_symlink() {
            return None;
        }
        
        let base = path.strip_prefix(&self.root).ok()?;
        
        let result = if metadata.is_file() {
            
            Vec::from([FilesEntry::build(path, base, &self.formats, &self.flag)?])
            
        } else {
            
            fs::read_dir(path).ok()?
                .flatten()
                .filter_map(|file| self.walk_path(&file.path()))
                .flatten()
                .collect()
            
        };
        
        if result.is_empty() {
            return None;
        }
        
        Some(result)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> Result<&[FilesEntry], Box<dyn Error>> {
        let path = path.as_ref();
        
        let mut to_be_added = self.walk_path(path)
            .into_iter()
            .flatten()
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
        
        queue.append(&mut entries.drain(..).collect());
        
        // entries -> indexes
        self.queue = queue.iter()
            .filter_map(|&entry| self.entries.iter().position(|current| current == entry))
            .collect();
    }
    
}

impl FilesEntry {
    
    // ---------- constructors ----------
    
    
    fn build(path: &Path, base: &Path, formats: &[Box<OsStr>], flag: &OsStr) -> Option<Self> {
        let extension = path.extension()?;
        
        formats.iter().find(|format| format.eq_ignore_ascii_case(extension))?;
        
        let path = path.to_owned()
            .into_boxed_path();
        
        let name = path.file_stem()?
            .to_os_string()
            .into_boxed_os_str();
        
        let container = base.parent()
            .map(Path::as_os_str)
            .filter(|parent| ! parent.is_empty())
            .map(ToOwned::to_owned)
            .map(OsString::into_boxed_os_str);
        
        let mark = marks::get(flag, &path);
        
        Some(
            Self {
                path,
                name,
                container,
                mark,
            }
        )
    }
    
    
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
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                let entry = FilesEntry {
                    path: tempfile.path().to_owned().into_boxed_path(),
                    name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(tempfile.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(tempfile.path()), Some(&entry));
            }
            
            #[test]
            fn invalid() {
                // setup
                
                // root
                //  |-> tempfile
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".pdf")
                    .tempfile_in(&root)
                    .unwrap();
                
                // operation
                
                let output = files.add(tempfile.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile.path()), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let container = subdirectory.path()
                    .strip_prefix(root.path())
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry = FilesEntry {
                    path: tempfile.path().to_owned().into_boxed_path(),
                    name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(tempfile.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(tempfile.path()), Some(&entry));
            }
            
            #[test]
            fn duplicated() {
                // setup
                
                // root
                //  |-> tempfile
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                let entry = FilesEntry {
                    path: tempfile.path().to_owned().into_boxed_path(),
                    name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(tempfile.path()).unwrap();
                
                // operation
                
                let output = files.add(tempfile.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.count(), 1);
                
                assert_eq!(files.get(tempfile.path()), Some(&entry));
            }
            
            #[test]
            fn outside_of_root() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile()
                    .unwrap();
                
                // operation
                
                let output = files.add(tempfile.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile.path()), None);
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
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_first = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(subdirectory.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 2);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                
                assert_eq!(files.get(tempfile_first.path()), Some(&entry_first));
                assert_eq!(files.get(tempfile_second.path()), Some(&entry_second));
            }
            
            #[test]
            fn invalid() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".txt")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".pdf")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                // operation
                
                let output = files.add(subdirectory.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile_first.path()), None);
                assert_eq!(files.get(tempfile_second.path()), None);
            }
            
            #[test]
            fn mixed() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".txt")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(subdirectory.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry_second);
                
                assert_eq!(files.get(tempfile_first.path()), None);
                assert_eq!(files.get(tempfile_second.path()), Some(&entry_second));
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
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let depth_one = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let depth_two = tempfile::Builder::new()
                    .tempdir_in(&depth_one)
                    .unwrap();
                
                let depth_three = tempfile::Builder::new()
                    .tempdir_in(&depth_two)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&depth_three)
                    .unwrap();
                
                let container = depth_three.path()
                    .strip_prefix(root.path())
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_first = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&depth_two)
                    .unwrap();
                
                let container = depth_two.path()
                    .strip_prefix(root.path())
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                let tempfile_third = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&depth_two)
                    .unwrap();
                
                let container = depth_two.path()
                    .strip_prefix(root.path())
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry_third = FilesEntry {
                    path: tempfile_third.path().to_owned().into_boxed_path(),
                    name: tempfile_third.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                // operation
                
                let output = files.add(depth_two.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 3);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                assert!(output.contains(&entry_third));
                
                assert_eq!(files.get(tempfile_first.path()), Some(&entry_first));
                assert_eq!(files.get(tempfile_second.path()), Some(&entry_second));
                assert_eq!(files.get(tempfile_third.path()), Some(&entry_third));
            }
            
            #[test]
            fn duplicated() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_first = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                files.add(subdirectory.path()).unwrap();
                
                // operation
                
                let output = files.add(subdirectory.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.count(), 2);
                
                assert_eq!(files.get(tempfile_first.path()), Some(&entry_first));
                assert_eq!(files.get(tempfile_second.path()), Some(&entry_second));
            }
            
            #[test]
            fn outside_of_root() {
                // setup
                
                // root
                //  |-> subdirectory
                //      |-> tempfile_first
                //      |-> tempfile_second
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::tempdir().unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                // operation
                
                let output = files.add(subdirectory.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile_first.path()), None);
                assert_eq!(files.get(tempfile_second.path()), None);
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
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                let entry = FilesEntry {
                    path: tempfile.path().to_owned().into_boxed_path(),
                    name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(tempfile.path()).unwrap();
                
                // operation
                
                let output = files.remove(tempfile.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(tempfile.path()), None);
            }
            
            #[test]
            fn not_found() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                // operation
                
                let output = files.remove(tempfile.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile.path()), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let container = subdirectory.path()
                    .strip_prefix(root.path())
                    .unwrap()
                    .as_os_str()
                    .to_owned()
                    .into_boxed_os_str();
                
                let entry = FilesEntry {
                    path: tempfile.path().to_owned().into_boxed_path(),
                    name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(container),
                    mark: FilesMark::None,
                };
                
                files.add(subdirectory.path()).unwrap();
                
                // operation
                
                let output = files.remove(tempfile.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(tempfile.path()), None);
            }
            
            #[test]
            fn in_queue() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                let entry_first = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(tempfile_first.path()).unwrap();
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(tempfile_second.path()).unwrap();
                
                let tempfile_third = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&root)
                    .unwrap();
                    
                let entry_third = FilesEntry {
                    path: tempfile_third.path().to_owned().into_boxed_path(),
                    name: tempfile_third.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: None,
                    mark: FilesMark::None,
                };
                
                files.add(tempfile_third.path()).unwrap();
                
                files.refresh_queue(&[
                    tempfile_first.path(),
                    tempfile_second.path(),
                    tempfile_third.path(),
                ]);
                
                // operation
                
                let output = files.remove(tempfile_second.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry_second);
                
                assert_eq!(files.get(tempfile_second.path()), None);
                
                assert_eq!(files.queue().collect::<Vec<&FilesEntry>>(), [&entry_first, &entry_third]);
            }
            
        }
        
        mod directory {
            
            use super::*;
            
            #[test]
            fn valid() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_first = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".mp4")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry_second = FilesEntry {
                    path: tempfile_second.path().to_owned().into_boxed_path(),
                    name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                files.add(subdirectory.path()).unwrap();
                
                // operation
                
                let output = files.remove(subdirectory.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 2);
                
                assert!(output.contains(&entry_first));
                assert!(output.contains(&entry_second));
                
                assert_eq!(files.get(tempfile_first.path()), None);
                assert_eq!(files.get(tempfile_second.path()), None);
            }
            
            #[test]
            fn not_found() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile = tempfile::Builder::new()
                    .suffix(".xls")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                // operation
                
                let output = files.remove(subdirectory.path());
                
                // control
                
                assert!(output.is_err());
                
                assert_eq!(files.get(tempfile.path()), None);
            }
            
            #[test]
            fn in_subdirectory() {
                // setup
                
                let root = tempfile::tempdir().unwrap();
                
                let mut files = new(root.path());
                
                let subdirectory = tempfile::Builder::new()
                    .tempdir_in(&root)
                    .unwrap();
                
                let tempfile_first = tempfile::Builder::new()
                    .suffix(".mkv")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                let entry = FilesEntry {
                    path: tempfile_first.path().to_owned().into_boxed_path(),
                    name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                    container: Some(subdirectory.path().file_stem().unwrap().to_owned().into_boxed_os_str()),
                    mark: FilesMark::None,
                };
                
                let tempfile_second = tempfile::Builder::new()
                    .suffix(".pdf")
                    .tempfile_in(&subdirectory)
                    .unwrap();
                
                files.add(subdirectory.path()).unwrap();
                
                // operation
                
                let output = files.remove(subdirectory.path());
                
                // control
                
                assert!(output.is_ok());
                
                let output = output.unwrap();
                
                assert_eq!(output.len(), 1);
                
                assert_eq!(output.first().unwrap(), &entry);
                
                assert_eq!(files.get(tempfile_first.path()), None);
                assert_eq!(files.get(tempfile_second.path()), None);
            }
            
        }
        
    }
    
    mod mark {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let entry = FilesEntry {
                path: tempfile.path().to_owned().into_boxed_path(),
                name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::Watched,
            };
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.mark(tempfile.path(), FilesMark::Watched);
            
            // control
            
            assert!(output.is_ok());
            
            files.remove(tempfile.path()).unwrap();
            files.add(tempfile.path()).unwrap();
            
            assert_eq!(files.get(tempfile.path()), Some(&entry));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let entry = FilesEntry {
                path: tempfile.path().to_owned().into_boxed_path(),
                name: tempfile.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.mark(tempfile.path(), FilesMark::Watched);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(files.get(tempfile.path()), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".pdf")
                .tempfile_in(&root)
                .unwrap();
            
            // operation
            
            let output = files.mark(tempfile.path(), FilesMark::Watched);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod refresh_queue {
        
        use super::*;
        
        #[test]
        fn file() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let entry = FilesEntry {
                path: tempfile_first.path().to_owned().into_boxed_path(),
                name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(tempfile_first.path()).unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".txt")
                .tempfile_in(&root)
                .unwrap();
            
            // operation
            
            files.refresh_queue(&[
                tempfile_first.path(),
                tempfile_second.path(),
                tempfile_first.path(),
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 1);
            
            assert_eq!(queue, [&entry]);
        }
        
        #[test]
        fn directory() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let subdirectory = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory)
                .unwrap();
            
            let container = subdirectory.path()
                .strip_prefix(root.path())
                .unwrap()
                .as_os_str()
                .to_owned()
                .into_boxed_os_str();
            
            let entry_first = FilesEntry {
                path: tempfile_first.path().to_owned().into_boxed_path(),
                name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: Some(container),
                mark: FilesMark::None,
            };
            
            files.add(tempfile_first.path()).unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory)
                .unwrap();
            
            let container = subdirectory.path()
                .strip_prefix(root.path())
                .unwrap()
                .as_os_str()
                .to_owned()
                .into_boxed_os_str();
            
            let entry_second = FilesEntry {
                path: tempfile_second.path().to_owned().into_boxed_path(),
                name: tempfile_second.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: Some(container),
                mark: FilesMark::None,
            };
            
            files.add(tempfile_second.path()).unwrap();
            
            let tempfile_third = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            let entry_third = FilesEntry {
                path: tempfile_third.path().to_owned().into_boxed_path(),
                name: tempfile_third.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(tempfile_third.path()).unwrap();
            
            files.refresh_queue(&[
                tempfile_first.path(),
                tempfile_third.path(),
                tempfile_second.path(),
            ]);
            
            // operation
            
            let container = subdirectory.path()
                .strip_prefix(root.path())
                .unwrap()
                .as_os_str();
            
            files.refresh_queue(&[
                tempfile_first.path().as_os_str(),
                container,
                tempfile_first.path().as_os_str(),
                tempfile_second.path().as_os_str(),
                tempfile_third.path().as_os_str(),
                tempfile_second.path().as_os_str(),
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 3);
            
            assert_eq!(queue, [&entry_first, &entry_second, &entry_third]);
        }
        
        #[test]
        fn deselection() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let entry_first = FilesEntry {
                path: tempfile_first.path().to_owned().into_boxed_path(),
                name: tempfile_first.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(tempfile_first.path()).unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_second.path()).unwrap();
            
            let tempfile_third = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            let entry_third = FilesEntry {
                path: tempfile_third.path().to_owned().into_boxed_path(),
                name: tempfile_third.path().file_stem().unwrap().to_owned().into_boxed_os_str(),
                container: None,
                mark: FilesMark::None,
            };
            
            files.add(tempfile_third.path()).unwrap();
            
            files.refresh_queue(&[
                tempfile_first.path(),
                tempfile_second.path(),
                tempfile_third.path(),
            ]);
            
            // operation
            
            files.refresh_queue(&[
                tempfile_first.path(),
                tempfile_third.path(),
            ]);
            
            // control
            
            let queue = files.queue().collect::<Vec<&FilesEntry>>();
            
            assert_eq!(queue.len(), 2);
            
            assert_eq!(queue, [&entry_first, &entry_third]);
        }
        
        #[test]
        fn empty() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_first.path()).unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            files.refresh_queue(&[
                tempfile_first.path(),
                tempfile_second.path(),
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
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.rename(tempfile.path(), "testing.txt");
            
            // control
            
            assert!(output.is_ok());
            
            assert!(tempfile.path().with_file_name("testing").with_extension("mkv").exists());
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.rename(tempfile.path(), "..");
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile.path().exists());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            // operation
            
            let output = files.rename(tempfile.path(), "testing.txt");
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile.path().exists());
        }
        
        #[test]
        fn overwrite() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_first.path()).unwrap();
            
            // operation
            
            let output = files.rename(tempfile_first.path(), tempfile_second.path().file_name().unwrap());
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile_first.path().exists());
            assert!(tempfile_second.path().exists());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.rename(tempfile.path(), tempfile.path().file_name().unwrap());
            
            // control
            
            assert!(output.is_ok());
            
            assert!(tempfile.path().exists());
        }
        
    }
    
    mod move_to_folder {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile.path(), Some("../../test"));
            
            // control
            
            assert!(output.is_ok());
            
            assert!(root.path().join("test").join(tempfile.path().file_name().unwrap()).exists());
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile.path(), Some(".."));
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile.path().exists());
        }
        
        #[test]
        fn to_root() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let subdirectory = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile.path(), None::<&OsStr>);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(root.path().join(tempfile.path().file_name().unwrap()).exists());
        }
        
        #[test]
        fn overwrite() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let subdirectory = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .prefix(tempfile_first.path().file_name().unwrap())
                .rand_bytes(0)
                .tempfile_in(&subdirectory)
                .unwrap();
            
            files.add(tempfile_first.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile_first.path(), subdirectory.path().file_name());
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile_first.path().exists());
            assert!(tempfile_second.path().exists());
        }
        
        #[test]
        fn already_in_root() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile.path(), None::<&OsStr>);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(tempfile.path().exists());
        }
        
        #[test]
        fn already_in_subdirectory() {
            // setup
            
            // root
            //  |-> subdirectory
            //      |-> tempfile
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let subdirectory = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.move_to_folder(tempfile.path(), subdirectory.path().file_stem());
            
            // control
            
            assert!(output.is_ok());
            
            assert!(tempfile.path().exists());
        }
        
    }
    
    mod delete {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile.path()).unwrap();
            
            // operation
            
            let output = files.delete(tempfile.path());
            
            // control
            
            assert!(output.is_ok());
            
            assert!(! tempfile.path().exists());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let files = new(root.path());
            
            let tempfile = tempfile::Builder::new()
                .suffix(".mp4")
                .tempfile_in(&root)
                .unwrap();
            
            // operation
            
            let output = files.delete(tempfile.path());
            
            // control
            
            assert!(output.is_err());
            
            assert!(tempfile.path().exists());
        }
        
    }
    
    mod perform_maintenance {
        
        use super::*;
        
        #[test]
        fn updated_files() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_first.path()).unwrap();
            
            files.mark(tempfile_first.path(), FilesMark::Updated).unwrap();
            
            files.remove(tempfile_first.path()).unwrap();
            files.add(tempfile_first.path()).unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_second.path()).unwrap();
            
            files.mark(tempfile_second.path(), FilesMark::Updated).unwrap();
            
            files.remove(tempfile_second.path()).unwrap();
            files.add(tempfile_second.path()).unwrap();
            
            let tempfile_third = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_third.path()).unwrap();
            
            // operation
            
            let output = files.perform_maintenance();
            
            // control
            
            assert!(output.is_ok());
            
            assert!(! tempfile_first.path().exists());
            assert!(! tempfile_second.path().exists());
            assert!(tempfile_third.path().exists());
        }
        
        #[test]
        fn empty_directories() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let mut files = new(root.path());
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            files.add(tempfile_first.path()).unwrap();
            
            let subdirectory_first = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory_first)
                .unwrap();
            
            files.add(tempfile_second.path()).unwrap();
            
            files.mark(tempfile_second.path(), FilesMark::Updated).unwrap();
            
            files.remove(tempfile_second.path()).unwrap();
            files.add(tempfile_second.path()).unwrap();
            
            let subdirectory_second = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            let tempfile_third = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory_second)
                .unwrap();
            
            files.add(tempfile_third.path()).unwrap();
            
            files.mark(tempfile_third.path(), FilesMark::Updated).unwrap();
            
            files.remove(tempfile_third.path()).unwrap();
            files.add(tempfile_third.path()).unwrap();
            
            let tempfile_fourth = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&subdirectory_second)
                .unwrap();
            
            files.add(tempfile_fourth.path()).unwrap();
            
            let subdirectory_third = tempfile::Builder::new()
                .tempdir_in(&root)
                .unwrap();
            
            // operation
            
            let output = files.perform_maintenance();
            
            // control
            
            assert!(output.is_ok());
            
            assert!(tempfile_first.path().exists());
            assert!(! tempfile_second.path().exists());
            assert!(! subdirectory_first.path().exists());
            assert!(! tempfile_third.path().exists());
            assert!(tempfile_fourth.path().exists());
            assert!(subdirectory_second.path().exists());
            assert!(! subdirectory_third.path().exists());
        }
        
    }
    
    mod new {
        
        use super::*;
        
        #[test]
        fn pre_existing() {
            // setup
            
            let root = tempfile::tempdir().unwrap();
            
            let tempfile_first = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let tempfile_second = tempfile::Builder::new()
                .suffix(".mkv")
                .tempfile_in(&root)
                .unwrap();
            
            let tempfile_third = tempfile::Builder::new()
                .suffix(".pdf")
                .tempfile_in(&root)
                .unwrap();
            
            let files = new(root.path());
            
            // operation
            
            let output = files.iter();
            
            // control
            
            for file in output {
                assert!(file.path() == tempfile_first.path() || file.path() == tempfile_second.path());
            }
            
            assert_eq!(files.count(), 2);
            
            assert!(files.get(tempfile_first.path()).is_some());
            assert!(files.get(tempfile_second.path()).is_some());
            assert!(files.get(tempfile_third.path()).is_none());
            
            assert!(files.queue().next().is_none());
        }
        
    }
    
    fn new(path: &Path) -> Files {
        let flag = "user.app.ena";
        let formats = Vec::from(["mkv", "mp4", "avi"]);
        
        Files::new(path, flag, formats.iter())
    }
    
}
