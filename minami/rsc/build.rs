use std::{
    env,
    process::Command,
};

fn main() {
    
    let out_dir = env::var("OUT_DIR").unwrap();
    
    Command::new("windres")
        .arg("rsc/app.rc")
        .arg(format!("{}/program.o", out_dir))
        .status()
        .unwrap();
    
    Command::new("gcc-ar")
        .arg("crus")
        .arg("libprogram.a")
        .arg("program.o")
        .current_dir(&out_dir)
        .status()
        .unwrap();
    
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static:+whole-archive=program");
    
}
