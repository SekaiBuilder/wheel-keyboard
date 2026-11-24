//! Wheel Keyboard for the desktop: type with two thumbs on a touchscreen, or
//! with a mouse or trackpad (left half of the window is the left wheel).
//!
//! What it learns is kept in a plain text file in the user's data folder and
//! never sent anywhere.

use std::path::PathBuf;
use std::time::Instant;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use wheel_keyboard::geometry::{DEAD_ZONE, INNER_EDGE};
use wheel_keyboard::{
    Action, Button, Config, Context, Key, Keyboard, Layer, Layout, ModState, Model, Modifier, Side,
};

const TOP_BAR: f32 = 48.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 560.0])
            .with_title("Wheel Keyboard"),
        ..Default::default()
    };
    eframe::run_native(
        "Wheel Keyboard",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    kb: Keyboard,
    text: String,
    undo: Vec<String>,
    status: String,
    start: Instant,
    /// Touch id → wheel, for real multi-touch screens.
    touches: Vec<(u64, Side)>,
    touch_seen: bool,
    mouse_side: Option<Side>,
    wheel_centres: [Pos2; 2],
    radius: f32,
    zone: Rect,
    model_dirty: bool,
}

impl App {
    fn new() -> Self {
        let layout = data_dir()
            .and_then(|d| std::fs::read_to_string(d.join("layout.txt")).ok())
            .and_then(|c| Layout::by_code(c.trim()))
            .unwrap_or_default();
        let mut kb = Keyboard::new(Config::default(), layout);
        load_model(&mut kb);
        App {
            kb,
            text: String::new(),
            undo: vec![],
            status: String::new(),
            start: Instant::now(),
            touches: vec![],
            touch_seen: false,
            mouse_side: None,
            wheel_centres: [Pos2::ZERO; 2],
            radius: 100.0,
            zone: Rect::NOTHING,
            model_dirty: false,
        }
    }

    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn save_model(&mut self) {
        if !self.model_dirty {
            return;
        }
        if let Some(dir) = data_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(
                model_file(&dir, self.kb.layout().code),
                self.kb.model().to_text(),
            );
        }
        self.model_dirty = false;
    }

    fn apply(&mut self, ctx: &egui::Context, actions: Vec<Action>) {
        if actions.is_empty() {
            return;
        }
        self.model_dirty = true;
        for a in actions {
            match a {
                Action::Text(s) => {
                    self.push_undo();
                    self.text.push_str(&s);
                    self.status = format!("typed {s:?}");
                }
                Action::Backspace(n) => {
                    self.push_undo();
                    for _ in 0..n {
                        self.text.pop();
                    }
                    self.status = "delete".into();
                }
                Action::Key { key, mods } => {
                    let cmd = mods.command_part();
                    let shortcut = cmd.contains(Modifier::Cmd) || cmd.contains(Modifier::Ctrl);
                    match (cmd.is_empty(), key) {
                        (true, Key::Enter) => {
                            self.push_undo();
                            self.text.push('\n');
                        }
                        (true, Key::Tab) => {
                            self.push_undo();
                            self.text.push('\t');
                        }
                        (false, Key::Char('C')) if shortcut => ctx.copy_text(self.text.clone()),
                        (false, Key::Char('Z')) if shortcut => {
                            if let Some(t) = self.undo.pop() {
                                self.text = t;
                                self.kb.set_text_before_cursor(&self.text);
                            }
                        }
                        _ => {}
                    }
                    self.status = format!("→ {}{}", mod_names(mods), key_name(key));
                }
                Action::Kana(on) => {
                    self.status = if on {
                        "kana input".into()
                    } else {
                        "alphanumeric input".into()
                    }
                }
            }
        }
    }

    fn push_undo(&mut self) {
        self.undo.push(self.text.clone());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
    }

    fn side_at(&self, p: Pos2) -> Side {
        if p.x < self.zone.center().x {
            Side::Left
        } else {
            Side::Right
        }
    }

    fn down(&mut self, side: Side, p: Pos2) {
        let now = self.now();
        let r = self.radius;
        let c = Pos2::new(
            p.x.clamp(self.zone.left() + r + 2.0, self.zone.right() - r - 2.0),
            p.y.clamp(self.zone.top() + r + 2.0, self.zone.bottom() - r - 2.0),
        );
        self.wheel_centres[idx(side)] = c;
        self.kb.touch_down(side, p.x, p.y, r, now);
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let events = ctx.input(|i| i.events.clone());
        for e in &events {
            if let egui::Event::Touch { id, phase, pos, .. } = e {
                self.touch_seen = true;
                let now = self.now();
                match phase {
                    egui::TouchPhase::Start if self.zone.contains(*pos) => {
                        let side = self.side_at(*pos);
                        if !self.kb.is_down(side) {
                            self.touches.push((id.0, side));
                            self.down(side, *pos);
                        }
                    }
                    egui::TouchPhase::Move => {
                        if let Some(&(_, side)) = self.touches.iter().find(|t| t.0 == id.0) {
                            self.kb.touch_move(side, pos.x, pos.y, now);
                        }
                    }
                    egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                        if let Some(i) = self.touches.iter().position(|t| t.0 == id.0) {
                            let (_, side) = self.touches.remove(i);
                            if *phase == egui::TouchPhase::End {
                                let a = self.kb.touch_up(side, now);
                                self.apply(ctx, a);
                            } else {
                                self.kb.touch_cancel(side);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if self.touch_seen {
            return; // egui also turns the first touch into mouse events; ignore those.
        }
        let (pressed, released, pos) = ctx.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_released(),
                i.pointer.latest_pos(),
            )
        });
        let now = self.now();
        if let Some(p) = pos {
            if pressed && self.zone.contains(p) && !ctx.wants_pointer_input() {
                let side = self.side_at(p);
                self.mouse_side = Some(side);
                self.down(side, p);
            } else if let Some(side) = self.mouse_side {
                self.kb.touch_move(side, p.x, p.y, now);
            }
        }
        if released {
            if let Some(side) = self.mouse_side.take() {
                let a = self.kb.touch_up(side, now);
                self.apply(ctx, a);
            }
        }
    }

    fn draw_wheel(&self, painter: &egui::Painter, side: Side) {
        let kb = &self.kb;
        let down = kb.is_down(side);
        let c = self.wheel_centres[idx(side)];
        let r = self.radius;
        let rings = kb.rings(side);
        let sel = kb.selection(side);
        let hl = kb.highlights();
        let vowels = side == kb.config.vowel_hand;
        let tint = if vowels {
            Color32::from_rgb(232, 129, 74)
        } else {
            Color32::from_rgb(91, 141, 239)
        };
        let alpha = if down { 255 } else { 170 };
        painter.circle_filled(c, r, Color32::from_rgba_unmultiplied(22, 26, 36, alpha));
        for (ring_i, keys) in [rings.inner, rings.outer].iter().enumerate() {
            let (r1, r2) = if ring_i == 0 {
                (DEAD_ZONE * r, INNER_EDGE * r)
            } else {
                (INNER_EDGE * r, r)
            };
            for (i, key) in keys.iter().enumerate() {
                let am = (-90.0 + i as f32 * 45.0).to_radians();
                let on = sel.is_some_and(|s| {
                    (s.ring == wheel_keyboard::RingId::Outer) == (ring_i == 1)
                        && s.dir as usize == i
                });
                let p = hl.iter().find(|(k, _)| k == key).map(|h| h.1 as f32);
                let fill = if on {
                    tint
                } else if let Some(p) = p {
                    tint.gamma_multiply(0.15 + p * 0.5)
                } else if ring_i == 0 {
                    Color32::from_rgba_unmultiplied(40, 46, 60, alpha)
                } else {
                    Color32::TRANSPARENT
                };
                sector(painter, c, r1, r2, am, fill);
                let a1 = am - 22.5f32.to_radians();
                painter.line_segment(
                    [c + Vec2::angled(a1) * r1, c + Vec2::angled(a1) * r2],
                    Stroke::new(1.0_f32, Color32::from_gray(60)),
                );
                let rm = (r1 + r2) / 2.0;
                let size = (r * 0.15).max(14.0)
                    * if ring_i == 1 { 0.82 } else { 1.0 }
                    * if p.is_some() { 1.15 } else { 1.0 };
                let color = if on {
                    Color32::BLACK
                } else if p.is_some() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(200)
                };
                painter.text(
                    c + Vec2::angled(am) * rm,
                    Align2::CENTER_CENTER,
                    display(kb, *key),
                    FontId::proportional(size),
                    color,
                );
            }
        }
        painter.circle_stroke(c, r, Stroke::new(1.5_f32, tint.gamma_multiply(0.6)));
        painter.circle_filled(
            c,
            DEAD_ZONE * r,
            Color32::from_gray(if down && sel.is_none() { 90 } else { 30 }),
        );
        painter.text(
            c,
            Align2::CENTER_CENTER,
            display(kb, rings.center),
            FontId::proportional((r * 0.1).max(11.0)),
            Color32::from_gray(180),
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = self.now();
        let ticked = self.kb.tick(now);
        self.apply(ctx, ticked);

        egui::TopBottomPanel::top("bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Wheel Keyboard");
                ui.separator();
                let current = self.kb.layout().code;
                let mut chosen = current;
                egui::ComboBox::from_id_salt("layout")
                    .selected_text(self.kb.layout().name)
                    .show_ui(ui, |ui| {
                        for l in Layout::all() {
                            ui.selectable_value(&mut chosen, l.code, l.name);
                        }
                    });
                if chosen != current {
                    self.model_dirty = true;
                    self.save_model();
                    if let Some(l) = Layout::by_code(chosen) {
                        self.kb.set_layout(l);
                    }
                    load_model(&mut self.kb);
                    if let Some(dir) = data_dir() {
                        let _ = std::fs::create_dir_all(&dir);
                        let _ = std::fs::write(dir.join("layout.txt"), chosen);
                    }
                    self.status = format!("layout: {}", self.kb.layout().name);
                }
                let mut c = self.kb.config.context;
                egui::ComboBox::from_id_salt("ctx")
                    .selected_text(format!("{c:?}"))
                    .show_ui(ui, |ui| {
                        for x in [
                            Context::Text,
                            Context::Terminal,
                            Context::Finder,
                            Context::Browser,
                        ] {
                            ui.selectable_value(&mut c, x, format!("{x:?}"));
                        }
                    });
                self.kb.config.context = c;
                let mut left = self.kb.config.vowel_hand == Side::Left;
                if ui.checkbox(&mut left, "Vowels on the left").changed() {
                    self.kb.config.vowel_hand = if left { Side::Left } else { Side::Right };
                }
                ui.checkbox(&mut self.kb.config.jis, "JIS");
                let mut secure = self.kb.secure_input();
                if ui.checkbox(&mut secure, "Password field").changed() {
                    self.kb.set_secure_input(secure);
                }
                if ui.button("Copy").clicked() {
                    ctx.copy_text(self.text.clone());
                    self.status = "copied".into();
                }
                if ui.button("Clear").clicked() {
                    self.push_undo();
                    self.text.clear();
                    self.kb.set_text_before_cursor("");
                }
                if ui.button("Forget what it learnt").clicked() {
                    self.kb.clear_model();
                    self.model_dirty = true;
                    self.save_model();
                    self.status = "forgot everything".into();
                }
            });
        });

        egui::TopBottomPanel::top("text").show(ctx, |ui| {
            let edit = egui::TextEdit::multiline(&mut self.text)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .font(FontId::proportional(20.0));
            if ui.add(edit).changed() {
                self.kb.set_text_before_cursor(&self.text);
            }
            ui.horizontal(|ui| {
                let sugg = self.kb.suggestions();
                for (i, s) in sugg.iter().enumerate() {
                    if ui.button(egui::RichText::new(s).size(18.0)).clicked() {
                        let a = self.kb.pick_suggestion(i);
                        self.apply(ctx, a);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.status).monospace().weak());
                    for m in Modifier::ALL {
                        let st = self.kb.modifier(m);
                        if st != ModState::Off
                            && ui
                                .selectable_label(st == ModState::Locked, mod_name(m))
                                .clicked()
                        {
                            self.kb.release_modifier(m);
                        }
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let zone = ui.available_rect_before_wrap();
            ui.allocate_rect(zone, Sense::hover());
            self.zone = zone;
            self.radius = ((zone.height() - TOP_BAR - 4.0) / 2.0)
                .min(zone.width() / 4.2)
                .clamp(56.0, 190.0);
            let r = self.radius;
            let lx = zone.left() + zone.width() * 0.22;
            let rx = zone.left() + zone.width() * 0.78;
            for side in [Side::Left, Side::Right] {
                if !self.kb.is_down(side) {
                    let x = if side == Side::Left { lx } else { rx };
                    self.wheel_centres[idx(side)] = Pos2::new(
                        x.clamp(zone.left() + r + 2.0, zone.right() - r - 2.0),
                        zone.bottom() - r - 2.0,
                    );
                }
            }
            let painter = ui.painter_at(zone);
            painter.line_segment(
                [
                    Pos2::new(zone.center().x, zone.top() + TOP_BAR),
                    Pos2::new(zone.center().x, zone.bottom() - 8.0),
                ],
                Stroke::new(1.0_f32, Color32::from_gray(50)),
            );
            for side in [Side::Left, Side::Right] {
                self.draw_wheel(&painter, side);
            }

            // Buttons above each resting wheel.
            let top = (zone.bottom() - 2.0 * r - TOP_BAR).max(zone.top() + 4.0);
            let more_on = self.kb.layer() == Layer::More;
            let place = |ui: &mut egui::Ui, x: f32, label: &str, on: bool| -> bool {
                let rect = Rect::from_center_size(Pos2::new(x, top + 20.0), Vec2::new(64.0, 36.0));
                ui.put(
                    rect,
                    egui::Button::new(egui::RichText::new(label).size(16.0)).selected(on),
                )
                .clicked()
            };
            let now = self.now();
            if place(
                ui,
                lx - 38.0,
                "Shift",
                self.kb.modifier(Modifier::Shift) != ModState::Off,
            ) {
                let a = self.kb.press(Button::Shift, now);
                self.apply(ctx, a);
            }
            if place(ui, lx + 38.0, "…", more_on) {
                let a = self.kb.press(Button::More, now);
                self.apply(ctx, a);
            }
            if self.kb.config.jis
                && place(
                    ui,
                    rx - 38.0,
                    if self.kb.kana() { "kana" } else { "A" },
                    self.kb.kana(),
                )
            {
                let a = self.kb.press(Button::Kana, now);
                self.apply(ctx, a);
            }
            if place(ui, rx + 38.0, "Enter", false) {
                let a = self.kb.press(Button::Enter, now);
                self.apply(ctx, a);
            }

            // Preview of what lifting the thumb types.
            let preview = [Side::Left, Side::Right]
                .iter()
                .find_map(|s| self.kb.preview(*s, now));
            if let Some(p) = preview {
                let sub = if p.cancel {
                    "cancel".to_string()
                } else if let Some(k) = p.corrected_from {
                    format!(
                        "corrected (you touched {})",
                        self.kb.label(k).to_lowercase()
                    )
                } else {
                    String::new()
                };
                let at = Pos2::new(zone.center().x, zone.top() + TOP_BAR + 40.0);
                painter.text(
                    at,
                    Align2::CENTER_CENTER,
                    display_label(&p.label),
                    FontId::proportional(44.0),
                    Color32::WHITE,
                );
                painter.text(
                    at + Vec2::new(0.0, 36.0),
                    Align2::CENTER_CENTER,
                    sub,
                    FontId::proportional(14.0),
                    Color32::from_gray(170),
                );
            } else {
                painter.text(
                    Pos2::new(zone.center().x, zone.top() + 20.0),
                    Align2::CENTER_CENTER,
                    "Touch a half and slide towards the letter",
                    FontId::monospace(14.0),
                    Color32::from_gray(150),
                );
            }
        });

        self.handle_input(ctx);
        if self.kb.is_down(Side::Left) || self.kb.is_down(Side::Right) {
            ctx.request_repaint();
        } else {
            self.save_model();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.model_dirty = true;
        self.save_model();
    }
}

fn idx(side: Side) -> usize {
    if side == Side::Left { 0 } else { 1 }
}

/// Fills an annular sector centred on `am` (radians, screen coordinates).
fn sector(painter: &egui::Painter, c: Pos2, r1: f32, r2: f32, am: f32, fill: Color32) {
    if fill == Color32::TRANSPARENT {
        return;
    }
    let half = 22.5f32.to_radians();
    let steps = 6;
    for s in 0..steps {
        let a = am - half + 2.0 * half * s as f32 / steps as f32;
        let b = am - half + 2.0 * half * (s + 1) as f32 / steps as f32;
        let pts = vec![
            c + Vec2::angled(a) * r1,
            c + Vec2::angled(a) * r2,
            c + Vec2::angled(b) * r2,
            c + Vec2::angled(b) * r1,
        ];
        painter.add(Shape::convex_polygon(pts, fill, Stroke::NONE));
    }
}

/// Labels drawable with egui's built-in fonts.
fn display(kb: &Keyboard, key: Key) -> String {
    match key {
        Key::Char(_) => kb.label(key),
        other => key_name(other),
    }
}

fn display_label(s: &str) -> String {
    s.replace('⌘', "Cmd+")
        .replace('⌥', "Alt+")
        .replace('⌃', "Ctrl+")
        .replace('⇧', "Shift+")
}

fn key_name(k: Key) -> String {
    match k {
        Key::Char(c) => c.to_string(),
        Key::Space => "space".into(),
        Key::Backspace => "del".into(),
        Key::Enter => "enter".into(),
        Key::Tab => "tab".into(),
        Key::Escape => "esc".into(),
        Key::Delete => "fwd del".into(),
        Key::Up => "↑".into(),
        Key::Down => "↓".into(),
        Key::Left => "←".into(),
        Key::Right => "→".into(),
        Key::Modifier(m) => mod_name(m).into(),
    }
}

fn mod_name(m: Modifier) -> &'static str {
    match m {
        Modifier::Ctrl => "Ctrl",
        Modifier::Alt => "Alt",
        Modifier::Shift => "Shift",
        Modifier::Cmd => "Cmd",
    }
}

fn mod_names(m: wheel_keyboard::Modifiers) -> String {
    Modifier::ALL
        .iter()
        .filter(|x| m.contains(**x))
        .map(|x| format!("{}+", mod_name(*x)))
        .collect()
}

/// Where settings and learnt models live: the platform's per-user data folder.
fn data_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }?;
    Some(base.join("wheel-keyboard"))
}

/// One learnt model per layout, so languages don't mix.
fn model_file(dir: &std::path::Path, code: &str) -> PathBuf {
    dir.join(format!("model-{code}.txt"))
}

fn load_model(kb: &mut Keyboard) {
    let m = data_dir()
        .and_then(|d| std::fs::read_to_string(model_file(&d, kb.layout().code)).ok())
        .and_then(|t| Model::from_text(&t))
        .unwrap_or_default();
    kb.set_model(m);
}
