use cruciverbal_tui::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let app = App::new();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
