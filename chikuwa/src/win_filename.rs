use std::iter;

const UTF16_CODEUNITS_LIMIT: usize = 255;

const RESERVED_CHARACTERS: &[char] = &[
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

const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "COM¹", "COM²", "COM³",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    "LPT¹", "LPT²", "LPT³",
    "CONIN$", "CONOUT$",
];

pub fn win_filename(content: &str) -> Option<String> {
    // -------------------- reserved characters --------------------
    
    // strip reserved characters defined in https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
    // `is_ascii_control` includes `0x7F` (DEL), which is not actually reserved by Windows; stripped anyway as non-printable
    
    let filtered = content
        .chars()
        .filter(|character| ! RESERVED_CHARACTERS.contains(character) && ! character.is_ascii_control());
    
    // enforce codeunits limit and trim trailing whitespace/dots
    let mut result = collect_limited_and_trimmed(filtered);
    
    // -------------------- reserved names --------------------
    
    // escape reserved device names defined in:
    // https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
    // https://learn.microsoft.com/en-us/windows/win32/devnotes/rtlisdosdevicename_u
    
    let stem_trimmed = result
        .split_once('.')
        .map_or(result.as_str(), |(left, _)| left)
        .trim_end_matches(' ');
    
    if RESERVED_NAMES.iter().any(|name| name.eq_ignore_ascii_case(stem_trimmed)) {
        // enforce codeunits limit and trim trailing whitespace/dots
        result = collect_limited_and_trimmed(iter::once('_').chain(result.chars()));
    }
    
    // -------------------- response --------------------
    
    if result.is_empty() {
        return None;
    }
    
    Some(result)
}

