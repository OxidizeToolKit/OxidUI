use std::io::stdout;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Modifier, Style as RaStyle},
    widgets::{Block, Borders, Widget},
};

use termoxide_layout::{
    Style as UiStyle,
    color::{Color as ToxColor, NamedColor},
    coord_mapper::{CoordMapper, MappedRect},
    font::FontStyle as ToxFont,
    layout::{Align, Display, FlexDirection, Justify},
    layout_engine::LayoutEngine,
    number::Float,
    unit::Unit,
};

use termoxide_rendering::{
    event_router::EventRouter,
    render_loop::{App, DirtySender, RenderLoop, dirty_channel},
    renderer::Renderer,
    view_node::{ComponentId, ViewNode},
};

// ── Style helpers ─────────────────────────────────────────────────────────── //

/// Convert a [`MappedRect`] (from `termoxide_layout`) to a ratatui [`Rect`].
fn to_rect(r: MappedRect) -> Rect {
    Rect::new(r.x, r.y, r.width, r.height)
}

/// Convert [`ToxFont`] bitset to ratatui [`Modifier`].
fn font_to_modifier(f: ToxFont) -> Modifier {
    let mut m = Modifier::empty();
    if f.has(ToxFont::BOLD) {
        m |= Modifier::BOLD;
    }
    if f.has(ToxFont::ITALIC) {
        m |= Modifier::ITALIC;
    }
    if f.has(ToxFont::UNDERLINE) {
        m |= Modifier::UNDERLINED;
    }
    if f.has(ToxFont::STRIKETHROUGH) {
        m |= Modifier::CROSSED_OUT;
    }
    if f.has(ToxFont::DIM) {
        m |= Modifier::DIM;
    }
    m
}

/// Convert a [`UiStyle`] to a ratatui [`RaStyle`].
fn ui_style(s: UiStyle) -> RaStyle {
    let mut ra = RaStyle::default();
    if let Some(c) = s.color {
        ra = ra.fg(c.to_ratatui());
    }
    if let Some(c) = s.background {
        ra = ra.bg(c.to_ratatui());
    }
    if let Some(f) = s.font_style {
        ra = ra.add_modifier(font_to_modifier(f));
    }
    ra
}

// Component ids used by this simple app. Keep these stable across frames.
const ID_TITLE: ComponentId = 1;
const ID_DISPLAY: ComponentId = 2;
const ID_HISTORY: ComponentId = 3;
const ID_HINT: ComponentId = 4;

// ── App state ──────────────────────────────────────────────────────────────── //

struct KeyboardApp {
    /// The text describing the last key press.
    last_key: String,
    /// Key history: last N keys pressed (newest last).
    history: Vec<String>,
    /// Wake the render loop whenever a key changes the view.
    dirty_tx: DirtySender,
}

impl KeyboardApp {
    fn new(dirty_tx: DirtySender) -> Self {
        Self {
            last_key: String::new(),
            history: Vec::new(),
            dirty_tx,
        }
    }
}

// ── App trait impl ─────────────────────────────────────────────────────────── //

impl App for KeyboardApp {
    fn build_view(&mut self, viewport: Rect) -> ViewNode {
        let current = self.last_key.clone();

        // ── Layout via termoxide_layout ──────────────────────────────── //
        let mut engine = LayoutEngine::new();

        // Leaf nodes: defined bottom-up (leaves before containers)
        let keybox_w = (viewport.width as i32 - 4).min(62).max(20);
        let keybox_id = engine
            .insert_ui_leaf(
                &UiStyle::new()
                    .with_width(Unit::cells(keybox_w))
                    .with_height(Unit::cells(3)),
            )
            .unwrap();

        // Middle: flex column, grows to fill remaining space, centers keybox
        let middle_id = engine
            .insert_ui_container(
                &UiStyle::new()
                    .with_display(Display::Flex)
                    .with_flex_direction(FlexDirection::Column)
                    .with_flex_grow(Float::ONE)
                    .with_justify_content(Justify::Center)
                    .with_align_items(Align::Center)
                    .with_width(Unit::FULL),
                &[keybox_id],
            )
            .unwrap();

        let title_id = engine
            .insert_ui_leaf(
                &UiStyle::new()
                    .with_width(Unit::FULL)
                    .with_height(Unit::cells(1)),
            )
            .unwrap();

        let history_id = engine
            .insert_ui_leaf(
                &UiStyle::new()
                    .with_width(Unit::FULL)
                    .with_height(Unit::cells(1)),
            )
            .unwrap();

        let hint_id = engine
            .insert_ui_leaf(
                &UiStyle::new()
                    .with_width(Unit::FULL)
                    .with_height(Unit::cells(1)),
            )
            .unwrap();

        // Root: flex column filling the whole viewport
        let root_id = engine
            .insert_ui_container(
                &UiStyle::new()
                    .with_display(Display::Flex)
                    .with_flex_direction(FlexDirection::Column)
                    .with_width(Unit::FULL)
                    .with_height(Unit::FULL),
                &[title_id, middle_id, history_id, hint_id],
            )
            .unwrap();

        engine
            .compute(root_id, viewport.width as f32, viewport.height as f32)
            .unwrap();

        // Convert computed layouts → ratatui Rects
        // Direct children of root are positioned relative to root (offset by viewport origin)
        let origin = (viewport.x, viewport.y);
        let title_rect = to_rect(CoordMapper::map_absolute(
            &engine.layout_of(title_id).unwrap(),
            origin,
        ));
        let middle_mapped =
            CoordMapper::map_absolute(&engine.layout_of(middle_id).unwrap(), origin);
        // keybox is a child of middle → offset by middle's absolute position
        let keybox_rect = to_rect(CoordMapper::map_absolute(
            &engine.layout_of(keybox_id).unwrap(),
            (middle_mapped.x, middle_mapped.y),
        ));
        let history_rect = to_rect(CoordMapper::map_absolute(
            &engine.layout_of(history_id).unwrap(),
            origin,
        ));
        let hint_rect = to_rect(CoordMapper::map_absolute(
            &engine.layout_of(hint_id).unwrap(),
            origin,
        ));

        // ── ViewNode tree ─────────────────────────────────────────────── //
        let title_node = ViewNode::text(
            title_rect,
            " TermOxide — Key Inspector",
            ui_style(
                UiStyle::new()
                    .with_color(ToxColor::Named(NamedColor::Cyan))
                    .with_font_style(ToxFont::BOLD),
            ),
        )
        .with_id(ID_TITLE);

        let key_text = current.clone();
        let display_node = ViewNode::raw(keybox_rect, move |buf, rect| {
            Block::default()
                .borders(Borders::ALL)
                .border_style(ui_style(
                    UiStyle::new().with_color(ToxColor::Named(NamedColor::Yellow)),
                ))
                .title(" Last key ")
                .render(rect, buf);

            let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width.saturating_sub(2), 1);
            let label = if key_text.is_empty() {
                "(press any key)".to_string()
            } else {
                key_text.clone()
            };
            let pad = (inner.width as usize).saturating_sub(label.len()) / 2;
            let padded = format!("{:>width$}{}", "", label, width = pad);
            buf.set_string(
                inner.x,
                inner.y,
                padded,
                ui_style(
                    UiStyle::new()
                        .with_color(ToxColor::Named(NamedColor::Green))
                        .with_font_style(ToxFont::BOLD),
                ),
            );
        })
        .with_id(ID_DISPLAY);

