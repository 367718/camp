// RFC 3986
const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

pub fn percent_encode(value: &str) -> String {
    let to_be_replaced = value.bytes()
        .filter(|byte| ! UNRESERVED.contains(byte))
        .count();
    
    let mut result = String::with_capacity(value.len() + (to_be_replaced * 2));
    
    for byte in value.bytes() {
        
        if ! UNRESERVED.contains(&byte) {
            result.push_str(&format!("%{:X}", byte));
            continue;
        }
        
        result.push(char::from(byte));
        
    }
    
    result
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod percent_encode {
        
        use super::*;
        
        #[test]
        fn numbers() {
            // setup
            
            let value = "123456";
            let control = "123456";
            
            // operation
            
            let output = percent_encode(value);
            
            // control
            
            assert_eq!(output, control);
        }
        
        #[test]
        fn letters() {
            // setup
            
            let value = "abcdefg";
            let control = "abcdefg";
            
            // operation
            
            let output = percent_encode(value);
            
            // control
            
            assert_eq!(output, control);
        }
        
        #[test]
        fn symbols() {
            // setup
            
            let value = "~./\\-";
            let control = "~.%2F%5C-";
            
            // operation
            
            let output = percent_encode(value);
            
            // control
            
            assert_eq!(output, control);
        }
        
        #[test]
        fn emoji() {
            // setup
            
            let value = "❤🎂";
            let control = "%E2%9D%A4%F0%9F%8E%82";
            
            // operation
            
            let output = percent_encode(value);
            
            // control
            
            assert_eq!(output, control);
        }
        
        #[test]
        fn mix() {
            // setup
            
            let value = "😊bcd1~234Ñ5¿6g";
            let control = "%F0%9F%98%8Abcd1~234%C3%915%C2%BF6g";
            
            // operation
            
            let output = percent_encode(value);
            
            // control
            
            assert_eq!(output, control);
        }
        
    }
    
}
