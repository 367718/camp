use std::os::raw::*;

unsafe extern "system" {
    
    // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
    pub fn WaitNamedPipeW(
        lpNamedPipeName: *const c_ushort, // LPCWSTR
        nTimeOut: c_ulong, // DWORD
    ) -> c_int;
    
}
