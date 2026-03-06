mod app;

#[cfg(feature = "gui")]
mod gui;

#[cfg(not(feature = "cli"))]
fn main() -> iced::Result {
    gui::run()
}

#[cfg(feature = "cli")]
fn main() -> anyhow::Result<()> {
    app::run()
}
