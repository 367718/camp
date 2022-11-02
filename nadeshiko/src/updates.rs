use std::path::Path;

use crate::IsCandidate;

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct UpdatesEntry<'f> {
    pub name: &'f str,
    pub path: &'f Path,
    pub episode: u32,
    pub id: u32,
}

const RESULT_VEC_INITIAL_CAPACITY: usize = 50;

pub fn get<'f>(files: &[(&'f str, &'f Path)], candidates: &[impl IsCandidate]) -> Option<Vec<UpdatesEntry<'f>>> {
    let mut result = Vec::with_capacity(RESULT_VEC_INITIAL_CAPACITY);
    
    for (name, path) in files {
        
        if let Some(entry) = build_entry(name, path, candidates) {
            result.push(entry);
        }
        
    }
    
    if ! result.is_empty() {
        return Some(result);
    }
    
    None
}

fn build_entry<'f>(name: &'f str, path: &'f Path, candidates: &[impl IsCandidate]) -> Option<UpdatesEntry<'f>> {
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
        id: u32,
    }
    
    impl IsCandidate for CandidatesEntry {
        
        fn is_relevant(&self, current: &str) -> bool {
            current.contains(&self.title)
        }
        
        fn clean(&self, current: &str) -> String {
            current.replace(&self.title, "")
        }
        
        fn can_download(&self, _episode: u32) -> bool {
            true
        }
        
        fn can_update(&self, _episode: u32) -> bool {
            true
        }
        
        fn id(&self) -> u32 {
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
            
            let output = get(&files, &candidates);
            
            // control
            
            assert!(output.is_some());
            
            let output = output.unwrap();
            
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
            
            let output = get(&files, &candidates);
            
            // control
            
            assert!(output.is_none());
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
                    "[Placeholder] Test - 12 [1080p]",
                    Path::new("fake/path/[Placeholder] Test - 12 [1080p].mkv"),
                ),
                (
                    "[Not-present] Other - 17 [720p]",
                    Path::new("fake/path/[Not-present] Other - 17 [720p].mkv"),
                ),
            ])
        }
        
    }
    
}
