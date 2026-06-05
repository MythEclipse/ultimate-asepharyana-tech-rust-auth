use anyhow::Result;
use rust_auth::bootstrap::Application;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let app = Application::build().await?;
    app.run().await?;
    Ok(())
}
