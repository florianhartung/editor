use core::EventSender;
use std::fs::File;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    file: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let post_init = |event_sender: &EventSender| {
        if let Some(file_to_open) = args.file {
            event_sender.open_file(file_to_open);
        } else {
            let default_file_to_open = "Cargo.toml";
            if File::open(default_file_to_open).is_ok() {
                event_sender.open_file(default_file_to_open.to_owned());
            }
        }
    };

    app::run::<frontend_tui::Frontend>(post_init)
}
