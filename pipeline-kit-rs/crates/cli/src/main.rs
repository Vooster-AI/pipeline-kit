#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // When `pipeline` is called without any arguments, launch the TUI
    pk_tui::run_app()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e))
}
