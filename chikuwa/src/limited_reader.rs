use std::io::{ self, Read, Take, ErrorKind };

pub struct LimitedReader<T> {
    inner: Take<T>,
}

impl<T: Read> LimitedReader<T> {
    
    pub fn new(reader: T, limit: u64) -> io::Result<Self> {
        let limit = limit.checked_add(1)
            .ok_or(io::Error::new(ErrorKind::InvalidInput, "Limit value exceeded the maximum value supported"))?;
        
        Ok(Self {
            inner: reader.take(limit),
        })
    }
    
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
    
}

impl<T: Read> Read for LimitedReader<T> {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.inner.read(buf)?;
        
        if self.inner.limit() == 0 {
            return Err(io::Error::from(ErrorKind::QuotaExceeded))
        }
        
        Ok(bytes)
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn exact() {
        // setup
        
        let content = b"abcdefghij";
        let mut buffer = Vec::new();
        let mut reader = LimitedReader::new(content.as_slice(), 10).unwrap();
        
        // operation
        
        let output = reader.read_to_end(&mut buffer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(buffer, content);
    }
    
    #[test]
    fn lower() {
        // setup
        
        let content = b"abcdefghij";
        let mut buffer = Vec::new();
        let mut reader = LimitedReader::new(content.as_slice(), 5).unwrap();
        
        // operation
        
        let output = reader.read_to_end(&mut buffer);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn higher() {
        // setup
        
        let content = b"abcdefghij";
        let mut buffer = Vec::new();
        let mut reader = LimitedReader::new(content.as_slice(), 15).unwrap();
        
        // operation
        
        let output = reader.read_to_end(&mut buffer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(buffer, content);
    }
    
}
