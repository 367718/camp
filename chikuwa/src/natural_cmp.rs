use std::{
    cmp::Ordering,
    iter::Peekable,
    str::Chars,
};

pub fn natural_cmp(first: &str, second: &str) -> Ordering {
    let mut fr_chars = first.chars().peekable();
    let mut sd_chars = second.chars().peekable();
    
    while let (Some(fr_curr), Some(sd_curr)) = (fr_chars.peek(), sd_chars.peek()) {
        
        let fr_curr = fr_curr.to_ascii_lowercase();
        let sd_curr = sd_curr.to_ascii_lowercase();
        
        if fr_curr != sd_curr {
            
            if fr_curr.is_numeric() && sd_curr.is_numeric() {
                
                let order = extract_number(&mut fr_chars).cmp(&extract_number(&mut sd_chars));
                
                if order != Ordering::Equal {
                    return order;
                }
                
                fr_chars.next();
                sd_chars.next();
                
                continue;
                
            }
            
            return fr_curr.cmp(&sd_curr);
            
        }
        
        fr_chars.next();
        sd_chars.next();
        
    }
    
    first.len().cmp(&second.len())
}

fn extract_number(chars: &mut Peekable<Chars>) -> u32 {
    let mut number = 0;
    
    while let Some(digit) = chars.peek().and_then(|curr| curr.to_digit(10)) {
        number = number * 10 + digit;
        chars.next();
    }
    
    number
}
