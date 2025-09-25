#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    app::run::<frontend_gui::Frontend>()
}
