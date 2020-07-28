extern crate bindgen;

use std::env;
use std::path::PathBuf;

// First AAudio supported release of Android
const ANDROID_VERSION: usize = 28;

fn main() {
    println!("cargo:rerun-if-env-changed=ANDROID_HOME");

    let android_home = PathBuf::from(env::var("ANDROID_HOME").expect("ANDROID_HOME must be set"));
    
    let android_ndk = if !android_home.join("ndk-bundle").exists() {
        PathBuf::from(env::var("ANDROID_NDK_HOME").expect("Android NDK must be installed"))
    } else {
        android_home.join("ndk-bundle")
    };

    let android_sysroot = android_ndk.join("sysroot");
    let android_include = android_ndk.join("sysroot/usr/include");
    
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
    
    let host_path = std::fs::read_dir(android_ndk.join("toolchains/llvm/prebuilt"))
        .expect("Path to prebuilt libs in ndk")
        .nth(0)
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();

    let android_prebuilt_include = android_ndk.join(
        &format!("toolchains/llvm/prebuilt/{}/sysroot/usr/lib/{}/{}", host_path, target, ANDROID_VERSION)
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
