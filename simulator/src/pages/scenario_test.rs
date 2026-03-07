use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use iced::widget::canvas;

use crate::types::Message;

/// Scenario Test view.
///
/// Renders the loaded hex grid using `iced::widget::canvas` with real hex tiles:
/// - pointy-top hex layout
/// - odd-r row offset (odd `y` rows are shifted by half a hex width)
///
/// Adds:
/// - metadata at the top (grid width/height, tile counts, current pan/zoom)
/// - different colors/icons based on tile metadata
/// - pan + zoom
/// - click-to-select + popup showing tile metadata
pub fn view(
    hex_grid_id: i64,
    loaded_grid: Option<&data::hex_grids::HexGrid>,
    tiles: &[data::hex_grids::HexTile],
    load_error: Option<&str>,
) -> Element<'static, Message> {
    let title = text("Scenario Test")
        .size(44)
        .horizontal_alignment(Horizontal::Center);

    let subtitle = text(format!("Hex Grid ID: {hex_grid_id}"))
        .size(18)
        .horizontal_alignment(Horizontal::Center);

    let mut content = column![title, subtitle]
        .spacing(10)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill);

    if let Some(err) = load_error {
        content = content.push(text(err).size(16).style(iced::theme::Text::Color(
            iced::Color::from_rgb(0.85, 0.2, 0.2),
        )));
    }

    match loaded_grid {
        None => {
            content = content.push(text("Loading grid...").size(18));
        }
        Some(grid) => {
            // Top metadata (requested)
            let total_cells = (grid.width.max(0) as i64) * (grid.height.max(0) as i64);
            let present_tiles = tiles.len() as i64;

            content = content.push(text(&grid.name).size(20)).push(
                row![
                    text(format!("Width: {}", grid.width)).size(16),
                    text(format!("Height: {}", grid.height)).size(16),
                    text(format!("Cells: {}", total_cells)).size(16),
                    text(format!("Persisted tiles: {}", present_tiles)).size(16),
                    text("Controls: drag to pan • wheel to zoom • click hex for details").size(16),
                ]
                .spacing(18)
                .width(Length::Fill),
            );

            // Canvas renderer with pan/zoom + selection + popup
            let view = HexGridCanvas::new(grid.clone(), tiles.to_vec());
            let canvas = canvas::Canvas::new(view)
                .width(Length::Fill)
                .height(Length::Fill);

            content = content.push(canvas);
        }
    }

    let back = button(text("Back").size(18))
        .padding(12)
        .width(Length::Fixed(160.0))
        .on_press(Message::LoadScenario);

    content = content.push(row![back].width(Length::Fill).spacing(12));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}

/// A `canvas::Program` that draws a hex grid + supports pan/zoom + selection.
#[derive(Debug, Clone)]
struct HexGridCanvas {
    grid: data::hex_grids::HexGrid,
    tiles: Vec<data::hex_grids::HexTile>,
}

