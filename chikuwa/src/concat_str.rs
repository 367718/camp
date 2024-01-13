#[macro_export]
macro_rules! concat_str {
    
    ( ) => { String::new() };
    
    ( $( $component:expr ),+ ) => {{
        
        let mut capacity = 0;
        $( capacity += $component.len(); )+
        
        let mut string = String::with_capacity(capacity);
        $( string.push_str($component); )+
        
        string
        
    }};
    
}
