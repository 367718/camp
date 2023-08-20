use std::path::Path;

use chiaki::CandidatesEntry;

pub struct UpdatesEntries<'f, 'c> {
    files: &'f [(&'f str, &'f Path)],
    candidates: &'c [&'c CandidatesEntry],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct UpdatesEntry<'f, 'c> {
    pub name: &'f str,
    pub path: &'f Path,
    pub episode: i64,
    pub candidate: &'c CandidatesEntry,
}

impl<'f, 'c> UpdatesEntries<'f, 'c> {
    
    pub fn get(files: &'f [(&'f str, &'f Path)], candidates: &'c [&'c CandidatesEntry]) -> Self {
        Self {
            files,
            candidates,
        }
    }
    
}

impl<'f, 'c> Iterator for UpdatesEntries<'f, 'c> {
    
    type Item = UpdatesEntry<'f, 'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(((name, path), rest)) = self.files.split_first() {
            
            let entry = build_entry(name, path, self.candidates);
            self.files = rest;
            
            if entry.is_some() {
                return entry;
            }
            
        }
        
        None
    }
    
}

fn build_entry<'f, 'c>(name: &'f str, path: &'f Path, candidates: &'c [&'c CandidatesEntry]) -> Option<UpdatesEntry<'f, 'c>> {
    let candidate = candidates.iter()
        .find(|candidate| candidate.pieces().iter().all(|piece| name.contains(piece)))?;
    
    let episode = crate::extractor::get(name, &candidate.pieces())?;
    
    Some(UpdatesEntry {
        name,
        path,
        episode,
        candidate,
    })
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let files = generate_files();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Fictional")),
                &CandidatesEntry::new().with_title(String::from("Not defined")),
                &CandidatesEntry::new().with_title(String::from("Test")),
            ];
            
            // operation
            
            let output = UpdatesEntries::get(&files, &candidates);
            
            // control
            
            let output: Vec<UpdatesEntry> = output.collect();
            
            assert_eq!(output, Vec::from([
                UpdatesEntry {
                    name: "[Imaginary] Fictional - 9 [480p]",
                    path: Path::new("fake/path/[Imaginary] Fictional - 9 [480p].mp4"),
                    episode: 9,
                    candidate: &CandidatesEntry::new().with_title(String::from("Fictional")),
                },
                UpdatesEntry {
                    name: "[Imaginary] Fictional - 10 [480p]",
                    path: Path::new("fake/path/[Imaginary] Fictional - 10 [480p].mp4"),
                    episode: 10,
                    candidate: &CandidatesEntry::new().with_title(String::from("Fictional")),
                },
                UpdatesEntry {
                    name: "[Placeholder] Test - 12 [1080p]",
                    path: Path::new("fake/path/[Placeholder] Test - 12 [1080p].mkv"),
                    episode: 12,
                    candidate: &CandidatesEntry::new().with_title(String::from("Test")),
                },
            ]));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let files = generate_files();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Not defined")),
            ];
            
            // operation
            
            let output = UpdatesEntries::get(&files, &candidates);
            
            // control
            
            let output: Vec<UpdatesEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let files = Vec::new();
            
            let candidates = Vec::new();
            
            // operation
            
            let output = UpdatesEntries::get(&files, &candidates);
            
            // control
            
            let output: Vec<UpdatesEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty_files() {
            // setup
            
            let files = Vec::new();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Fictional")),
                &CandidatesEntry::new().with_title(String::from("Not defined")),
                &CandidatesEntry::new().with_title(String::from("Test")),
            ];
            
            // operation
            
            let output = UpdatesEntries::get(&files, &candidates);
            
            // control
            
            let output: Vec<UpdatesEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty_candidates() {
            // setup
            
            let files = generate_files();
            
            let candidates = Vec::new();
            
            // operation
            
            let output = UpdatesEntries::get(&files, &candidates);
            
            // control
            
            let output: Vec<UpdatesEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        fn generate_files() -> Vec<(&'static str, &'static Path)> {
            Vec::from([
                (
                    "[Imaginary] Fictional - 9 [480p]",
                    Path::new("fake/path/[Imaginary] Fictional - 9 [480p].mp4"),
                ),
                (
                    "[Imaginary] Fictional - 10 [480p]",
                    Path::new("fake/path/[Imaginary] Fictional - 10 [480p].mp4"),
                ),
                (
                    "[Not-present] Other - 17 [720p]",
                    Path::new("fake/path/[Not-present] Other - 17 [720p].mkv"),
                ),
                (
                    "[Placeholder] Test - 12 [1080p]",
                    Path::new("fake/path/[Placeholder] Test - 12 [1080p].mkv"),
                ),
            ])
        }
        
    }
    
}
