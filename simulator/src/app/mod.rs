//! Application module facade.
//!
//! This module groups the `iced::Application` implementation and app wiring.
//! Keeping this separate makes `main.rs` primarily responsible for bootstrapping
//! (e.g., running migrations) and then launching the app.
//!
//! Further split:
//! - `state`: the `Simulator` struct + initialization
//! - `handlers`: side-effecting operations (DB loads/saves, migrations, etc.)
//! - `router`: mapping `Screen` -> page view
//! - `simulator`: `iced::Application` implementation that ties everything together

pub mod handlers;
pub mod router;
pub mod simulator;
pub mod state;

pub use simulator::Simulator;
