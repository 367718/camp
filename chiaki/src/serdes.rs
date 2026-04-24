use std::{
    io::{ self, Write },
    mem,
};

use super::ListEntry;

const VALUE_SIZE: usize = mem::size_of::<u16>();
const TAG_SIZE_SIZE: usize = mem::size_of::<u16>();
const HEADER_SIZE: usize = VALUE_SIZE + TAG_SIZE_SIZE;

pub fn serialize(writer: &mut impl Write, entry: &ListEntry) -> io::Result<()> {
    // -------------------- header --------------------
    
    let tag_size = u16::try_from(entry.tag.len())
        .expect("Tag size exceeded the maximum value supported");
    
    let mut header = [0; HEADER_SIZE];
    header[0..VALUE_SIZE].copy_from_slice(&entry.value.to_le_bytes());
    header[VALUE_SIZE..HEADER_SIZE].copy_from_slice(&tag_size.to_le_bytes());
    
    writer.write_all(&header)?;
    
    // -------------------- tag --------------------
    
    writer.write_all(entry.tag)?;
    
    // -------------------- response --------------------
    
    Ok(())
}

pub fn deserialize(data: &[u8]) -> Option<(ListEntry<'_>, &[u8])> {
    // -------------------- header --------------------
    
    let (current, rest) = data.split_at_checked(HEADER_SIZE)?;
    let value = u16::from_le_bytes(current[0..VALUE_SIZE].try_into().unwrap());
    let tag_size = u16::from_le_bytes(current[VALUE_SIZE..HEADER_SIZE].try_into().unwrap());
    
    // -------------------- tag --------------------
    
    let (current, rest) = rest.split_at_checked(usize::from(tag_size))?;
    let tag = current;
    
    // -------------------- response --------------------
    
    Some((ListEntry { tag, value }, rest))
}

#[cfg(test)]
mod tests {
    
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
        
        assert!(output.is_some());
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, entry);
    }
    
    #[test]
    fn deserialize_empty() {
        // setup
        
        let content = Vec::new();
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn deserialize_no_tag_size() {
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
    fn deserialize_no_tag() {
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
    fn deserialize_no_value() {
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
        
        assert!(output.is_some());
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, ListEntry {
            tag: b"ace",
            value: 11,
        });
    }
    
    #[test]
    fn deserialize_low_tag_size() {
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
        
        assert!(output.is_some());
        
        let (output, _) = output.unwrap();
        
        assert_eq!(output, ListEntry {
            tag: b"placeholde",
            value: 89,
        });
    }
    
    #[test]
    fn deserialize_high_tag_size() {
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
