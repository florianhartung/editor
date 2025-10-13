fn main() -> anyhow::Result<()> {
    env_logger::init();

    app::run::<frontend_tui::Frontend>()
}
