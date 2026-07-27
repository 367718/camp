use std::io::{ self, Write };

use super::ListEntry;

pub fn serialize(writer: &mut impl Write, entry: &ListEntry) -> io::Result<()> {
    // -------------------- header --------------------
    
    let tag_size = u16::try_from(entry.tag.len())
        .expect("Tag size exceeds the maximum value supported");
    
    let value_bytes = entry.value.to_le_bytes();
    let tag_size_bytes = tag_size.to_le_bytes();
    
    let header = [
        value_bytes[0],
        value_bytes[1],
        tag_size_bytes[0],
        tag_size_bytes[1],
    ];
    
    writer.write_all(&header)?;
    
    // -------------------- tag --------------------
    
    writer.write_all(entry.tag)?;
    
    // -------------------- response --------------------
    
    Ok(())
}

pub fn deserialize(data: &[u8]) -> Option<(ListEntry<'_>, &[u8])> {
    // -------------------- header --------------------
    
    let (header, rest) = data.split_first_chunk::<4>()?;
    
    let value = u16::from_le_bytes([header[0], header[1]]);
    let tag_size = u16::from_le_bytes([header[2], header[3]]);
    
    // -------------------- tag --------------------
    
    let (tag, rest) = rest.split_at_checked(usize::from(tag_size))?;
    
    // -------------------- response --------------------
    
    Some((ListEntry { tag, value }, rest))
}

#[cfg(test)]
mod tests {
    
    // roundtrip
    // no_content
    // empty_tag
    // no_tag_size
    // no_tag
    // no_value
    // low_tag_size
    // high_tag_size
    
    use super::*;
    
    #[test]
    fn roundtrip() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        // operation
        
        serialize(&mut content, &entry).unwrap();
        
        let output = deserialize(&content);
        
        // control
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, entry);
    }
    
    #[test]
    fn no_content() {
        // setup
        
        let content = Vec::new();
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn empty_tag() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"",
            value: 89,
        };
        
        // operation
        
        serialize(&mut content, &entry).unwrap();
        
        let output = deserialize(&content);
        
        // control
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, entry);
    }
    
    #[test]
    fn no_tag_size() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&entry.value.to_le_bytes());
        content.extend_from_slice(entry.tag);
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn no_tag() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&entry.value.to_le_bytes());
        content.extend_from_slice(&entry.tag.len().to_le_bytes());        
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn no_value() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"\x03\0aceholder",
            value: 89,
        };
        
        content.extend_from_slice(&(entry.tag.len() as u16).to_le_bytes());
        content.extend_from_slice(entry.tag);
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, ListEntry {
            tag: b"ace",
            value: 11,
        });
    }
    
    #[test]
    fn low_tag_size() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&entry.value.to_le_bytes());
        content.extend_from_slice(&(entry.tag.len() as u16 - 1).to_le_bytes());
        content.extend_from_slice(entry.tag);
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, ListEntry {
            tag: b"placeholde",
            value: 89,
        });
    }
    
    #[test]
    fn high_tag_size() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&entry.value.to_le_bytes());
        content.extend_from_slice(&(entry.tag.len() as u16 + 1).to_le_bytes());
        content.extend_from_slice(entry.tag);
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
}
