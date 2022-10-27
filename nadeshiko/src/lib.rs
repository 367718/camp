pub mod downloads;
pub mod updates;
mod extractor;

pub trait IsCandidate {
    
    fn is_relevant(&self, current: &str) -> bool;
    
    fn clean(&self, current: &str) -> String;
    
    fn can_download(&self, episode: u32) -> bool;
    
    fn can_update(&self, episode: u32) -> bool;
    
    fn id(&self) -> u32;
    
}
