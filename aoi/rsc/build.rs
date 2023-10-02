use std::{
    env,
    path::Path,
    process::Command,
};

fn main() {
    
    // -------------------- prevent rerun --------------------
    
    println!("cargo:rerun-if-changed=build.rs");
    
    // -------------------- resource file --------------------
    
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out = env::var("OUT_DIR").unwrap();
    
    let rc = Path::new(&root).join("rsc").join("app.rc");
    let res = Path::new(&out).join("app.res");
    
    let result = Command::new("rc")
        .arg(format!("/fo{}", res.display()))
        .arg(rc.as_os_str())
        .status()
        .unwrap();
    
    assert!(result.success(), "Resource file conversion failed");
    
    println!("cargo:rustc-link-arg={}", res.display());
    
    // -------------------- windows api --------------------
    
    // WaitNamedPipeW
    println!("cargo:rustc-link-lib=kernel32");
    
}
