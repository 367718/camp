use std::{
    mem,
    os::raw::*,
};

extern "system" {
        
    // https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime
    fn GetLocalTime(
        lpsystemtime: *mut Systemtime,
    );
    
}

#[repr(C)]
#[allow(non_snake_case)]
// https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime
struct Systemtime {
    wYear: c_ushort,
    wMonth: c_ushort,
    wDayOfWeek: c_ushort,
    wDay: c_ushort,
    wHour: c_ushort,
    wMinute: c_ushort,
    wSecond: c_ushort,
    wMilliseconds: c_ushort,
}

pub fn current_date() -> String {
    let mut st = unsafe {
        
        mem::zeroed::<Systemtime>()
        
    };
    
    unsafe {
        
        GetLocalTime(
            &mut st,
        );
        
    }
    
    format!("{:04}{:02}{:02}", st.wYear, st.wMonth, st.wDay)
}
