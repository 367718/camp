use core::range::Range;

pub fn delimited_range(content: &[u8], left: &[u8], right: &[u8]) -> Option<Range<usize>> {
    let start = subslice_index(content, left)? + left.len();
    let end = start + subslice_index(&content[start..], right)?;
    
    Some(Range { start, end })
}

fn subslice_index(mut haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let (needle_first, needle_rest) = needle.split_first()?;
    
    let first_lower = needle_first.to_ascii_lowercase();
    let first_upper = needle_first.to_ascii_uppercase();
    
    let mut index = 0;
    
    while haystack.len() >= needle.len() {
        
        // limit search to where a full match is still possible
        let haystack_limit = haystack.len() - needle.len() + 1;
        
        let position = haystack[..haystack_limit]
            .iter()
            .position(|&byte| byte == first_lower || byte == first_upper)?;
        
        index += position;
        haystack = &haystack[position..];
        
        // first byte matches, compare rest
        if haystack[1..needle.len()].eq_ignore_ascii_case(needle_rest) {
            return Some(index);
        }
        
        // skip over matched byte and continue search
        index += 1;
        haystack = &haystack[1..];
        
    }
    
    None
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod subslice_index {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let haystack = b"placeholder";
            let needle = b"holder";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert_eq!(output, Some(5));
        }
        
        #[test]
        fn case_mismatch() {
            // setup
            
            let haystack = b"placeholder";
            let needle = b"HOLDER";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert_eq!(output, Some(5));
        }
        
        #[test]
        fn content_match() {
            // setup
            
            let haystack = b"placeholder";
            let needle = b"placeholder";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert_eq!(output, Some(0));
        }
        
        #[test]
        fn emoji() {
            // setup
            
            let haystack = "pla🔌ceholder🔌";
            let needle = b"HOLDER";
            
            // operation
            
            let output = subslice_index(haystack.as_bytes(), needle);
            
            // control
            
            assert_eq!(output, Some(9));
        }
        
        #[test]
        fn not_contained() {
            // setup
            
            let haystack = b"placeholder";
            let needle = b"HODLER";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let haystack = b"";
            let needle = b"";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn haystack_empty() {
            // setup
            
            let haystack = b"";
            let needle = b"HOLDER";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn needle_empty() {
            // setup
            
            let haystack = b"placeholder";
            let needle = b"";
            
            // operation
            
            let output = subslice_index(haystack, needle);
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
    #[cfg(test)]
    mod delimited_range {
        
        use super::*;
        
        #[cfg(test)]
        mod lookup {
            
            use super::*;
            
            #[test]
            fn simple() {
                // setup
                
                let content = b"<a>test</a>";
                
                // operation
                
                let output = delimited_range(content, b"<a>", b"</a>");
                
                // control
                
                assert_eq!(output, Some(Range { start: 3, end: 7 }));
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
                
                let output = delimited_range(content, b"<link>", b"</link>");
                
                // control
                
                assert_eq!(output, Some(Range { start: 146, end: 186 }));
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
                
                let output = delimited_range(content, b"<comment>", b"</comment>");
                
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
                
                let output = delimited_range(content, b"<linkz>", b"</link>");
                
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
                
                let output = delimited_range(content, b"<link>", b"</linkz>");
                
                // control
                
                assert_eq!(output, None);
            }
            
            #[test]
            fn right_before_left() {
                // setup
                
                let content = b"</a>test<a>";
                
                // operation
                
                let output = delimited_range(content, b"<a>", b"</a>");
                
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
                
                let output = delimited_range(content, b"", b"");
                
                // control
                
                assert_eq!(output, None);
            }
            
            #[test]
            fn empty_content() {
                // setup
                
                let content = b"";
                
                // operation
                
                let output = delimited_range(content, b"<a>", b"</a>");
                
                // control
                
                assert_eq!(output, None);
            }
            
            #[test]
            fn empty_left_and_right() {
                // setup
                
                let content = b"<a>test</a>";
                
                // operation
                
                let output = delimited_range(content, b"", b"");
                
                // control
                
                assert_eq!(output, None);
            }
            
            #[test]
            fn empty_left() {
                // setup
                
                let content = b"<a>test</a>";
                
                // operation
                
                let output = delimited_range(content, b"", b"</a>");
                
                // control
                
                assert_eq!(output, None);
            }
            
            #[test]
            fn empty_right() {
                // setup
                
                let content = b"<a>test</a>";
                
                // operation
                
                let output = delimited_range(content, b"<a>", b"");
                
                // control
                
                assert_eq!(output, None);
            }
            
        }
        
    }
    
}