        let hist_text = {
            let keys: Vec<&str> = self
                .history
                .iter()
                .rev()
                .take(20)
                .map(String::as_str)
                .collect();
            format!(" History: {}", keys.join("  "))
        };
        let history_node = ViewNode::text(
            history_rect,
            hist_text,
            ui_style(UiStyle::new().with_color(ToxColor::Named(NamedColor::BrightBlack))),
        )
        .with_id(ID_HISTORY);

        let hint_node = ViewNode::text(
            hint_rect,
            " Press Ctrl-C or Ctrl-D to quit",
            ui_style(UiStyle::new().with_color(ToxColor::Named(NamedColor::BrightBlack))),
        )
        .with_id(ID_HINT);

        ViewNode::container(
            viewport,
            vec![title_node, display_node, history_node, hint_node],
        )
    }

    fn handle_event(&mut self, id: Option<ComponentId>, event: Event) -> bool {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                let prefix = if let Some(cid) = id {
                    format!("c{}:", cid)
                } else {
                    String::new()
                };
                let label = format!("{}{}", prefix, key_label(code, modifiers));
                self.last_key = label.clone();
                self.history.push(label);
                self.dirty_tx.mark();
            }
            Event::Mouse(me) => {
                let prefix = if let Some(cid) = id {
                    format!("c{}:", cid)
                } else {
                    String::new()
                };
                let ml = mouse_label(me.kind, me.modifiers);
                let label = format!("{}{} @({}, {})", prefix, ml, me.column, me.row);
                self.last_key = label.clone();
                self.history.push(label);
                self.dirty_tx.mark();
            }
            _ => {}
        }

        false // never quit here — Ctrl-C is handled by RenderLoop
    }
}

// ── Key label helper ───────────────────────────────────────────────────────── //

fn key_label(code: KeyCode, modifiers: KeyModifiers) -> String {
    let mut parts = Vec::new();

    if modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift".to_string());
    }

    let key = match code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(n) => format!("F{n}"),
        _ => format!("{code:?}"),
    };
    parts.push(key);
    parts.join("-")
}

fn mouse_label(kind: MouseEventKind, modifiers: KeyModifiers) -> String {
    let mut parts = Vec::new();

    if modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift".to_string());
    }

    let key = match kind {
        MouseEventKind::Down(btn) => format!("Down({:?})", btn),
        MouseEventKind::Up(btn) => format!("Up({:?})", btn),
        MouseEventKind::Drag(btn) => format!("Drag({:?})", btn),
        MouseEventKind::Moved => "Moved".to_string(),
        MouseEventKind::ScrollUp => "ScrollUp".to_string(),
        MouseEventKind::ScrollDown => "ScrollDown".to_string(),
        _ => format!("{:?}", kind),
    };

    parts.push(key);
    parts.join("-")
}

// ── Entry point ────────────────────────────────────────────────────────────── //

fn main() {
    let (dirty_tx, dirty_rx) = dirty_channel();

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend).expect("failed to create terminal");
    let renderer = Renderer::new(terminal).expect("failed to create renderer");
    let event_router = EventRouter::new();

    let mut app = KeyboardApp::new(dirty_tx);

    RenderLoop::new(renderer, event_router, dirty_rx)
        .run(&mut app)
        .expect("render loop error");
}
