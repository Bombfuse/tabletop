use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Event, Geometry, Path, Program, Stroke};
use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};

use crate::gui::{HexGridRow, Message, ToolsGui};

/// Hex grid editor view with true pointy-top hexagon rendering.
///
/// Coordinate model:
/// - Uses (x,y) integer coordinates where x in [0..width) and y in [0..height).
/// - Rendered as pointy-top hexes using an "odd-r" offset layout (odd rows shifted right).
///
/// Persistence:
/// - Naming + saving are handled by Messages (`HexGridNameChanged`, `SaveHexGrid`).
/// - Listing/loading/deleting are handled by Messages (`RefreshHexGrids`, `LoadHexGridById`, `DeleteHexGridById`).
pub fn view(gui: &ToolsGui) -> Element<'_, Message> {
    let header = column![
        text("Hex Grid Editor").size(22),
        text("Pointy-top hexes. Left-click drag paints. Right-click deletes. Middle-drag pans.")
            .size(14),
    ]
    .spacing(6);

    let controls = row![
        column![
            text("Name").size(14),
            text_input("My Hex Grid", &gui.hex_grid_name)
                .on_input(Message::HexGridNameChanged)
                .padding(8),
        ]
        .spacing(6)
        .width(Length::Fixed(220.0)),
        column![
            text("Width").size(14),
            text_input("9", &gui.hex_grid_width)
                .on_input(Message::HexGridWidthChanged)
                .padding(8),
        ]
        .spacing(6)
        .width(Length::Fixed(140.0)),
        column![
            text("Height").size(14),
            text_input("9", &gui.hex_grid_height)
                .on_input(Message::HexGridHeightChanged)
                .padding(8),
        ]
        .spacing(6)
        .width(Length::Fixed(140.0)),
        button("Apply resize").on_press(Message::HexGridApplyResize),
        button("Save grid").on_press(Message::SaveHexGrid),
        button("Create new grid").on_press(Message::CreateNewHexGrid),
        button("Refresh list").on_press(Message::RefreshHexGrids),
        iced::widget::Space::with_width(Length::Fill),
    ]
    .spacing(12)
    .align_items(Alignment::End);

    // Main editing area:
    // - Left: the canvas (should keep as much space as possible)
    // - Right: the list sidebar (scrollable) + tile presence inspector
    let grid = hex_grid_canvas(gui);
    let editor = selected_tile_editor(gui);
    let list = existing_hex_grids_list(gui);

    let right_sidebar = column![editor, horizontal_rule(1), list,]
        .spacing(12)
        .width(Length::Fixed(380.0))
        .height(Length::Fill);

    column![
        header,
        horizontal_rule(1),
        controls,
        row![grid, right_sidebar].spacing(16).height(Length::Fill),
    ]
    .spacing(12)
    .height(Length::Fill)
    .into()
}

fn parse_dim(s: &str) -> Option<i32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.parse::<i32>().ok().filter(|v| *v > 0)
}

fn existing_hex_grids_list(gui: &ToolsGui) -> Element<'_, Message> {
    let mut col = column![
        text("Saved Hex Grids").size(18),
        text("Load a grid to edit it in the canvas.").size(13),
    ]
    .spacing(8);

    if gui.hex_grids.is_empty() {
        col = col.push(text("No saved hex grids yet.").size(13));
        return container(col).padding(12).into();
    }

    for HexGridRow {
        id,
        name,
        width,
        height,
    } in gui.hex_grids.iter().cloned()
    {
        let is_active = gui.hex_grid_id == Some(id);

        let row_ui = row![
            text(format!("#{id}")).size(13),
            text(format!("{name} ({width}x{height})"))
                .size(13)
                .width(Length::Fill),
            if is_active {
                button("Editing").padding(6)
            } else {
                button("Load")
                    .padding(6)
                    .on_press(Message::LoadHexGridById(id))
            },
            button("Delete")
                .padding(6)
                .on_press(Message::DeleteHexGridById(id)),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        col = col.push(row_ui);
    }

    // Scrollable list that doesn't take space from the canvas (lives in the fixed-width sidebar).
    scrollable(container(col).padding(12))
        .height(Length::Fill)
        .into()
}