#[derive(Debug, Clone, Copy)]
struct Viewport {
    /// Screen-space translation.
    pan: iced::Vector,
    /// Scale factor (1.0 means "base radius that fits the grid").
    zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        // `zoom=1.0` means: use the base radius computed to fit the grid into the canvas.
        Self {
            pan: iced::Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DragState {
    dragging: bool,
    last_cursor: Option<iced::Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedHex {
    x: i32,
    y: i32,
}

#[derive(Debug, Default)]
struct CanvasState {
    viewport: Viewport,
    drag: DragState,
    selected: Option<SelectedHex>,
}

impl HexGridCanvas {
    fn new(grid: data::hex_grids::HexGrid, tiles: Vec<data::hex_grids::HexTile>) -> Self {
        Self { grid, tiles }
    }

    fn get_tile(&self, x: i32, y: i32) -> Option<&data::hex_grids::HexTile> {
        self.tiles.iter().find(|t| t.coord.x == x && t.coord.y == y)
    }

    fn tile_style(tile: Option<&data::hex_grids::HexTile>) -> (iced::Color, char) {
        // Color + icon based on tile metadata (requested).
        // Priority order: Unit > Item > Level > tile_type > default present.
        if let Some(t) = tile {
            if t.unit_id.is_some() {
                return (iced::Color::from_rgb(0.85, 0.35, 0.35), 'U');
            }
            if t.item_id.is_some() {
                return (iced::Color::from_rgb(0.25, 0.65, 0.35), 'I');
            }
            if t.level_id.is_some() {
                return (iced::Color::from_rgb(0.55, 0.45, 0.85), 'L');
            }
            if let Some(tt) = t.tile_type.as_deref() {
                // Simple mapping for stringly types
                match tt {
                    "Unit" => return (iced::Color::from_rgb(0.85, 0.35, 0.35), 'U'),
                    "Item" => return (iced::Color::from_rgb(0.25, 0.65, 0.35), 'I'),
                    "Experience" => return (iced::Color::from_rgb(0.95, 0.75, 0.25), 'X'),
                    _ => {}
                }
            }
            return (iced::Color::from_rgb(0.28, 0.55, 0.85), '•');
        }

        // "Empty space" (no persisted hex_tiles row)
        (iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0), ' ')
    }

    /// Compute a base hex radius that fits the grid into the given bounds.
    /// The viewport zoom scales the final rendered radius.
    fn base_radius_to_fit(&self, bounds: iced::Size) -> f32 {
        compute_radius_to_fit(&self.grid, bounds)
    }

    fn world_to_screen(viewport: &Viewport, p: iced::Point) -> iced::Point {
        iced::Point::new(
            p.x * viewport.zoom + viewport.pan.x,
            p.y * viewport.zoom + viewport.pan.y,
        )
    }

    fn screen_to_world(viewport: &Viewport, p: iced::Point) -> iced::Point {
        iced::Point::new(
            (p.x - viewport.pan.x) / viewport.zoom,
            (p.y - viewport.pan.y) / viewport.zoom,
        )
    }

    /// Hit-test: find the nearest hex center (in *screen space*) within a radius threshold.
    ///
    /// We intentionally do hit-testing in screen space using the exact same geometry/origin math
    /// as `draw()`, so it stays correct under pan + zoom.
    fn hit_test_hex(
        &self,
        base_r: f32,
        viewport: &Viewport,
        bounds: iced::Rectangle,
        cursor_screen: iced::Point,
    ) -> Option<SelectedHex> {
        // Match `draw()` exactly:
        // - base radius that fits grid
        // - scaled by viewport zoom
        // - origin is centered using the scaled grid pixel size
        // - then pan is applied in screen space
        let r = (base_r * viewport.zoom).max(2.0);
        let w = (3.0_f32).sqrt() * r;
        let v_step = 1.5 * r;

        let grid_size = compute_grid_pixel_size(&self.grid, r);
        let origin_world = iced::Point::new(
            (bounds.width - grid_size.width) / 2.0,
            (bounds.height - grid_size.height) / 2.0,
        );
        let origin = iced::Point::new(
            origin_world.x + viewport.pan.x,
            origin_world.y + viewport.pan.y,
        );

        // Approximate the row from screen Y (good enough to narrow search).
        let approx_y_f = ((cursor_screen.y - origin.y - r) / v_step).round();
        let approx_y = approx_y_f.clamp(i32::MIN as f32, i32::MAX as f32) as i32;

        // Approximate the col from screen X. Since odd/even rows have different offsets,
        // we estimate using an even-row formula, then search neighbors.
        let approx_x_f = ((cursor_screen.x - origin.x - (w / 2.0)) / w).round();
        let approx_x = approx_x_f.clamp(i32::MIN as f32, i32::MAX as f32) as i32;

        let mut best: Option<(SelectedHex, f32)> = None;

        for dy in -2_i64..=2_i64 {
            for dx in -2_i64..=2_i64 {
                let y64 = (approx_y as i64) + dy;
                let x64 = (approx_x as i64) + dx;

                let y = y64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                let x = x64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

                if x < 0 || y < 0 || x >= self.grid.width || y >= self.grid.height {
                    continue;
                }

                let row_offset = if y % 2 == 0 { 0.0 } else { w / 2.0 };
                let cx = origin.x + (x as f32) * w + row_offset + (w / 2.0);
                let cy = origin.y + (y as f32) * v_step + r;

                let dx = cursor_screen.x - cx;
                let dy = cursor_screen.y - cy;
                let d2 = dx * dx + dy * dy;

                if best.map(|(_, b)| d2 < b).unwrap_or(true) {
                    best = Some((SelectedHex { x, y }, d2));
                }
            }
        }

        // Accept if within the circumscribed circle of the hex.
        let threshold = (r * 0.95) * (r * 0.95);
        match best {
            Some((h, d2)) if d2 <= threshold => Some(h),
            _ => None,
        }
    }
}

impl<MessageT> canvas::Program<MessageT> for HexGridCanvas {
    type State = CanvasState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<<iced::Renderer as canvas::Renderer>::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            frame.size(),
            iced::Color::from_rgb(0.98, 0.98, 0.985),
        );

        // Base radius that fits the grid; actual rendered radius is scaled by zoom.
        let base_r = self.base_radius_to_fit(bounds.size());
        let r = base_r * state.viewport.zoom;

        // Prevent degenerate drawing.
        let r = r.max(2.0);

        let w = (3.0_f32).sqrt() * r;
        let v_step = 1.5 * r;

        let grid_size = compute_grid_pixel_size(&self.grid, r);

        // World-space origin, then transformed by viewport (pan/zoom already applied via r and manual pan).
        // Since we scale r directly, we only need to apply pan in screen space.
        let origin_world = iced::Point::new(
            (bounds.width - grid_size.width) / 2.0,
            (bounds.height - grid_size.height) / 2.0,
        );

        // Apply pan as a translation in screen space.
        let origin = iced::Point::new(
            origin_world.x + state.viewport.pan.x,
            origin_world.y + state.viewport.pan.y,
        );

        // Colors
        let stroke_present = iced::Color::from_rgb(0.20, 0.20, 0.23);
        let stroke_empty = iced::Color::from_rgba(0.25, 0.25, 0.28, 0.40);
        let fill_empty = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0);

