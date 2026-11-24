//! The window: a header with the active layer and status, a row of layer
//! chips, and the keyboard drawn from the firmware's physical layout.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2,
};
use eframe::emath::Rot2;
use rynk::rmk_types::action::KeyAction;
use tokio::sync::mpsc::UnboundedSender;

use crate::device::{Command, KeyPlacement, KeyboardState, LinkStatus, Shared, battery_text};
use crate::labels::{Kind, KeyLabel, LayerNames, key_label};

pub struct ViewerApp {
    state: Shared,
    commands: UnboundedSender<Command>,
    layer_names: Vec<String>,
    /// `Some(layer)` while the user pins a layer to look at; `None` = follow
    /// the keyboard's active layer.
    pinned: Option<u8>,
}

impl ViewerApp {
    pub fn new(state: Shared, commands: UnboundedSender<Command>, layer_names: Vec<String>) -> Self {
        Self {
            state,
            commands,
            layer_names,
            pinned: None,
        }
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Copy the state out so the connection thread is never blocked on us.
        let state = self.state.lock().unwrap().clone();
        let shown_layer = self.pinned.unwrap_or(state.current_layer);

        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(4.0);
            self.header(ui, &state, shown_layer);
            if state.loaded {
                ui.add_space(2.0);
                self.layer_chips(ui, &state, shown_layer);
            }
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if state.loaded {
                draw_keyboard(ui, &state, shown_layer, &self.layer_names);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(waiting_text(&state.link)).weak());
                });
            }
        });
    }
}

impl ViewerApp {
    fn header(&mut self, ui: &mut egui::Ui, state: &KeyboardState, shown_layer: u8) {
        ui.horizontal(|ui| {
            let (dot, tip) = match &state.link {
                LinkStatus::Connected(name) => (Color32::from_rgb(80, 200, 120), format!("Connected to {name}")),
                LinkStatus::Connecting(name) => (Color32::from_rgb(230, 190, 60), format!("Connecting to {name}…")),
                LinkStatus::Searching => (Color32::GRAY, "Looking for a Rynk keyboard…".into()),
                LinkStatus::Lost(e) => (Color32::from_rgb(220, 80, 80), format!("Disconnected: {e}")),
            };
            let (r, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 5.0, dot);
            ui.interact(r, ui.scope_id().with("link"), Sense::hover()).on_hover_text(tip);

            let title = if state.loaded {
                self.layer_names.layer_name(shown_layer)
            } else {
                "TOTEM".to_string()
            };
            ui.label(egui::RichText::new(title).heading().strong());
            if state.loaded {
                ui.label(egui::RichText::new(format!("layer {shown_layer}")).weak());
                if self.pinned.is_some() {
                    ui.label(egui::RichText::new("(pinned)").weak().italics());
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.loaded
                    && ui
                        .small_button("⟳")
                        .on_hover_text("Re-read keymap from the keyboard")
                        .clicked()
                {
                    let _ = self.commands.send(Command::Reload);
                }
                for (i, p) in state.peripherals.iter().enumerate().rev() {
                    let side = match i {
                        0 => "L",
                        1 => "R",
                        _ => "P",
                    };
                    let text = if !p.connected {
                        format!("{side} ✗")
                    } else {
                        battery_text(&p.battery).map_or(side.to_string(), |b| format!("{side} {b}"))
                    };
                    let tip = if p.connected { "half connected" } else { "half not connected" };
                    ui.label(egui::RichText::new(text).small()).on_hover_text(tip);
                }
                if state.leds.is_some_and(|l| l.caps_lock()) {
                    ui.label(
                        egui::RichText::new("CAPS")
                            .small()
                            .strong()
                            .color(Color32::from_rgb(230, 190, 60)),
                    );
                }
            });
        });
    }

