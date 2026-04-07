use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");

    let git_sha = std::env::var("GIT_SHA")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(resolve_git_sha);

    println!("cargo:rustc-env=GIT_SHA={}", git_sha);
}

fn resolve_git_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
