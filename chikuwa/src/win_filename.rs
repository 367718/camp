use std::{
    ffi::OsString,
};

const DISALLOWED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const DISALLOWED_CHARACTERS: &[char] = &[
    '"', // double quotes
    '*', // asterisk
    '/', // slash
    ':', // colon
    '<', // less than
    '>', // greater than
    '?', // question mark
    '\\', // backslash
    '|', // vertical bar
];

pub fn win_filename(base: &str) -> Option<OsString> {
    if DISALLOWED_NAMES.contains(&base) {
        return None;
    }
    
    let clean: String = base
        .chars()
        .filter(|character| ! character.is_ascii_control())
        .filter(|character| ! DISALLOWED_CHARACTERS.contains(character))
        .collect();
    
    if clean.is_empty() {
        return None;
    }
    
    Some(OsString::from(clean))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn no_replacement() {
        // setup
        
        let content = "[Example] Placeholder - 17 (720p) [83538700].mkv";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("[Example] Placeholder - 17 (720p) [83538700].mkv"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn with_directory() {
        // setup
        
        let content = "madeup/[Example] Placeholder - 17 (720p) [83538700].mkv";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("madeup[Example] Placeholder - 17 (720p) [83538700].mkv"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn single() {
        // setup
        
        let content = "[Example] Place:holder - 17 (720p) [83538700].mkv";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("[Example] Placeholder - 17 (720p) [83538700].mkv"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn multiple() {
        // setup
        
        let content = "[Example] Place:*holde?/r - 1\\7 (720p) [83538700].mkv";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("[Example] Placeholder - 17 (720p) [83538700].mkv"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn empty() {
        // setup
        
        let content = "*:*|";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = None;
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn invalid() {
        // setup
        
        let content = "COM4";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = None;
        
        assert_eq!(output, control);
    }
    
}
