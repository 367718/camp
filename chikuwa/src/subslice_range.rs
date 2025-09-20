use std::ops::Range;

pub fn subslice_range(content: &[u8], left: &[u8], right: &[u8]) -> Option<Range<usize>> {
    let start = find_subslice_ignore_case(content, left)
        .and_then(|index| index.checked_add(left.len()))?;
    
    let end = find_subslice_ignore_case(&content[start..], right)
        .and_then(|index| index.checked_add(start))?;
    
    Some(start..end)
}

fn find_subslice_ignore_case(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    // 'windows' function panics on 0 length
    if needle.is_empty() {
        return None;
    }
    
    haystack.windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod lookup {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let content = b"<a>test</a>";
            
            // operation
            
            let output = subslice_range(content, b"<a>", b"</a>");
            
            // control
            
            assert_eq!(output, Some(3..7));
        }
        
        #[test]
        fn complex() {
            // setup
            
            let content = br#"
                <item>
                    <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123456.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123456</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
            "#;
            
            // operation
            
            let output = subslice_range(content, b"<link>", b"</link>");
            
            // control
            
            assert_eq!(output, Some(134..174));
        }
        
        #[test]
        fn left_and_right_not_present() {
            // setup
            
            let content = br#"
                <item>
                    <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123456.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123456</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
            "#;
            
            // operation
            
            let output = subslice_range(content, b"<comment>", b"</comment>");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn left_not_present() {
            // setup
            
            let content = br#"
                <item>
                    <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123456.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123456</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
            "#;
            
            // operation
            
            let output = subslice_range(content, b"<linkz>", b"</link>");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn right_not_present() {
            // setup
            
            let content = br#"
                <item>
                    <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123456.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123456</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
            "#;
            
            // operation
            
            let output = subslice_range(content, b"<link>", b"</linkz>");
            
            // control
            
            assert_eq!(output, None);
        }
        
    }
    
    #[cfg(test)]
    mod arguments {
        
        use super::*;
        
        #[test]
        fn empty() {
            // setup
            
            let content = b"";
            
            // operation
            
            let output = subslice_range(content, b"", b"");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn empty_content() {
            // setup
            
            let content = b"";
            
            // operation
            
            let output = subslice_range(content, b"<a>", b"</a>");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn empty_left_and_right() {
            // setup
            
            let content = b"<a>test</a>";
            
            // operation
            
            let output = subslice_range(content, b"", b"");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn empty_left() {
            // setup
            
            let content = b"<a>test</a>";
            
            // operation
            
            let output = subslice_range(content, b"", b"</a>");
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn empty_right() {
            // setup
            
            let content = b"<a>test</a>";
            
            // operation
            
            let output = subslice_range(content, b"<a>", b"");
            
            // control
            
            assert_eq!(output, None);
        }
        
    }
    
}
