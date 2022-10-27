mod pool;

use std::{
    error::Error,
    time::Duration,
};

use pool::Pool;

pub struct Client {
    pool: Pool,
}

impl Client {
    
    // ---------- constructors ----------
    
    
    pub fn new(timeout: Duration) -> Self {
        Self {
            pool: Pool::new(timeout),
        }
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn get<S: AsRef<str>>(&mut self, url: S) -> Result<Vec<u8>, Box<dyn Error>> {
        self.pool.get(url)
    }
    
}

#[cfg(test)]
mod lib {
    
    use super::*;
    
    use std::mem;
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut client = Client::new(Duration::from_secs(15));
            
            let ok_mock = mockito::mock("GET", "/ok")
                .with_status(200)
                .with_body("12345")
                .create();
            
            // operation
            
            let output = client.get(&[&mockito::server_url(), "/ok"].concat());
            
            // control
            
            ok_mock.assert();
            
            assert!(output.is_ok());
            
            let response = output.unwrap();
            
            assert_eq!(response, b"12345");
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut client = Client::new(Duration::from_secs(15));
            
            let redirect_mock = mockito::mock("GET", "/redirect")
                .with_status(301)
                .create();
            
            // operation
            
            let output = client.get(&[&mockito::server_url(), "/redirect"].concat());
            
            // control
            
            redirect_mock.assert();
            
            assert!(output.is_err());
        }
        
    }
    
    #[test]
    fn enforce_64_bit_wide_pointers() {
        // setup
        
        let left_hand = mem::size_of::<usize>();
        let right_hand = mem::size_of::<u64>();
        
        // operation
        
        let output = left_hand == right_hand;
        
        // control
        
        assert!(output);
    }
    
}
