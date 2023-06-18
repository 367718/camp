use std::{
    mem,
    os::raw::*,
};

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
        pub fn GetLocalTime(
            lpsystemtime: *mut Systemtime,
        );
        
    }
    
    #[repr(C)]
    #[allow(non_snake_case)]
    // https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime
    pub struct Systemtime {
        pub wYear: c_ushort,
        pub wMonth: c_ushort,
        pub wDayOfWeek: c_ushort,
        pub wDay: c_ushort,
        pub wHour: c_ushort,
        pub wMinute: c_ushort,
        pub wSecond: c_ushort,
        pub wMilliseconds: c_ushort,
    }
    
}

pub fn current_date() -> String {
    let date = unsafe {
        
        let mut store = mem::zeroed::<ffi::Systemtime>();
        
        ffi::GetLocalTime(
            &mut store,
        );
        
        store
        
    };
    
    format!("{:04}{:02}{:02}", date.wYear, date.wMonth, date.wDay)
}
