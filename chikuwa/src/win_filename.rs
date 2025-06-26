// https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file

use std::{
    ffi::OsString,
    path::Path,
};

const DISALLOWED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "COM¹", "COM²", "COM³",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    "LPT¹", "LPT²", "LPT³",
];

const DISALLOWED_CHARACTERS: &[char] = &[
    '<', // less than
    '>', // greater than
    ':', // colon
    '"', // double quote
    '/', // forward slash
    '\\', // backslash
    '|', // vertical bar or pipe
    '?', // question mark
    '*', // asterisk
];

pub fn win_filename(base: &str) -> Option<OsString> {
    // -------------------- forbidden names --------------------
    
    // Do not use the following reserved names for the name of a file:
    //  CON, PRN, AUX, NUL,
    //  COM1, COM2, COM3, COM4, COM5, COM6, COM7, COM8, COM9,
    //  COM¹, COM², COM³,
    //  LPT1, LPT2, LPT3, LPT4, LPT5, LPT6, LPT7, LPT8, LPT9,
    //  LPT¹, LPT², and LPT³.
    //  Also avoid these names followed immediately by an extension; for example, NUL.txt and NUL.tar.gz are both equivalent to NUL.
    
    let mut first_stage = base;
    
    let file_stem = Path::new(&first_stage)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(first_stage);
    
    if DISALLOWED_NAMES.iter().any(|name| name.eq_ignore_ascii_case(file_stem)) {
        first_stage = &first_stage[file_stem.len()..];
    }
    
    // -------------------- forbidden characters --------------------
    
    // Use any character in the current code page for a name, including Unicode characters and characters in the extended character set (128–255), except for the following:
    //  The following reserved characters:
    //      < (less than)
    //      > (greater than)
    //      : (colon)
    //      " (double quote)
    //      / (forward slash)
    //      \ (backslash)
    //      | (vertical bar or pipe)
    //      ? (question mark)
    //      * (asterisk)
    //  Integer value zero, sometimes referred to as the ASCII NUL character.
    //  Characters whose integer representations are in the range from 1 through 31, except for alternate data streams where these characters are allowed.
    //  Any other character that the target file system does not allow.
    
    let mut second_stage: String = first_stage
        .chars()
        .filter(|character| ! DISALLOWED_CHARACTERS.contains(character) && ! character.is_ascii_control())
        .collect();
    
    // -------------------- whitespace or dot at end --------------------
    
    // Do not end a file or directory name with a space or a period.
    // Although the underlying file system may support such names, the Windows shell and user interface does not.
    
    let trimmed_len = second_stage
        .trim_end_matches(|character: char| character.is_whitespace() || character == '.')
        .len();
    
    second_stage.truncate(trimmed_len);
    
    // -------------------- result --------------------
    
    if second_stage.is_empty() {
        return None;
    }
    
    Some(OsString::from(second_stage))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn no_replacement() {
        // setup
        
        let content = "[Example] Placeholder - 17 (720p) [83538700]";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("[Example] Placeholder - 17 (720p) [83538700]"));
        
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
    fn name_invalid() {
        // setup
        
        let content = "CoM4.txt";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from(".txt"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn dirty_end() {
        // setup
        
        let content = "[Example] Placeholder - 17 (720p) [83538700].mkv.  ...   . . .";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("[Example] Placeholder - 17 (720p) [83538700].mkv"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn double_invalid() {
        // setup
        
        let content = "COM1COM1";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("COM1COM1"));
        
        assert_eq!(output, control);
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let content = "🔌🔌🔌🔌aaaaaa🔌🔌🔌🔌🔌🔌";
        
        // operation
        
        let output = win_filename(content);
        
        // control
        
        let control = Some(OsString::from("🔌🔌🔌🔌aaaaaa🔌🔌🔌🔌🔌🔌"));
        
        assert_eq!(output, control);
    }
    
}