fn hex_grid_canvas(gui: &ToolsGui) -> Element<'_, Message> {
    let Some(w) = parse_dim(&gui.hex_grid_width) else {
        return container(text("Enter a valid width (> 0)."))
            .padding(12)
            .into();
    };
    let Some(h) = parse_dim(&gui.hex_grid_height) else {
        return container(text("Enter a valid height (> 0)."))
            .padding(12)
            .into();
    };

    // The canvas is the pan/scroll surface:
    // - Mouse wheel scrolls vertically.
    // - Trackpad horizontal scroll pans horizontally.
    // - Middle-click drag pans in both axes.
    //
    // Keep the canvas viewport relative to the page by using Fill for width/height.
    // The canvas content is larger than the viewport; the Program translates it.
    let radius = 16.0_f32;
    let padding = 14.0_f32;

    let program = HexGridProgram::new(
        w,
        h,
        radius,
        padding,
        gui.hex_grid_selected_x,
        gui.hex_grid_selected_y,
        &gui.hex_grid_tiles_present,
    );

    container(
        Canvas::new(program)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(6)
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .into()
}

fn selected_tile_editor(gui: &ToolsGui) -> Element<'_, Message> {
    let title = text("Tile").size(18);

    let Some(x) = gui.hex_grid_selected_x else {
        return container(
            column![
                title,
                horizontal_rule(1),
                text("Click a hex tile to see if it is present.").size(14),
            ]
            .spacing(10),
        )
        .padding(12)
        .width(Length::Fill)
        .into();
    };
    let Some(y) = gui.hex_grid_selected_y else {
        return container(
            column![
                title,
                horizontal_rule(1),
                text("Click a hex tile to see if it is present.").size(14),
            ]
            .spacing(10),
        )
        .padding(12)
        .width(Length::Fill)
        .into();
    };

    let present = gui.hex_grid_tiles_present.contains(&(x, y));

    let body = column![
        text(format!("Selected: ({x},{y})")).size(14),
        text(format!("Present: {}", if present { "yes" } else { "no" })).size(14),
        row![
            button("Paint tile").on_press(Message::HexGridTileClicked(x, y)),
            button("Delete tile").on_press(Message::HexGridTileClear(x, y)),
        ]
        .spacing(12),
    ]
    .spacing(10);

    container(column![title, horizontal_rule(1), body].spacing(10))
        .padding(12)
        .width(Length::Fill)
        .into()
}

#[derive(Debug, Default)]
struct ScrollState {
    scroll_x: f32,
    scroll_y: f32,

    // Drag-to-pan state (middle mouse)
    dragging: bool,
    last_cursor: Option<Point>,

    // Paint state (left mouse)
    painting: bool,
    last_painted: Option<(i32, i32)>,
}

struct HexGridProgram {
    w: i32,
    h: i32,
    radius: f32,
    padding: f32,

    selected_x: Option<i32>,
    selected_y: Option<i32>,

    /// Present tiles (x,y)
    present: std::collections::BTreeSet<(i32, i32)>,

    cache: Cache,
}

impl HexGridProgram {
    fn new(
        w: i32,
        h: i32,
        radius: f32,
        padding: f32,
        selected_x: Option<i32>,
        selected_y: Option<i32>,
        tiles_present: &std::collections::BTreeSet<(i32, i32)>,
    ) -> Self {
        let present = tiles_present.iter().copied().collect();
        Self {
            w,
            h,
            radius,
            padding,
            selected_x,
            selected_y,
            present,
            cache: Cache::new(),
        }
    }

    fn hex_w(&self) -> f32 {
        (3.0_f32).sqrt() * self.radius
    }

    fn hex_h(&self) -> f32 {
        2.0 * self.radius
    }

    fn step_x(&self) -> f32 {
        self.hex_w()
    }

    fn step_y(&self) -> f32 {
        1.5 * self.radius
    }

    /// Pointy-top odd-r layout: odd rows shift right by half a column.
    fn center_for(&self, x: i32, y: i32) -> Point {
        let row_shift = if (y & 1) == 1 {
            self.step_x() / 2.0
        } else {
            0.0
        };

        let cx = self.padding + row_shift + (x as f32) * self.step_x() + self.hex_w() / 2.0;
        let cy = self.padding + (y as f32) * self.step_y() + self.hex_h() / 2.0;

        Point::new(cx, cy)
    }

