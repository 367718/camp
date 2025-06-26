pub struct FormData<'r> {
    boundary: &'r [u8],
    content: &'r [u8],
}

pub struct FormDataIterator<'r> {
    boundary: &'r [u8],
    content: &'r [u8],
}

impl<'r> FormData<'r> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(boundary: &'r [u8], content: &'r [u8]) -> Self {
        FormData {
            boundary,
            content,
        }
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn iter(&self) -> FormDataIterator {
        FormDataIterator {
            boundary: self.boundary,
            content: self.content,
        }
    }
    
    pub fn contains_key(&self, query: &[u8]) -> bool {
        self.iter()
            .any(|(key, _)| key == query)
    }
    
    pub fn get<'g>(&'g self, query: &[u8]) -> impl Iterator<Item = &'g [u8]> {
        self.iter()
            .filter(move |(key, _)| *key == query)
            .map(|(_, value)| value)
    }
    
}

impl<'i> IntoIterator for &'i FormData<'i> {
    
    type IntoIter = FormDataIterator<'i>;
    type Item = (&'i [u8], &'i [u8]);
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
    
}

impl<'r> Iterator for FormDataIterator<'r> {
    
    type Item = (&'r [u8], &'r [u8]);
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // example
        
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #1"
        // 
        // placeholder value #1
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #2"
        // 
        // placeholder value #2
        // -----------------------------9999999999999999999999999999--
        
        while let Some(param) = chikuwa::subslice_range(self.content, self.boundary, self.boundary) {
            
            let item = build_pair(&self.content[param.start..param.end]);
            self.content = &self.content[param.end..];
            
            if item.is_some() {
                return item;
            }
            
        }
        
        None
        
    }
    
}

fn build_pair(param: &[u8]) -> Option<(&[u8], &[u8])> {
    
    // example
    
    // Content-Disposition: form-data; name="placeholder"
    // 
    // placeholder value
    // --
    
    let data = chikuwa::subslice_range(param, b"Content-Disposition: form-data; name=\"", b"\"\r\n\r\n")?;
    
    let key = &param[data.start..data.end];
    let value = param[data.end..][5..].strip_suffix(b"\r\n--")?;
    
    Some((key, value))
    
}