    fn layer_chips(&mut self, ui: &mut egui::Ui, state: &KeyboardState, shown_layer: u8) {
        let Some(caps) = &state.caps else { return };
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for layer in 0..caps.num_layers {
                let mut text = egui::RichText::new(self.layer_names.layer_name(layer)).small();
                if layer == state.current_layer {
                    text = text.strong();
                }
                let resp = ui
                    .selectable_label(layer == shown_layer, text)
                    .on_hover_text(if layer == state.current_layer {
                        "Active on the keyboard"
                    } else {
                        "Click to preview; click again to follow the keyboard"
                    });
                if resp.clicked() {
                    self.pinned = if self.pinned == Some(layer) || layer == state.current_layer && self.pinned.is_none() {
                        None
                    } else {
                        Some(layer)
                    };
                }
            }
        });
    }
}

fn waiting_text(link: &LinkStatus) -> String {
    match link {
        LinkStatus::Searching => "Waiting for the keyboard…\nPlug in the dongle via USB.".into(),
        LinkStatus::Connecting(n) => format!("Connecting to {n}…"),
        LinkStatus::Connected(_) => "Reading keymap…".into(),
        LinkStatus::Lost(e) => format!("Not connected: {e}\nRetrying…"),
    }
}

struct Palette {
    fill: Color32,
    text: Color32,
    hold: Color32,
}

fn palette(kind: Kind, dark: bool) -> Palette {
    let (fill, hold) = match (kind, dark) {
        (Kind::Normal, true) => (Color32::from_rgb(52, 56, 64), Color32::from_rgb(120, 180, 255)),
        (Kind::Modifier, true) => (Color32::from_rgb(40, 62, 92), Color32::from_rgb(120, 180, 255)),
        (Kind::Layer, true) => (Color32::from_rgb(78, 58, 34), Color32::from_rgb(255, 170, 80)),
        (Kind::Control, true) => (Color32::from_rgb(86, 44, 48), Color32::from_rgb(255, 140, 140)),
        (Kind::Transparent | Kind::Empty, true) => (Color32::from_rgb(34, 36, 40), Color32::GRAY),
        (Kind::Normal, false) => (Color32::from_rgb(236, 238, 242), Color32::from_rgb(30, 100, 200)),
        (Kind::Modifier, false) => (Color32::from_rgb(214, 228, 248), Color32::from_rgb(30, 100, 200)),
        (Kind::Layer, false) => (Color32::from_rgb(250, 228, 200), Color32::from_rgb(190, 100, 10)),
        (Kind::Control, false) => (Color32::from_rgb(248, 214, 214), Color32::from_rgb(180, 40, 40)),
        (Kind::Transparent | Kind::Empty, false) => (Color32::from_rgb(246, 246, 248), Color32::GRAY),
    };
    let text = if dark { Color32::from_gray(235) } else { Color32::from_gray(25) };
    Palette { fill, text, hold }
}

fn draw_keyboard(ui: &mut egui::Ui, state: &KeyboardState, layer: u8, names: &[String]) {
    let keys = &state.layout;
    if keys.is_empty() {
        return;
    }

    // Bounding box of all keys in key units (rotation ignored for bounds).
    let (mut min, mut max) = (Pos2::new(f32::MAX, f32::MAX), Pos2::new(f32::MIN, f32::MIN));
    for k in keys {
        min.x = min.x.min(k.x - k.w / 2.0);
        min.y = min.y.min(k.y - k.h / 2.0);
        max.x = max.x.max(k.x + k.w / 2.0);
        max.y = max.y.max(k.y + k.h / 2.0);
    }
    let size_units = max - min;
    let avail = ui.available_rect_before_wrap();
    let unit = (avail.width() / size_units.x).min(avail.height() / size_units.y);
    let drawn = size_units * unit;
    let origin = avail.center() - drawn / 2.0;
    let to_screen = |x: f32, y: f32| origin + Vec2::new(x - min.x, y - min.y) * unit;
    ui.allocate_rect(avail, Sense::hover());

    let dark = ui.visuals().dark_mode;
    let painter = ui.painter_at(avail);
    let gap = (unit * 0.06).max(2.0);

    for k in keys {
        let Some(action) = state.action(layer, k.row, k.col) else { continue };
        let center = to_screen(k.x, k.y);
        let half = Vec2::new(k.w * unit - gap, k.h * unit - gap) / 2.0;
        let rect = Rect::from_center_size(center, half * 2.0);

        let mut label = key_label(action, names);
        let mut fallthrough = false;
        if *action == KeyAction::Transparent {
            // Show what the key does on the default layer underneath.
            if let Some(below) = state.action(state.default_layer, k.row, k.col)
                && *below != KeyAction::Transparent
            {
                label = key_label(below, names);
                fallthrough = true;
            }
        }

        let colors = palette(if fallthrough { Kind::Transparent } else { label.kind }, dark);
        let outline = Stroke::new(1.0, colors.fill.gamma_multiply(1.6));

        if k.rotation_deg.abs() < 0.01 {
            painter.rect(rect, CornerRadius::same((unit * 0.12) as u8), colors.fill, outline, StrokeKind::Inside);
        } else {
            let rot = Rot2::from_angle(k.rotation_deg.to_radians());
            let corners = [
                Vec2::new(-half.x, -half.y),
                Vec2::new(half.x, -half.y),
                Vec2::new(half.x, half.y),
                Vec2::new(-half.x, half.y),
            ]
            .map(|v| center + rot * v)
            .to_vec();
            painter.add(Shape::convex_polygon(corners, colors.fill, outline));
        }

        let text_color = if fallthrough { colors.text.gamma_multiply(0.45) } else { colors.text };
        draw_key_text(&painter, rect, unit, &label, text_color, colors.hold, fallthrough);

        let resp = ui.interact(rect, ui.scope_id().with(("key", k.row, k.col)), Sense::hover());
        resp.on_hover_ui(|ui| key_tooltip(ui, k, action, &label, fallthrough));
    }
}

