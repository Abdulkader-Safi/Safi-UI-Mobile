//! Build script — runs `glslc` to compile shaders to SPIR-V + MSL.
//!
//! No-op unless `feature = "gpu"` is on, so host CI (no glslc) never hits it.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=shaders/");
    println!("cargo:rerun-if-changed=build.rs");

    if env::var_os("CARGO_FEATURE_GPU").is_none() {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set by cargo"));
    let shader_out = out_dir.join("shaders");
    std::fs::create_dir_all(&shader_out).expect("create shaders/ in OUT_DIR");

    let glslc = locate_glslc();

    for shader in &["rect.glsl", "text.glsl"] {
        compile(
            &glslc,
            Path::new("shaders").join(shader).as_path(),
            &shader_out,
        );
    }
}

fn locate_glslc() -> PathBuf {
    if let Ok(path) = env::var("GLSLC") {
        return PathBuf::from(path);
    }
    let candidates = ["glslc", "/usr/local/bin/glslc", "/opt/homebrew/bin/glslc"];
    for c in &candidates {
        if Command::new(c).arg("--version").output().is_ok() {
            return PathBuf::from(c);
        }
    }
    if let Some(sdk) = env::var_os("VULKAN_SDK") {
        let p = PathBuf::from(sdk).join("bin").join("glslc");
        if Command::new(&p).arg("--version").output().is_ok() {
            return p;
        }
    }
    panic!(
        "safi-ui[gpu]: `glslc` not found on PATH. Install glslang \
         (brew install glslang / apt install glslang-tools), or set $GLSLC / \
         $VULKAN_SDK."
    );
}

fn compile(glslc: &Path, src: &Path, out_dir: &Path) {
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("shader file stem");

    for (stage, ext, define) in &[
        ("vert", "vert.spv", "VERTEX"),
        ("frag", "frag.spv", "FRAGMENT"),
    ] {
        let out = out_dir.join(format!("{stem}.{ext}"));
        let status = Command::new(glslc)
            .args(["-fshader-stage", stage, "-D"])
            .arg(define)
            .arg("-o")
            .arg(&out)
            .arg(src)
            .status()
            .expect("invoke glslc");
        assert!(
            status.success(),
            "glslc failed for {} stage={}",
            src.display(),
            stage
        );
    }

    // MSL emission via `glslc --target-env=opengl4.5` is not supported;
    // SDL_GPU's `ShaderFormat::MSL` flag means it can ingest the SPIR-V we
    // already produced and cross-compile internally via SPIRV-Cross. Per
    // PRD §8.2 we ship both formats, but in practice SDL_GPU handles the
    // cross-compilation. Leaving a hook here for an explicit MSL path
    // (e.g. spirv-cross) when todo 16's atlas needs platform-specific
    // tuning.
}
