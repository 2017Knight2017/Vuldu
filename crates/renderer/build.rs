use std::path::PathBuf;

fn main() {
    let mut build = cxx_build::bridge("src/bridge.rs");
    
    build.file("renderer_cpp/src/renderer.cpp")
        .file("renderer_cpp/src/instance.cpp")
        .file("renderer_cpp/src/device.cpp")
        .file("renderer_cpp/src/pipeline.cpp")
        .file("renderer_cpp/src/debug_messenger.cpp")
        .file("renderer_cpp/src/draw.cpp")
        .file("renderer_cpp/src/descriptors.cpp")
        .file("renderer_cpp/src/swapchain.cpp")
        .file("renderer_cpp/src/buffers.cpp")
        .file("renderer_cpp/src/utils.cpp")
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
    } else { 
        build.include("renderer_cpp/third_party");
    }

    let profile = std::env::var("PROFILE").unwrap();
    if profile == "debug" {
        build.define("DEBUG_MODE", None);
    }

    build.compile("vulkan_renderer");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "macos" => {            
            if profile == "debug" {
                println!("cargo:rustc-link-lib=vulkan");
            } else {
                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let moltenvk_path = PathBuf::from(manifest_dir)
                    .join("renderer_cpp")
                    .join("third_party")
                    .join("moltenvk")
                    .join("lib");

                println!("cargo:rustc-link-search=native={}", moltenvk_path.display());
                println!("cargo:rustc-link-lib=static=MoltenVK");

                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=Foundation");
                println!("cargo:rustc-link-lib=framework=QuartzCore");
                println!("cargo:rustc-link-lib=framework=IOKit");
                println!("cargo:rustc-link-lib=framework=IOSurface");
            }
        }
        "windows" => {
            println!("cargo:rustc-link-lib=vulkan-1");
        }
        _ => {
            println!("cargo:rustc-link-lib=vulkan");
        }
    }

    println!("cargo:rerun-if-changed=renderer_cpp/src");
    println!("cargo:rerun-if-changed=renderer_cpp/include");
    println!("cargo:rerun-if-changed=src/bridge.rs");
}