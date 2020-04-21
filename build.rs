extern crate bindgen;

use std::env;
use std::path::PathBuf;

// First AAudio supported release of Android
const ANDROID_VERSION: usize = 28;

fn main() {
    println!("cargo:rerun-if-env-changed=ANDROID_HOME");

    let android_home = PathBuf::from(env::var("ANDROID_HOME").expect("ANDROID_HOME must be set"));
    
    if !android_home.join("ndk-bundle").exists() {
        panic!("Android NDK must be installed")
    }

    let android_sysroot = android_home.join("ndk-bundle/sysroot");
    let android_include = android_home.join("ndk-bundle/sysroot/usr/include");
    
    let os   = env::var("CARGO_CFG_TARGET_OS").expect("Can't build without os");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Can't build without arch");

    if os.as_str() != "android" { panic!("Only android is supported") } 
    
    let target = match arch.as_str() {
        "arm" => "arm-linux-androideabi",
        "aarch64" => "aarch64-linux-android",
        "x86" => "i686-linux-android",
        "x86_64" => "x86_64-linux-android",
        a => panic!("Unsupported architecture {}", a)
    };

    let android_prebuilt_include = android_home.join(
        &format!("ndk-bundle/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/{}/{}", target, ANDROID_VERSION)
    );
    let aaudio_dylib_path = android_prebuilt_include.join("libaaudio.so");

    println!("cargo:rustc-link-search=native={}", android_prebuilt_include.display());
    println!("cargo:rustc-link-lib=dylib={}", aaudio_dylib_path.display());

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("--target={}", &target))
        .clang_arg(format!("--sysroot={}", android_sysroot.display()))
        .clang_arg(format!("-I{}", android_include.display()))
        .clang_arg(format!("-I{}", android_include.join(target).display()))
        .clang_arg(format!("-I{}", android_include.join("android/versioning.h").display()))
        .header(format!("{}", android_include.join("aaudio/AAudio.h").display()))
        .generate_comments(false)
        .trust_clang_mangling(false)
        .generate()
        .expect("Unable to generate AAudio bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("env variable OUT_DIR not found"));
    bindings
        .write_to_file(out_path.join("aaudio.rs"))
        .expect("Could not write bindings to file");
}