fn draw_key_text(
    painter: &egui::Painter,
    rect: Rect,
    unit: f32,
    label: &KeyLabel,
    text_color: Color32,
    hold_color: Color32,
    fallthrough: bool,
) {
    let max_w = rect.width() * 0.88;
    let has_hold = label.hold.as_deref().is_some_and(|h| !h.is_empty());

    if !label.tap.is_empty() {
        let pos = if has_hold {
            rect.center() - Vec2::new(0.0, rect.height() * 0.12)
        } else {
            rect.center()
        };
        let size = fit_font(painter, &label.tap, unit * 0.30, max_w);
        painter.text(pos, Align2::CENTER_CENTER, &label.tap, FontId::proportional(size), text_color);
    }
    if let Some(hold) = label.hold.as_deref().filter(|h| !h.is_empty()) {
        let size = fit_font(painter, hold, unit * 0.19, max_w);
        let pos = rect.center_bottom() - Vec2::new(0.0, rect.height() * 0.2);
        let color = if fallthrough { hold_color.gamma_multiply(0.45) } else { hold_color };
        painter.text(pos, Align2::CENTER_CENTER, hold, FontId::proportional(size), color);
    }
    if fallthrough {
        painter.text(
            rect.left_top() + Vec2::new(rect.width() * 0.1, rect.height() * 0.08),
            Align2::LEFT_TOP,
            "▽",
            FontId::proportional(unit * 0.14),
            text_color,
        );
    }
}

/// Largest font size ≤ `start` whose rendering of `text` fits in `max_w`.
fn fit_font(painter: &egui::Painter, text: &str, start: f32, max_w: f32) -> f32 {
    let width = painter
        .layout_no_wrap(text.to_owned(), FontId::proportional(start), Color32::WHITE)
        .size()
        .x;
    if width <= max_w || width <= 0.0 {
        start
    } else {
        (start * max_w / width).max(6.0)
    }
}

fn key_tooltip(ui: &mut egui::Ui, k: &KeyPlacement, action: &KeyAction, label: &KeyLabel, fallthrough: bool) {
    ui.label(egui::RichText::new(format!("matrix ({}, {})", k.row, k.col)).weak());
    if fallthrough {
        ui.label("Transparent – falls through to:");
    }
    match &label.hold {
        Some(h) => {
            ui.label(format!("tap: {}", if label.tap.is_empty() { "—" } else { &label.tap }));
            ui.label(format!("hold: {h}"));
        }
        None if !label.tap.is_empty() => {
            ui.label(&label.tap);
        }
        None => {
            ui.label("no action");
        }
    }
    ui.separator();
    ui.label(egui::RichText::new(format!("{action:?}")).monospace().small());
}
