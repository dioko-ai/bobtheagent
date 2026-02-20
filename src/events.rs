use std::io;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(1);

#[derive(Debug, Default)]
struct LeftMouseState {
    press_position: Option<(u16, u16)>,
    dragged_since_press: bool,
}

fn left_mouse_state() -> &'static Mutex<LeftMouseState> {
    static LEFT_MOUSE_STATE: OnceLock<Mutex<LeftMouseState>> = OnceLock::new();
    LEFT_MOUSE_STATE.get_or_init(|| Mutex::new(LeftMouseState::default()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Tick,
    Quit,
    NextPane,
    PrevPane,
    MoveUp,
    MoveDown,
    CursorLeft,
    CursorRight,
    ScrollChatUp,
    ScrollChatDown,
    ScrollRightUpGlobal,
    ScrollRightDownGlobal,
    InputChar(char),
    Backspace,
    InsertNewline,
    Submit,
    MouseScrollUp,
    MouseScrollDown,
    MouseLeftClick(u16, u16),
}

fn map_key_event(key_event: KeyEvent) -> AppEvent {
    if key_event.kind != KeyEventKind::Press {
        return AppEvent::Tick;
    }

    if key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return AppEvent::Quit;
    }
    if key_event.code == KeyCode::Char('u') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return AppEvent::ScrollRightUpGlobal;
    }
    if key_event.code == KeyCode::Char('d') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return AppEvent::ScrollRightDownGlobal;
    }

    match key_event.code {
        KeyCode::Tab => AppEvent::NextPane,
        KeyCode::BackTab => AppEvent::PrevPane,
        KeyCode::Up
            if key_event.modifiers.contains(KeyModifiers::SHIFT)
                || key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            AppEvent::ScrollChatUp
        }
        KeyCode::Down
            if key_event.modifiers.contains(KeyModifiers::SHIFT)
                || key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            AppEvent::ScrollChatDown
        }
        KeyCode::PageUp => AppEvent::ScrollRightUpGlobal,
        KeyCode::PageDown => AppEvent::ScrollRightDownGlobal,
        KeyCode::Up => AppEvent::MoveUp,
        KeyCode::Down => AppEvent::MoveDown,
        KeyCode::Left => AppEvent::CursorLeft,
        KeyCode::Right => AppEvent::CursorRight,
        KeyCode::Backspace => AppEvent::Backspace,
        KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppEvent::InsertNewline
        }
        KeyCode::Enter => AppEvent::Submit,
        KeyCode::Char(c) => AppEvent::InputChar(c),
        _ => AppEvent::Tick,
    }
}

fn map_mouse_event_kind(kind: MouseEventKind) -> AppEvent {
    match kind {
        MouseEventKind::ScrollUp => AppEvent::MouseScrollUp,
        MouseEventKind::ScrollDown => AppEvent::MouseScrollDown,
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => AppEvent::MouseLeftClick(0, 0),
        _ => AppEvent::Tick,
    }
}

fn map_mouse_event(mouse_event: crossterm::event::MouseEvent) -> AppEvent {
    match mouse_event.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let mut state = left_mouse_state().lock().unwrap_or_else(|e| e.into_inner());
            state.press_position = Some((mouse_event.column, mouse_event.row));
            state.dragged_since_press = false;
            AppEvent::Tick
        }
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            let mut state = left_mouse_state().lock().unwrap_or_else(|e| e.into_inner());
            if state.press_position.is_some() {
                state.dragged_since_press = true;
            }
            AppEvent::Tick
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            let mut state = left_mouse_state().lock().unwrap_or_else(|e| e.into_inner());
            let should_emit_click = state.press_position.take().is_some() && !state.dragged_since_press;
            state.dragged_since_press = false;
            if should_emit_click {
                AppEvent::MouseLeftClick(mouse_event.column, mouse_event.row)
            } else {
                AppEvent::Tick
            }
        }
        _ => map_mouse_event_kind(mouse_event.kind),
    }
}

#[cfg(test)]
fn reset_left_mouse_state_for_tests() {
    let mut state = left_mouse_state().lock().unwrap_or_else(|e| e.into_inner());
    state.press_position = None;
    state.dragged_since_press = false;
}

pub fn next_event() -> io::Result<AppEvent> {
    if event::poll(EVENT_POLL_INTERVAL)? {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                return Ok(map_key_event(key_event));
            }
            Event::Mouse(mouse_event) => {
                return Ok(map_mouse_event(mouse_event));
            }
            _ => {}
        }
    }

    Ok(AppEvent::Tick)
}

pub fn has_pending_input() -> io::Result<bool> {
    event::poll(Duration::from_millis(0))
}

#[cfg(test)]
#[path = "../tests/unit/events_tests.rs"]
mod tests;
