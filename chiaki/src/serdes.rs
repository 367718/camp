use std::io::{ self, Write };

use super::{ MEM_SIZE, ListEntry };

pub fn serialize(writer: &mut impl Write, entry: &ListEntry) -> io::Result<()> {
    let tag_size = u64::try_from(entry.tag.len())
        .expect("Tag size exceeded the maximum value supported")
        .to_le_bytes();
    
    let value = entry.value.to_le_bytes();
    
    writer.write_all(&tag_size)?;
    writer.write_all(entry.tag)?;
    writer.write_all(&value)?;
    
    Ok(())
}

pub fn deserialize(data: &[u8]) -> Option<ListEntry<'_>> {
    // -------------------- tag size --------------------
    
    let (current, rest) = data.split_at_checked(MEM_SIZE)?;
    let tag_size = usize::try_from(u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() }))
        .expect("Tag size exceeded the maximum value supported");
    
    // -------------------- tag --------------------
    
    let (current, rest) = rest.split_at_checked(tag_size)?;
    let tag = current;
    
    // -------------------- value --------------------
    
    let current = rest.get(..MEM_SIZE)?;
    let value = u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() });
    
    Some(ListEntry {
        tag,
        value,
    })
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
        
        assert_eq!(output, Some(entry));
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
        
        content.extend_from_slice(entry.tag);
        content.extend_from_slice(&entry.value.to_le_bytes());
        
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
        
        content.extend_from_slice(&entry.tag.len().to_le_bytes());
        content.extend_from_slice(&entry.value.to_le_bytes());
        
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
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&entry.tag.len().to_le_bytes());
        content.extend_from_slice(entry.tag);
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn deserialize_low_tag_size() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&(entry.tag.len() - 1).to_le_bytes());
        content.extend_from_slice(entry.tag);
        content.extend_from_slice(&entry.value.to_le_bytes());
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert_eq!(output, Some(ListEntry {
            tag: b"placeholde",
            value: 22898,
        }));
    }
    
    #[test]
    fn deserialize_high_tag_size() {
        // setup
        
        let mut content = Vec::new();
        
        let entry = ListEntry {
            tag: b"placeholder",
            value: 89,
        };
        
        content.extend_from_slice(&(entry.tag.len() + 1).to_le_bytes());
        content.extend_from_slice(entry.tag);
        content.extend_from_slice(&entry.value.to_le_bytes());
        
        // operation
        
        let output = deserialize(&content);
        
        // control
        
        assert!(output.is_none());
    }
    
}