        // Draw grid
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let row_offset = if y % 2 == 0 { 0.0 } else { w / 2.0 };

                let cx = origin.x + (x as f32) * w + row_offset + (w / 2.0);
                let cy = origin.y + (y as f32) * v_step + r;

                let center = iced::Point::new(cx, cy);
                let hex = hex_path_pointy(center, r);

                let tile = self.get_tile(x, y);
                let (fill, icon) = Self::tile_style(tile);

                // Fill/stroke
                if tile.is_some() {
                    frame.fill(&hex, fill);
                    frame.stroke(
                        &hex,
                        canvas::Stroke::default()
                            .with_width((1.5 * state.viewport.zoom).max(1.0))
                            .with_color(stroke_present),
                    );
                } else {
                    frame.fill(&hex, fill_empty);
                    frame.stroke(
                        &hex,
                        canvas::Stroke::default()
                            .with_width((1.0 * state.viewport.zoom).max(0.75))
                            .with_color(stroke_empty),
                    );
                }

                // Selected highlight ring
                if state.selected == Some(SelectedHex { x, y }) {
                    frame.stroke(
                        &hex,
                        canvas::Stroke::default()
                            .with_width((3.0 * state.viewport.zoom).max(2.0))
                            .with_color(iced::Color::from_rgb(0.95, 0.75, 0.25)),
                    );
                }

