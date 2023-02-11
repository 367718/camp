mod pool;

use std::{
    error::Error,
    time::Duration,
};

use pool::Pool;

pub struct HttpClient {
    pool: Pool,
}

impl HttpClient {
    
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
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut client = HttpClient::new(Duration::from_secs(15));
            
            let mut server = mockito::Server::new();
            
            let ok_mock = server.mock("GET", "/ok")
                .with_status(200)
                .with_body("12345")
                .create();
            
            // operation
            
            let output = client.get(&[&server.url(), "/ok"].concat());
            
            // control
            
            ok_mock.assert();
            
            assert!(output.is_ok());
            
            let response = output.unwrap();
            
            assert_eq!(response, b"12345");
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut client = HttpClient::new(Duration::from_secs(15));
            
            let mut server = mockito::Server::new();
            
            let redirect_mock = server.mock("GET", "/redirect")
                .with_status(301)
                .create();
            
            // operation
            
            let output = client.get(&[&server.url(), "/redirect"].concat());
            
            // control
            
            redirect_mock.assert();
            
            assert!(output.is_err());
        }
        
    }
    
}
