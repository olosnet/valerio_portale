use std::process::Command;

use chrono::Utc;

fn main() {

    let build_time = Utc::now();
    
    // Formatta in diversi modi
    let timestamp = build_time.timestamp().to_string();
    let date = build_time.format("%Y-%m-%d").to_string();
    let time = build_time.format("%H:%M:%S UTC").to_string();
    let datetime = build_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();


    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
    println!("cargo:rustc-env=BUILD_DATE={}", date);
    println!("cargo:rustc-env=BUILD_TIME={}", time);
    println!("cargo:rustc-env=BUILD_DATETIME={}", datetime);

    // Aggiungi hash del commit git (se disponibile)
    if let Ok(output) = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output() 
    {
        let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }
    
    // Aggiungi informazioni sul branch git
    if let Ok(output) = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
    {
        let git_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    } else {
        println!("cargo:rustc-env=GIT_BRANCH=unknown");
    }

}