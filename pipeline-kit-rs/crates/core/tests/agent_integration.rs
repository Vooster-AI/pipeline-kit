// Integration tests that exercise external agent CLIs.
// These are behind a feature flag and marked #[ignore] to run only in CI jobs
// that explicitly enable them.

#![cfg(feature = "e2e-cli-tests")]

use std::env;
use tokio::process::Command;

#[tokio::test]
#[ignore]
async fn claude_cli_is_available() {
    // Optional: require API key if STRICT mode enabled
    if env::var("CI").ok().as_deref() != Some("true") {
        // Only enforce in CI contexts
        eprintln!("Skipping strict checks outside CI");
        return;
    }

    let status = Command::new("claude").arg("-h").status().await;
    assert!(status.is_ok(), "failed to spawn claude CLI");
    assert!(
        status.unwrap().success(),
        "claude CLI not available or returned error"
    );
}

#[tokio::test]
#[ignore]
async fn cursor_agent_cli_is_available() {
    if env::var("CI").ok().as_deref() != Some("true") {
        eprintln!("Skipping strict checks outside CI");
        return;
    }

    let status = Command::new("cursor-agent").arg("-h").status().await;
    assert!(status.is_ok(), "failed to spawn cursor-agent CLI");
    assert!(
        status.unwrap().success(),
        "cursor-agent CLI not available or returned error"
    );
}