fn collect_limited_and_trimmed(characters: impl Iterator<Item = char>) -> String {
    let (_, upper_bound) = characters.size_hint();
    let capacity = upper_bound
        .unwrap_or(0)
        // a character that takes 1 UTF-16 codeunit can take up to 3 UTF-8 bytes
        .min(UTF16_CODEUNITS_LIMIT * 3);
    
    let mut result = String::with_capacity(capacity);
    
    // -------------------- codeunit limit --------------------
    
    // each component of a path on NTFS cannot exceed 255 UTF-16 code units (WCHARs)
    
    let mut utf16_len = 0;
    
    for character in characters {
        
        utf16_len += character.len_utf16();
        
        if utf16_len > UTF16_CODEUNITS_LIMIT {
            break;
        }
        
        result.push(character);
        
    }
    
    // -------------------- trailing whitespace or dot --------------------
    
    // filenames must not end with a space or period
    
    let trimmed_len = result
        .trim_end_matches([' ', '.'])
        .len();
    
    result.truncate(trimmed_len);
    
    // -------------------- response --------------------
    
    result
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod general {
        
        // clean
        // extension
        // empty
        // dots
        // dots_and_whitespace
        // extension_with_dots_and_whitespace
        // non_ascii_trailing_whitespace_preserved
        
        use super::*;
        
        #[test]
        fn clean() {
            // setup
            
            let content = "placeholder";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("placeholder")));
        }
        
        #[test]
        fn extension() {
            // setup
            
            let content = "placeholder.txt";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("placeholder.txt")));
        }
        
        #[test]
        fn empty() {
            // setup
            
            let content = "";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn dots() {
            // setup
            
            let content = "..";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn dots_and_whitespace() {
            // setup
            
            let content = " . . ";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn extension_with_dots_and_whitespace() {
            // setup
            
            let content = "placeholder.txt . . ";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("placeholder.txt")));
        }
        
        #[test]
        fn non_ascii_trailing_whitespace_preserved() {
            // setup
            
            let content = "placeholder\u{00A0}";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("placeholder\u{00A0}")));
        }
        
    }
    
    mod characters {
        
        // reserved
        // mix
        // control
        // many_leading
        // exposed_reserved_name
        // exposed_trailing_dots_and_whitespace
        
        use super::*;
        
        #[test]
        fn reserved() {
            // setup
            
            let content = "<>:\"/\\|?*\x00\x1f";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn mix() {
            // setup
            
            let content = "a<b:c>d";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("abcd")));
        }
        
        #[test]
        fn control() {
            // setup
            
            let content = "a\tb\nc\x00d\x7fe";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("abcde")));
        }
        
        #[test]
        fn many_leading() {
            // setup
            
            let content = format!("{}file.txt", "<".repeat(500));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some(String::from("file.txt")));
        }
        
        #[test]
        fn exposed_reserved_name() {
            // setup
            
            let content = "C<O>N.txt";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_CON.txt")));
        }
        
        #[test]
        fn exposed_trailing_dots_and_whitespace() {
            // setup
            
            let content = "placeholder . <";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("placeholder")));
        }
        
    }
    
    mod names {
        
        // reserved
        // non_reserved
        // non_reserved_con_number
        // extension
        // superscript_and_extension
        // trailing_whitespace_and_extension
        // trailing_dot
        // leading_dot
        // leading_whitespace_preserved
        
        use super::*;
        
        #[test]
        fn reserved() {
            // setup
            
            let content = "con";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_con")));
        }
        
        #[test]
        fn non_reserved() {
            // setup
            
            let content = "console";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("console")));
        }
        
        #[test]
        fn non_reserved_con_number() {
            // setup
            
            let content = "CON1";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("CON1")));
        }
        
        #[test]
        fn extension() {
            // setup
            
            let content = "CONOUT$.txt";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_CONOUT$.txt")));
        }
        
        #[test]
        fn superscript_and_extension() {
            // setup
            
            let content = "com¹.txt";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_com¹.txt")));
        }
        
        #[test]
        fn trailing_whitespace_and_extension() {
            // setup
            
            let content = "CON .txt";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_CON .txt")));
        }
        
        #[test]
        fn trailing_dot() {
            // setup
            
            let content = "CON.";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from("_CON")));
        }
        
        #[test]
        fn leading_dot() {
            // setup
            
            let content = ".CON";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from(".CON")));
        }
        
        #[test]
        fn leading_whitespace_preserved() {
            // setup
            
            let content = " CON";
            
            // operation
            
            let output = win_filename(content);
            
            // control
            
            assert_eq!(output, Some(String::from(" CON")));
        }
        
    }
    
    mod truncation {
        
        // exact_limit
        // limit_exceeded
        // reserved_name_at_limit
        // surrogate_pair_fitting_at_limit
        // surrogate_pair_boundary
        // reserved_name_retruncation_at_surrogate_pair
        // exposed_trailing_dot
        // exposed_reserved_name
        
        use super::*;
        
        #[test]
        fn exact_limit() {
            // setup
            
            let content = "€".repeat(255);
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some("€".repeat(255)));
        }
        
        #[test]
        fn limit_exceeded() {
            // setup
            
            let content = "€".repeat(256);
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some("€".repeat(255)));
        }
        
        #[test]
        fn reserved_name_at_limit() {
            // setup
            
            let content = format!("CON.{}", "a".repeat(251));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some(format!("_CON.{}", "a".repeat(250))));
        }
        
        #[test]
        fn surrogate_pair_fitting_at_limit() {
            // setup
            
            let content = format!("{}🔌", "a".repeat(253));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some(format!("{}🔌", "a".repeat(253))));
        }
        
        #[test]
        fn surrogate_pair_boundary() {
            // setup
            
            let content = format!("{}🔌", "a".repeat(254));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some("a".repeat(254)));
        }
        
        #[test]
        fn reserved_name_retruncation_at_surrogate_pair() {
            // setup
            
            let content = format!("CON.{}🔌", "a".repeat(249));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some(format!("_CON.{}", "a".repeat(249))));
        }
        
        #[test]
        fn exposed_trailing_dot() {
            // setup
            
            let content = format!("{}.b", "a".repeat(254));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some("a".repeat(254)));
        }
        
        #[test]
        fn exposed_reserved_name() {
            // setup
            
            let content = format!("CON{}a", " ".repeat(252));
            
            // operation
            
            let output = win_filename(&content);
            
            // control
            
            assert_eq!(output, Some(String::from("_CON")));
        }
        
    }
    
}
