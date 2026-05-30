use std::io::{ self, Read };

pub struct Feed {
    content: Vec<u8>,
}

pub struct FeedEntries<'c> {
    content: &'c [u8],
}

#[cfg_attr(debug_assertions, derive(PartialEq, Debug))]
pub struct FeedEntry<'c> {
    pub title: &'c [u8],
    pub link: &'c [u8],
}

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

impl Feed {
    
    pub fn new(client: &mut akari::Client, url: &str, max_size: u64) -> io::Result<Self> {
        let response = client.get(url)?;
        
        let size = response.content_length()
            .unwrap_or(0)
            .min(max_size);
        
        let mut reader = response.take(size);
        let mut content = Vec::with_capacity(usize::try_from(size).expect("Unsupported platform"));
        
        io::copy(&mut reader, &mut content)?;
        
        Ok(Self { content })
    }
    
    pub fn iter(&self) -> FeedEntries<'_> {
        FeedEntries { content: &self.content }
    }
    
}

impl<'c> IntoIterator for &'c Feed {
    
    type IntoIter = FeedEntries<'c>;
    type Item = FeedEntry<'c>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
    
}

impl<'c> Iterator for FeedEntries<'c> {
    
    type Item = FeedEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // expected structure
        
        // ...
        // <item>
        // ...
        // <title>...</title>
        // ...
        // <link>...</link>
        // ...
        // </item>
        // ...
        
        while let Some(item) = chikuwa::delimited_range(self.content, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let current = &self.content[item];
            self.content = &self.content[item.end..][ITEM_CLOSE_TAG.len()..];
            
            let Some(title) = chikuwa::delimited_range(current, TITLE_OPEN_TAG, TITLE_CLOSE_TAG) else {
                continue;
            };
            
            let Some(link) = chikuwa::delimited_range(current, LINK_OPEN_TAG, LINK_CLOSE_TAG) else {
                continue;
            };
            
            return Some(Self::Item {
                title: &current[title],
                link: &current[link],
            });
            
        }
        
        None
        
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod valid {
        
        use super::*;
        
        #[test]
        fn one() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123456.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123456</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"http://localhost/download/123456.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn three() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123456.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123456</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <title>[Example] Placeholder - 18 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/654321.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/654321</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <title>[Example] Placeholder - 19 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123123.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123123</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"http://localhost/download/123456.torrent",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 18 (720p) [83538700].mkv",
                link: b"http://localhost/download/654321.torrent",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 19 (720p) [83538700].mkv",
                link: b"http://localhost/download/123123.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
            
    }
    
    #[cfg(test)]
    mod malformed {
        
        use super::*;
        
        #[test]
        fn empty_link() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <link></link>
                            <guid isPermaLink="true">http://localhost/view/123456</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"",
            }));
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn nested_items() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 16 (720p) [83538700].mkv</title>
                            <item>
                                <link>http://localhost/download/321321.torrent</link>
                            </item>
                            <guid isPermaLink="true">http://localhost/view/321321</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <item>
                                <link>http://localhost/download/321321.torrent</link>
                            </item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <guid isPermaLink="true">http://localhost/view/321321</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 16 (720p) [83538700].mkv",
                link: b"http://localhost/download/321321.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn multiple_link() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123456.torrent</link>
                            <link>http://localhost/download/1234567890.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123456</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"http://localhost/download/123456.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn no_items() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn barebones() {
            // setup
            
            let content = br#"
                <item>
                    <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123456.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123456</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
                <item>
                    <title>[Example] Placeholder - 18 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/654321.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/654321</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
                <item>
                    <title>[Example] Placeholder - 19 (720p) [83538700].mkv</title>
                    <link>http://localhost/download/123123.torrent</link>
                    <guid isPermaLink="true">http://localhost/view/123123</guid>
                    <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                </item>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"http://localhost/download/123456.torrent",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 18 (720p) [83538700].mkv",
                link: b"http://localhost/download/654321.torrent",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 19 (720p) [83538700].mkv",
                link: b"http://localhost/download/123123.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
        
    }
    
    #[cfg(test)]
    mod invalid {
        
        use super::*;
        
        #[test]
        fn unclosed_item() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                        <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                        <link>http://localhost/download/123456.torrent</link>
                        <guid isPermaLink="true">http://localhost/view/123456</guid>
                        <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), None);
        }
        
        #[test]
        fn missing_link() {
            // setup
            
            let content = br#"
                <rss version="2.0">
                    <channel>
                        <title>Example RSS</title>
                        <description>Example RSS Feed</description>
                        <link>http://localhost/</link>
                        <atom:link href="http://localhost/rss" rel="self" type="application/rss+xml"/>
                        <item>
                            <title>[Example] Placeholder - 17 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123456.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123456</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <title>[Example] Placeholder - 18 (720p) [83538700].mkv</title>
                            <link></link>
                            <guid isPermaLink="true">http://localhost/view/654321</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <title>[Example] Placeholder - 19 (720p) [83538700].mkv</title>
                            <guid isPermaLink="true">http://localhost/view/654321</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                        <item>
                            <title>[Example] Placeholder - 20 (720p) [83538700].mkv</title>
                            <link>http://localhost/download/123123.torrent</link>
                            <guid isPermaLink="true">http://localhost/view/123123</guid>
                            <pubDate>Thu, 01 Jan 1970 00:00:00 -0000</pubDate>
                        </item>
                    </channel>
                </rss>
            "#;
            
            let entries = Feed {
                content: content.to_vec(),
            };
            
            // operation
            
            let mut output = entries.iter();
            
            // control
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 17 (720p) [83538700].mkv",
                link: b"http://localhost/download/123456.torrent",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 18 (720p) [83538700].mkv",
                link: b"",
            }));
            
            assert_eq!(output.next(), Some(FeedEntry {
                title: b"[Example] Placeholder - 20 (720p) [83538700].mkv",
                link: b"http://localhost/download/123123.torrent",
            }));
            
            assert_eq!(output.next(), None);
        }
        
    }
    
}