    fn hex_points(&self, center: Point) -> [Point; 6] {
        // Pointy-top orientation:
        // angle 0 at -90 degrees, then every 60 degrees.
        let mut pts = [Point::ORIGIN; 6];
        for i in 0..6 {
            let angle = (-90.0_f32 + i as f32 * 60.0).to_radians();
            pts[i] = Point::new(
                center.x + self.radius * angle.cos(),
                center.y + self.radius * angle.sin(),
            );
        }
        pts
    }

    fn path_for(&self, x: i32, y: i32) -> Path {
        let c = self.center_for(x, y);
        let pts = self.hex_points(c);

        Path::new(|b| {
            b.move_to(pts[0]);
            for p in &pts[1..] {
                b.line_to(*p);
            }
            b.close();
        })
    }

    fn point_in_poly(p: Point, poly: &[Point; 6]) -> bool {
        // Ray casting algorithm.
        let mut inside = false;
        let mut j = 5usize;
        for i in 0..6usize {
            let pi = poly[i];
            let pj = poly[j];

            let intersects = ((pi.y > p.y) != (pj.y > p.y))
                && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y + 0.000001) + pi.x);
            if intersects {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    fn hit_test(&self, cursor: Point) -> Option<(i32, i32)> {
        // Fast approximate: guess row by y, then guess col by x with row shift, then
        // check a small neighborhood with exact point-in-hex.
        let y_guess =
            ((cursor.y - self.padding - self.hex_h() / 2.0) / self.step_y()).round() as i32;

        let mut best: Option<(i32, i32)> = None;

        for y in (y_guess - 2)..=(y_guess + 2) {
            if y < 0 || y >= self.h {
                continue;
            }
            let row_shift = if (y & 1) == 1 {
                self.step_x() / 2.0
            } else {
                0.0
            };
            let x_guess = ((cursor.x - self.padding - row_shift - self.hex_w() / 2.0)
                / self.step_x())
            .round() as i32;

            for x in (x_guess - 2)..=(x_guess + 2) {
                if x < 0 || x >= self.w {
                    continue;
                }
                let c = self.center_for(x, y);
                let pts = self.hex_points(c);
                if Self::point_in_poly(cursor, &pts) {
                    best = Some((x, y));
                    break;
                }
            }

            if best.is_some() {
                break;
            }
        }

        best
    }

    fn content_size(&self) -> (f32, f32) {
        let w = self.hex_w();
        let h = self.padding * 2.0 + (self.h as f32 - 1.0) * self.step_y() + self.hex_h();
        // include odd-row extra half-shift by adding step_x/2 to the total width
        let w_total =
            self.padding * 2.0 + (self.w as f32 - 1.0) * self.step_x() + w + (self.step_x() / 2.0);
        (w_total, h)
    }
}

impl Program<Message> for HexGridProgram {
    type State = ScrollState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let scroll_x = state.scroll_x;
        let scroll_y = state.scroll_y;

        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            frame.fill_rectangle(
                Point::ORIGIN,
                frame.size(),
                Color::from_rgba(0.08, 0.09, 0.11, 1.0),
            );

            // Translate drawing by pan offsets
            frame.translate(Vector::new(-scroll_x, -scroll_y));

            let stroke_normal = Stroke {
                width: 1.25,
                style: canvas::Style::Solid(Color::from_rgba(0.55, 0.58, 0.64, 1.0)),
                ..Stroke::default()
            };

            let stroke_selected = Stroke {
                width: 2.25,
                style: canvas::Style::Solid(Color::from_rgba(0.95, 0.78, 0.20, 1.0)),
                ..Stroke::default()
            };

            let fill_present = Color::from_rgba(0.20, 0.45, 0.85, 0.35);
            let fill_selected = Color::from_rgba(0.95, 0.78, 0.20, 0.25);

