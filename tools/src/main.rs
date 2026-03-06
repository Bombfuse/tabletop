use std::time::{Duration, Instant};

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Tools",
        native_options,
        Box::new(|cc| Ok(Box::new(ToolsApp::new(cc)))),
    )
}

struct ToolsApp {
    last_action: Option<&'static str>,
    auto_close_deadline: Option<Instant>,
}

impl ToolsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let auto_close_deadline = read_autoclose_deadline_from_env();
        Self {
            last_action: None,
            auto_close_deadline,
        }
    }
}

impl eframe::App for ToolsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If AUTOCLOSE_TIMEOUT is set, close after that many seconds.
        if let Some(deadline) = self.auto_close_deadline {
            if Instant::now() >= deadline {
                // eframe 0.30: request the window/viewport to close via egui.
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            // Keep repainting so the countdown progresses even when idle.
            let remaining = deadline.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(remaining.min(Duration::from_millis(100)));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tools");
            ui.add_space(8.0);

            if let Some(deadline) = self.auto_close_deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                ui.label(format!(
                    "Auto-close in: {:.1}s (set via AUTOCLOSE_TIMEOUT)",
                    remaining.as_secs_f32()
                ));
                ui.add_space(8.0);
            } else {
                ui.label("Choose an option:");
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Create Card").clicked() {
                    self.last_action = Some("Create Card");
                }
                if ui.button("Create Campaign").clicked() {
                    self.last_action = Some("Create Campaign");
                }
            });

            ui.add_space(12.0);

            if let Some(action) = self.last_action {
                ui.separator();
                ui.label(format!("Last action: {action}"));
            }
        });
    }
}

fn read_autoclose_deadline_from_env() -> Option<Instant> {
    let raw = std::env::var("AUTOCLOSE_TIMEOUT").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Accept integers or floats (seconds).
    let seconds: f32 = match trimmed.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };

    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }

    Some(Instant::now() + Duration::from_secs_f32(seconds))
}
