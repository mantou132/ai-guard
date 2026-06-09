use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() {
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=frontend/rspack.config.mjs");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");
    println!("cargo:rerun-if-changed=frontend/public");
    println!("cargo:rerun-if-changed=frontend/src");

    if env::var("AI_GUARD_SKIP_FRONTEND_BUILD").as_deref() == Ok("1") {
        println!("cargo:warning=AI_GUARD_SKIP_FRONTEND_BUILD=1, skipping frontend build");
        ensure_placeholder_dist();
        return;
    }

    let frontend = Path::new("frontend");
    if !frontend.join("package.json").exists() {
        ensure_placeholder_dist();
        return;
    }

    let is_release = env::var("PROFILE").as_deref() == Ok("release");
    let force_build = env::var("AI_GUARD_BUILD_FRONTEND").as_deref() == Ok("1");
    let frontend_required =
        env::var("AI_GUARD_FRONTEND_REQUIRED").as_deref() == Ok("1") || is_release;

    if !is_release && !force_build && !frontend_required {
        println!(
            "cargo:warning=skipping frontend build for debug profile; use \
             AI_GUARD_BUILD_FRONTEND=1 to force it"
        );
        ensure_placeholder_dist();
        return;
    }

    if !frontend.join("node_modules").exists() {
        let message =
            "frontend/node_modules is missing; run `just install` before a production build";
        if frontend_required {
            panic!("{message}");
        }
        println!("cargo:warning={message}");
        ensure_placeholder_dist();
        return;
    }

    let pnpm = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    let status = Command::new(pnpm)
        .args(["run", "build"])
        .current_dir(frontend)
        .stdin(Stdio::null())
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) if frontend_required => panic!("frontend build failed with status {status}"),
        Ok(status) => println!("cargo:warning=frontend build failed with status {status}"),
        Err(err) if frontend_required => panic!("failed to run frontend build: {err}"),
        Err(err) => println!("cargo:warning=failed to run frontend build: {err}"),
    }
}

fn ensure_placeholder_dist() {
    let dist = PathBuf::from("frontend").join("dist");
    let index = dist.join("index.html");
    if index.exists() {
        return;
    }
    if let Err(err) = fs::create_dir_all(&dist) {
        println!("cargo:warning=failed to create placeholder frontend/dist: {err}");
        return;
    }
    if let Err(err) = fs::write(
        index,
        r#"<!doctype html><html><body><app-root></app-root><p>Run frontend build to generate UI assets.</p></body></html>"#,
    ) {
        println!("cargo:warning=failed to write placeholder frontend/dist/index.html: {err}");
    }
}