            for y in 0..self.h {
                for x in 0..self.w {
                    let path = self.path_for(x, y);
                    let is_selected = self.selected_x == Some(x) && self.selected_y == Some(y);
                    let is_present = self.present.contains(&(x, y));

                    if is_present {
                        frame.fill(&path, fill_present);
                    }

                    if is_selected {
                        frame.fill(&path, fill_selected);
                        frame.stroke(&path, stroke_selected.clone());
                    } else {
                        frame.stroke(&path, stroke_normal.clone());
                    }
                }
            }
        });

        vec![geom]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let (content_w, content_h) = self.content_size();
        let max_scroll_x = (content_w - bounds.width).max(0.0);
        let max_scroll_y = (content_h - bounds.height).max(0.0);

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let (dx_lines, dy_lines) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (x, y),
                    mouse::ScrollDelta::Pixels { x, y } => (x / 24.0, y / 24.0),
                };

                // Default: wheel scrolls vertically. If the user scrolls horizontally (trackpad)
                // or provides x delta, respect it. Additionally, Shift+wheel pans horizontally.
                let pan_x = -dx_lines * 32.0;
                let pan_y = -dy_lines * 32.0;

                if cursor.is_over(bounds) {
                    // If shift is held, treat vertical wheel as horizontal pan.
                    // NOTE: Iced does not expose modifiers on wheel events directly in all backends;
                    // this still supports true horizontal deltas from trackpads.
                }

                state.scroll_x = (state.scroll_x + pan_x).clamp(0.0, max_scroll_x);
                state.scroll_y = (state.scroll_y + pan_y).clamp(0.0, max_scroll_y);

                self.cache.clear();
                return (canvas::event::Status::Captured, None);
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.dragging = true;
                state.last_cursor = cursor.position_in(bounds);
                return (canvas::event::Status::Captured, None);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                state.dragging = false;
                state.last_cursor = None;
                return (canvas::event::Status::Captured, None);
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.painting = false;
                state.last_painted = None;
                return (canvas::event::Status::Captured, None);
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    let Some(cur) = cursor.position_in(bounds) else {
                        return (canvas::event::Status::Captured, None);
                    };
                    let Some(prev) = state.last_cursor else {
                        state.last_cursor = Some(cur);
                        return (canvas::event::Status::Captured, None);
                    };

                    let delta = cur - prev;
                    state.last_cursor = Some(cur);

                    // Dragging moves the camera opposite direction of cursor movement.
                    state.scroll_x = (state.scroll_x - delta.x).clamp(0.0, max_scroll_x);
                    state.scroll_y = (state.scroll_y - delta.y).clamp(0.0, max_scroll_y);

                    self.cache.clear();
                    return (canvas::event::Status::Captured, None);
                }

                // Paint while left button is held: emit a "tile clicked" message as the cursor
                // moves across new hexes.
                if state.painting {
                    let cursor_pos = match cursor.position_in(bounds) {
                        Some(p) => Point::new(p.x + state.scroll_x, p.y + state.scroll_y),
                        None => return (canvas::event::Status::Captured, None),
                    };

                    if let Some((x, y)) = self.hit_test(cursor_pos) {
                        if state.last_painted != Some((x, y)) {
                            state.last_painted = Some((x, y));
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::HexGridTileClicked(x, y)),
                            );
                        }
                    }

                    return (canvas::event::Status::Captured, None);
                }
            }
            _ => {}
        }

        let cursor_pos = match cursor.position_in(bounds) {
            Some(p) => Point::new(p.x + state.scroll_x, p.y + state.scroll_y),
            None => return (canvas::event::Status::Ignored, None),
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some((x, y)) = self.hit_test(cursor_pos) {
                    // Begin painting on press, and paint the pressed tile immediately.
                    state.painting = true;
                    state.last_painted = Some((x, y));
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::HexGridTileClicked(x, y)),
                    );
                }
                (canvas::event::Status::Ignored, None)
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some((x, y)) = self.hit_test(cursor_pos) {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::HexGridTileClear(x, y)),
                    );
                }
                (canvas::event::Status::Ignored, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let Some(p) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let p = Point::new(p.x + state.scroll_x, p.y + state.scroll_y);

        if state.dragging {
            mouse::Interaction::Grabbing
        } else if self.hit_test(p).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
