fn main() {
    
    // -------------------- prevent rerun --------------------
    
    println!("cargo:rerun-if-changed=build.rs");
    
    // -------------------- windows api --------------------
    
    println!("cargo:rustc-link-lib=winhttp");
    
}
