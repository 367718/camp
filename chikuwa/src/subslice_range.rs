use std::ops::Range;

pub fn subslice_range(content: &[u8], left: &[u8], right: &[u8]) -> Option<Range<usize>> {
    // windows method panics if given a zero as length
    if left.is_empty() || right.is_empty() {
        return None;
    }
    
    let start = content.windows(left.len())
        .position(|window| window.eq_ignore_ascii_case(left))
        .and_then(|index| index.checked_add(left.len()))?;
    
    let end = content[start..].windows(right.len())
        .position(|window| window.eq_ignore_ascii_case(right))
        .and_then(|index| index.checked_add(start))?;
    
    Some(start..end)
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