                // Icon: draw a single character at the center for "data" tiles
                if tile.is_some() && icon != ' ' && state.viewport.zoom >= 0.65 {
                    let label = canvas::Text {
                        content: icon.to_string(),
                        position: center,
                        color: iced::Color::from_rgb(0.10, 0.10, 0.12),
                        size: iced::Pixels((18.0 * state.viewport.zoom).max(10.0)),
                        horizontal_alignment: iced::alignment::Horizontal::Center,
                        vertical_alignment: iced::alignment::Vertical::Center,
                        ..Default::default()
                    };
                    frame.fill_text(label);
                }
            }
        }

        // Popup: show selected tile data
        if let Some(sel) = state.selected {
            let tile = self.get_tile(sel.x, sel.y);

            // Popup content
            let mut lines = vec![format!("({},{})", sel.x, sel.y)];
            match tile {
                None => lines.push("Empty (no hex_tiles row)".to_string()),
                Some(t) => {
                    lines.push("Tile present".to_string());
                    lines.push(format!("unit_id: {:?}", t.unit_id));
                    lines.push(format!("item_id: {:?}", t.item_id));
                    lines.push(format!("level_id: {:?}", t.level_id));
                    lines.push(format!("type: {:?}", t.tile_type));
                }
            }

            // Popup position: top-left-ish inside canvas
            let popup_pos = iced::Point::new(16.0, 16.0);
            let padding = 10.0;
            let line_h = 18.0;
            let width = 260.0;
            let height = padding * 2.0 + (lines.len() as f32) * line_h;

            // Background box
            frame.fill_rectangle(
                popup_pos,
                iced::Size::new(width, height),
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.92),
            );
            let popup_border = canvas::Path::rectangle(popup_pos, iced::Size::new(width, height));
            frame.stroke(
                &popup_border,
                canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(iced::Color::from_rgba(0.1, 0.1, 0.12, 0.35)),
            );

            // Text lines
            for (i, line) in lines.iter().enumerate() {
                frame.fill_text(canvas::Text {
                    content: line.clone(),
                    position: iced::Point::new(
                        popup_pos.x + padding,
                        popup_pos.y + padding + (i as f32) * line_h,
                    ),
                    color: iced::Color::from_rgb(0.10, 0.10, 0.12),
                    size: iced::Pixels(15.0),
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<MessageT>) {
        use canvas::event::Status;

        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    if let Some(cursor_pos) = cursor.position_in(bounds) {
                        // Click-select
                        let base_r = self.base_radius_to_fit(bounds.size());
                        if let Some(hit) =
                            self.hit_test_hex(base_r, &state.viewport, bounds, cursor_pos)
                        {
                            state.selected = Some(hit);
                        } else {
                            state.selected = None;
                        }

                        // Start drag
                        state.drag.dragging = true;
                        state.drag.last_cursor = Some(cursor_pos);
                        return (Status::Captured, None);
                    }
                    (Status::Ignored, None)
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    state.drag.dragging = false;
                    state.drag.last_cursor = None;
                    (Status::Captured, None)
                }
                iced::mouse::Event::CursorMoved { .. } => {
                    if state.drag.dragging {
                        if let Some(cursor_pos) = cursor.position_in(bounds) {
                            if let Some(last) = state.drag.last_cursor {
                                let dx = cursor_pos.x - last.x;
                                let dy = cursor_pos.y - last.y;
                                state.viewport.pan.x += dx;
                                state.viewport.pan.y += dy;
                            }
                            state.drag.last_cursor = Some(cursor_pos);
                            return (Status::Captured, None);
                        }
                    }
                    (Status::Ignored, None)
                }
                iced::mouse::Event::WheelScrolled { delta } => {
                    // Zoom around cursor position (keeps the point under cursor stable)
                    if let Some(cursor_pos) = cursor.position_in(bounds) {
                        let old_zoom = state.viewport.zoom.max(0.05);
                        let scroll_y = match delta {
                            iced::mouse::ScrollDelta::Lines { y, .. } => y,
                            iced::mouse::ScrollDelta::Pixels { y, .. } => y / 60.0,
                        };

                        let zoom_factor = (1.0 + (scroll_y * 0.10)).clamp(0.80, 1.25);
                        let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 3.5);

                        // Adjust pan so that the world point under the cursor stays fixed.
                        let world_before = HexGridCanvas::screen_to_world(
                            &Viewport {
                                pan: state.viewport.pan,
                                zoom: old_zoom,
                            },
                            cursor_pos,
                        );
                        state.viewport.zoom = new_zoom;
                        let screen_after = HexGridCanvas::world_to_screen(
                            &Viewport {
                                pan: state.viewport.pan,
                                zoom: new_zoom,
                            },
                            world_before,
                        );

                        state.viewport.pan.x += cursor_pos.x - screen_after.x;
                        state.viewport.pan.y += cursor_pos.y - screen_after.y;

                        return (Status::Captured, None);
                    }
                    (Status::Ignored, None)
                }
                _ => (Status::Ignored, None),
            },
            _ => (Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        // Show grabbing cursor while dragging
        if state.drag.dragging {
            return iced::mouse::Interaction::Grabbing;
        }

        // Hint clickable tiles
        if cursor.position_in(bounds).is_some() {
            return iced::mouse::Interaction::Pointer;
        }

        iced::mouse::Interaction::default()
    }
}

