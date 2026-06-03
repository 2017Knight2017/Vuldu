fn main() {
    let mut build = cxx_build::bridge("src/FFI.rs");
    
    build.file("renderer_cpp/src/renderer.cpp")
        .file("renderer_cpp/src/instance.cpp")
        .file("renderer_cpp/src/device.cpp")
        .file("renderer_cpp/src/pipeline.cpp")
        .file("renderer_cpp/src/debug_messenger.cpp")
        .include("renderer_cpp/include")
        .flag_if_supported("-std=c++20");

    if cfg!(target_env = "msvc") {
        build.flag("/W4");
    } else {
        build.flag("-Wall")
             .flag("-Wextra")
             .flag("-Wpedantic");
    }

    if let Ok(vulkan_sdk) = std::env::var("VULKAN_SDK") {
        build.include(format!("{}/include", vulkan_sdk));
        
        println!("cargo:rustc-link-search=native={}/lib", vulkan_sdk);
    } else { }

    let profile = std::env::var("PROFILE").unwrap();
    if profile == "debug" {
        build.define("DEBUG_MODE", None);
    }

    build.compile("vulkan_renderer");

    println!("cargo:rustc-link-lib=vulkan");
    println!("cargo:rerun-if-changed=renderer_cpp/src");
    println!("cargo:rerun-if-changed=renderer_cpp/include");
    println!("cargo:rerun-if-changed=src/FFI.rs");
}