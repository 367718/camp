use std::path::Path;

use crate::IsCandidate;

pub struct UpdatesEntries<'f, T> {
    files: &'f [(&'f str, &'f Path)],
    candidates: &'f [T],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct UpdatesEntry<'f> {
    pub name: &'f str,
    pub path: &'f Path,
    pub episode: i64,
    pub id: i64,
}

impl<'f, T: IsCandidate> UpdatesEntries<'f, T> {
    
    pub fn get(files: &'f [(&'f str, &'f Path)], candidates: &'f [T]) -> Self {
        Self {
            files,
            candidates,
        }
    }
    
}

impl<'f, T: IsCandidate> Iterator for UpdatesEntries<'f, T> {
    
    type Item = UpdatesEntry<'f>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(((name, path), rest)) = self.files.split_first() {
            
            let result = build_entry(name, path, self.candidates);
            self.files = rest;
            
            if result.is_some() {
                return result;
            }
            
        }
        
        None
    }
    
}

fn build_entry<'f, T: IsCandidate>(name: &'f str, path: &'f Path, candidates: &'f [T]) -> Option<UpdatesEntry<'f>> {
    let candidate = candidates.iter()
        .find(|candidate| candidate.is_relevant(name))?;
    
    let episode = crate::extractor::get(&candidate.clean(name))
        .filter(|&episode| candidate.can_update(episode))?;
    
    let id = candidate.id();
    
    Some(UpdatesEntry {
        name,
        path,
        episode,
        id,
    })
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    struct CandidatesEntry {
        title: String,
        id: i64,
    }
    
    impl IsCandidate for CandidatesEntry {
        
        fn is_relevant(&self, current: &str) -> bool {
            current.contains(&self.title)
        }
        
        fn clean(&self, current: &str) -> String {
            current.replace(&self.title, "")
        }
        
        fn can_download(&self, _episode: i64) -> bool {
            true
        }
        
        fn can_update(&self, _episode: i64) -> bool {
            true
        }
        
        fn id(&self) -> i64 {
            self.id
        }
        
    }
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let files = generate_files();
            
            let candidates = [
                CandidatesEntry {
                    title: String::from("Fictional"),
                    id: 15,
                },
                CandidatesEntry {
                    title: String::from("Not defined"),
                    id: 2,
                },
                CandidatesEntry {
                    title: String::from("Test"),
                    id: 10,
                },
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
                    id: 15,
                },
                UpdatesEntry {
                    name: "[Imaginary] Fictional - 10 [480p]",
                    path: Path::new("fake/path/[Imaginary] Fictional - 10 [480p].mp4"),
                    episode: 10,
                    id: 15,
                },
                UpdatesEntry {
                    name: "[Placeholder] Test - 12 [1080p]",
                    path: Path::new("fake/path/[Placeholder] Test - 12 [1080p].mkv"),
                    episode: 12,
                    id: 10,
                },
            ]));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let files = generate_files();
            
            let candidates = [
                CandidatesEntry {
                    title: String::from("Not defined"),
                    id: 2,
                },
            ];
            
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