/// Compute a reasonable hex radius that fits the grid into the available canvas size.
fn compute_radius_to_fit(grid: &data::hex_grids::HexGrid, bounds: iced::Size) -> f32 {
    // Avoid division by zero / invalid.
    let gw = grid.width.max(1) as f32;
    let gh = grid.height.max(1) as f32;

    // For pointy-top, odd-r:
    // total width ≈ w * cols + w/2 (because odd rows shift by w/2)
    // total height ≈ (gh - 1) * (3/2 r) + 2r
    //
    // where w = sqrt(3) r
    let sqrt3 = (3.0_f32).sqrt();

    let max_r_by_width = bounds.width / (sqrt3 * gw + (sqrt3 / 2.0)); // w*cols + w/2 => sqrt3*r*cols + sqrt3*r/2
    let max_r_by_height = bounds.height / (1.5 * (gh - 1.0) + 2.0);

    // Keep some padding.
    let r = max_r_by_width.min(max_r_by_height) * 0.95;

    // Clamp to something visible.
    r.max(6.0).min(48.0)
}

/// Compute the pixel size of the rendered grid for centering.
fn compute_grid_pixel_size(grid: &data::hex_grids::HexGrid, r: f32) -> iced::Size {
    let sqrt3 = (3.0_f32).sqrt();
    let w = sqrt3 * r;
    let cols = grid.width.max(1) as f32;
    let rows = grid.height.max(1) as f32;

    let width = w * cols + (w / 2.0); // extra half-hex for odd-row offset
    let height = (rows - 1.0) * (1.5 * r) + (2.0 * r);

    iced::Size::new(width, height)
}

/// Build a pointy-top hex path centered at `center` with radius `r`.
///
/// Corner angles for pointy-top:
/// - 0° at "top"
/// - then every 60° clockwise
fn hex_path_pointy(center: iced::Point, r: f32) -> canvas::Path {
    let mut builder = canvas::path::Builder::new();

    // Precompute the 6 corners.
    let mut corners = [iced::Point::ORIGIN; 6];
    for i in 0..6 {
        // Pointy top: start at -90° (top), then add 60°.
        let angle = (-90.0_f32 + 60.0 * (i as f32)).to_radians();
        corners[i] = iced::Point::new(center.x + r * angle.cos(), center.y + r * angle.sin());
    }

    builder.move_to(corners[0]);
    for i in 1..6 {
        builder.line_to(corners[i]);
    }
    builder.close();

    builder.build()
}

// Small helpers because stable Rust doesn't have these on f32 by default.
trait ToRadians {
    fn to_radians(self) -> f32;
}
impl ToRadians for f32 {
    fn to_radians(self) -> f32 {
        self * std::f32::consts::PI / 180.0
    }
}
