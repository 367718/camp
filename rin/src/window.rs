use std::{
    error::Error,
    fs::File,
    io::{ Write, BufWriter },
};

use crate::Config;

pub struct Window {
    maximized: bool,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
}

pub enum DimensionError {
    ZeroOrLess,
}

pub enum CoordinateError {
    Less,
}

impl Window {
    
    pub const DEFAULT_MAXIMIZED: bool = false;
    pub const DEFAULT_WIDTH: i32 = 780;
    pub const DEFAULT_HEIGHT: i32 = 850;
    pub const DEFAULT_X: i32 = 25;
    pub const DEFAULT_Y: i32 = 25;
    
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            maximized: Self::DEFAULT_MAXIMIZED,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            x: Self::DEFAULT_X,
            y: Self::DEFAULT_Y,
        }
    }
    
    pub fn serialize(&self, writer: &mut BufWriter<&File>) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "window.maximized = {}", self.maximized)?;
        writeln!(writer, "window.width = {}", self.width)?;
        writeln!(writer, "window.height = {}", self.height)?;
        writeln!(writer, "window.x = {}", self.x)?;
        writeln!(writer, "window.y = {}", self.y)?;
        
        Ok(())
    }
    
    pub fn deserialize(content: &[u8]) -> Result<(Self, bool), Box<dyn Error>> {
        let mut corrected = false;
        
        let maximized = Config::get_value(content, b"window.maximized")?;
        let width = Config::get_value(content, b"window.width")?;
        let height = Config::get_value(content, b"window.height")?;
        let x = Config::get_value(content, b"window.x")?;
        let y = Config::get_value(content, b"window.y")?;
        
        let mut window = Window {
            maximized: maximized == "true",
            width: width.parse().unwrap_or(-1),
            height: height.parse().unwrap_or(-1),
            x: x.parse().unwrap_or(-1),
            y: y.parse().unwrap_or(-1),
        };
        
        // width
        if Self::validate_dimension(window.width).is_err() {
            window.width = Self::DEFAULT_WIDTH;
            corrected = true;
        }
        
        // height
        if Self::validate_dimension(window.height).is_err() {
            window.height = Self::DEFAULT_HEIGHT;
            corrected = true;
        }
        
        // x
        if Self::validate_coordinate(window.x).is_err() {
            window.x = Self::DEFAULT_X;
            corrected = true;
        }
        
        // y
        if Self::validate_coordinate(window.y).is_err() {
            window.y = Self::DEFAULT_Y;
            corrected = true;
        }
        
        Ok((window, corrected))
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn maximized(&self) -> bool {
        self.maximized
    }
    
    pub fn width(&self) -> i32 {
        self.width
    }
    
    pub fn height(&self) -> i32 {
        self.height
    }
    
    pub fn x(&self) -> i32 {
        self.x
    }
    
    pub fn y(&self) -> i32 {
        self.y
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn set_maximized(&mut self, maximized: bool) -> bool {
        if self.maximized == maximized {
            return false;
        }
        
        self.maximized = maximized;
        
        true
    }
    
    pub fn set_width(&mut self, width: i32) -> Result<bool, Box<dyn Error>> {
        if self.width == width {
            return Ok(false);
        }
        
        Self::check_dimension(width, "Window width")?;
        
        self.width = width;
        
        Ok(true)
    }
    
    pub fn set_height(&mut self, height: i32) -> Result<bool, Box<dyn Error>> {
        if self.height == height {
            return Ok(false);
        }
        
        Self::check_dimension(height, "Window height")?;
        
        self.height = height;
        
        Ok(true)
    }
    
    pub fn set_x(&mut self, x: i32) -> Result<bool, Box<dyn Error>> {
        if self.x == x {
            return Ok(false);
        }
        
        Self::check_coordinate(x, "Window x")?;
        
        self.x = x;
        
        Ok(true)
    }
    
    pub fn set_y(&mut self, y: i32) -> Result<bool, Box<dyn Error>> {
        if self.y == y {
            return Ok(false);
        }
        
        Self::check_coordinate(y, "Window y")?;
        
        self.y = y;
        
        Ok(true)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_dimension(dimension: i32, field: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_dimension(dimension) {
            match error {
                DimensionError::ZeroOrLess => return Err([field, ": cannot be less than or equal to zero"].concat().into()),
            }
        }
        
        Ok(())
    }
    
    fn check_coordinate(coordinate: i32, field: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_coordinate(coordinate) {
            match error {
                CoordinateError::Less => return Err([field, ": cannot be less than zero"].concat().into()),
            }
        }
        
        Ok(())
    }
    
    pub fn validate_dimension(dimension: i32) -> Result<(), DimensionError> {
        if dimension <= 0 {
            return Err(DimensionError::ZeroOrLess);
        }
        
        Ok(())
    }
    
    pub fn validate_coordinate(coordinate: i32) -> Result<(), CoordinateError> {
        if coordinate < 0 {
            return Err(CoordinateError::Less);
        }
        
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod maximized {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_maximized(true);
            
            // control
            
            assert_eq!(output, true);
            
            assert_eq!(window.maximized(), true);
            
            assert_ne!(window.maximized(), Window::DEFAULT_MAXIMIZED);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_maximized(Window::DEFAULT_MAXIMIZED);
            
            // control
            
            assert_eq!(output, Window::DEFAULT_MAXIMIZED);
            
            assert_eq!(window.maximized(), Window::DEFAULT_MAXIMIZED);
        }
        
    }
    
    mod width {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let width = Window::DEFAULT_WIDTH;
            
            // operation
            
            let output = Window::validate_dimension(width);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_width(150);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(window.width(), 150);
            
            assert_ne!(window.width(), Window::DEFAULT_WIDTH);
        }
        
        #[test]
        fn invalid_zero() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_width(0);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.width(), Window::DEFAULT_WIDTH);
        }
        
        #[test]
        fn invalid_less() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_width(-150);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.width(), Window::DEFAULT_WIDTH);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_width(Window::DEFAULT_WIDTH);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(window.width(), Window::DEFAULT_WIDTH);
        }
        
    }
    
    mod height {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let height = Window::DEFAULT_HEIGHT;
            
            // operation
            
            let output = Window::validate_dimension(height);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_height(80);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(window.height(), 80);
            
            assert_ne!(window.height(), Window::DEFAULT_HEIGHT);
        }
        
        #[test]
        fn invalid_zero() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_height(0);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.height(), Window::DEFAULT_HEIGHT);
        }
        
        #[test]
        fn invalid_less() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_height(-150);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.height(), Window::DEFAULT_HEIGHT);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_height(Window::DEFAULT_HEIGHT);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(window.height(), Window::DEFAULT_HEIGHT);
        }
        
    }
    
    mod x {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let x = Window::DEFAULT_X;
            
            // operation
            
            let output = Window::validate_coordinate(x);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_x(100);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(window.x(), 100);
            
            assert_ne!(window.x(), Window::DEFAULT_X);
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_x(-100);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.x(), Window::DEFAULT_X);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_x(Window::DEFAULT_X);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(window.x(), Window::DEFAULT_X);
        }
        
    }
    
    mod y {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let y = Window::DEFAULT_Y;
            
            // operation
            
            let output = Window::validate_coordinate(y);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_y(30);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(window.y(), 30);
            
            assert_ne!(window.y(), Window::DEFAULT_Y);
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_y(-30);
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(window.y(), Window::DEFAULT_Y);
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut window = Window::new();
            
            // operation
            
            let output = window.set_y(Window::DEFAULT_Y);
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(window.y(), Window::DEFAULT_Y);
        }
        
    }
    
}
