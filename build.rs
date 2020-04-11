extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let android_home = PathBuf::from(env::var("ANDROID_HOME").expect("ANDROID_HOME must be set"));
    let android_sysroot = android_home.join("ndk-bundle/sysroot");
    let android_include = android_home.join("ndk-bundle/sysroot/usr/include");
    let android_prebuilt = android_home.join("ndk-bundle/toolchains/llvm/prebuilt/darwin-x86_64/sysroot");

    println!("cargo:rustc-link-search={}", android_include.display());
    
    let os   = env::var("CARGO_CFG_TARGET_OS").expect("Can't build without os");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Can't build without arch");

    if os.as_str() != "android" { panic!("Only android is supported") } 
    
    let target = match arch.as_str() {
        "arm" => "armv7-linux-androideabi",
        "aarch64" => "aarch64-linux-android",
        "x86" => "i686-linux-android",
        a => panic!("Unsupported architecture {}", a)
    };

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("--target={}", &target))
        .clang_arg(format!("--sysroot={}", android_sysroot.display()))
        .clang_arg(format!("-L{}", android_prebuilt.display()))
        .clang_arg(format!("-I{}", android_include.display()))
        .clang_arg(format!("-I{}/{}", android_include.display(), &target))
        .clang_arg(format!("-include{}/android/versioning.h", android_include.display()))
        .clang_arg("-Iinclude")
        .header(android_include.join("aaudio/AAudio.h").to_str().unwrap())
        .generate_comments(false)
        .trust_clang_mangling(false)
        .generate()
        .expect("Unable to generate AAudio bindings");

    let out_path = PathBuf::from("./src");
    bindings
        .write_to_file(out_path.join("aaudio.rs"))
        .expect("Could not write bindings to file");
}