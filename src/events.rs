use std::io;
use std::time::Duration;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(1);

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

pub fn next_event() -> io::Result<AppEvent> {
    if event::poll(EVENT_POLL_INTERVAL)? {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                return Ok(map_key_event(key_event));
            }
            Event::Mouse(mouse_event) => {
                if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse_event.kind
                {
                    return Ok(AppEvent::MouseLeftClick(
                        mouse_event.column,
                        mouse_event.row,
                    ));
                }
                return Ok(map_mouse_event_kind(mouse_event.kind));
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
