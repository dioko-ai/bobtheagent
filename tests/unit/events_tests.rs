use super::*;
use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};

fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn maps_navigation_and_quit_keys() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
        AppEvent::NextPane
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
        AppEvent::PrevPane
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        AppEvent::Quit
    );
}

#[test]
fn maps_escape_to_tick() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        AppEvent::Tick
    );
}

#[test]
fn maps_movement_keys() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        AppEvent::MoveDown
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        AppEvent::MoveUp
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        AppEvent::CursorLeft
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        AppEvent::CursorRight
    );
}

#[test]
fn maps_shift_up_down_to_chat_scroll() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)),
        AppEvent::ScrollChatUp
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT)),
        AppEvent::ScrollChatDown
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)),
        AppEvent::ScrollChatUp
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL)),
        AppEvent::ScrollChatDown
    );
}

#[test]
fn maps_page_up_down_to_right_pane_global_scroll() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
        AppEvent::ScrollRightUpGlobal
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
        AppEvent::ScrollRightDownGlobal
    );
}

#[test]
fn maps_ctrl_u_d_to_right_pane_scroll_regardless_of_focus() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
        AppEvent::ScrollRightUpGlobal
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)),
        AppEvent::ScrollRightDownGlobal
    );
}

#[test]
fn maps_text_editing_keys() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
        AppEvent::InputChar('k')
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        AppEvent::Backspace
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        AppEvent::Submit
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)),
        AppEvent::InsertNewline
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)),
        AppEvent::InsertNewline
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)),
        AppEvent::InsertNewline
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::NONE)),
        AppEvent::InsertNewline
    );
}

#[test]
fn maps_left_with_ctrl_or_alt_to_word_left() {
    assert_eq!(
        map_key_event(KeyEvent::new(
            KeyCode::Left,
            KeyModifiers::CONTROL
        )),
        AppEvent::CursorLeftWord
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)),
        AppEvent::CursorLeftWord
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT)),
        AppEvent::CursorLeft
    );
    assert_eq!(
        map_key_event(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::CONTROL
        )),
        AppEvent::CursorRightWord
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)),
        AppEvent::CursorRightWord
    );
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT)),
        AppEvent::CursorRight
    );
}

#[test]
fn maps_unhandled_keys_to_tick() {
    assert_eq!(
        map_key_event(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
        AppEvent::Tick
    );
}

#[test]
fn maps_mouse_wheel_to_active_scroll_events() {
    assert_eq!(
        map_mouse_event_kind(MouseEventKind::ScrollUp),
        AppEvent::MouseScrollUp
    );
    assert_eq!(
        map_mouse_event_kind(MouseEventKind::ScrollDown),
        AppEvent::MouseScrollDown
    );
}

#[test]
fn maps_left_click_mouse_down() {
    assert_eq!(
        map_mouse_event_kind(MouseEventKind::Down(crossterm::event::MouseButton::Left)),
        AppEvent::MouseLeftClick(0, 0)
    );
}

#[test]
fn left_click_emits_only_on_release_without_drag() {
    reset_left_mouse_state_for_tests();

    assert_eq!(
        map_mouse_event(mouse_event(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            8,
            4
        )),
        AppEvent::Tick
    );
    assert_eq!(
        map_mouse_event(mouse_event(
            MouseEventKind::Up(crossterm::event::MouseButton::Left),
            10,
            6
        )),
        AppEvent::MouseLeftClick(10, 6)
    );
}

#[test]
fn left_drag_sequence_does_not_emit_click() {
    reset_left_mouse_state_for_tests();

    assert_eq!(
        map_mouse_event(mouse_event(
            MouseEventKind::Down(crossterm::event::MouseButton::Left),
            8,
            4
        )),
        AppEvent::Tick
    );
    assert_eq!(
        map_mouse_event(mouse_event(
            MouseEventKind::Drag(crossterm::event::MouseButton::Left),
            20,
            10
        )),
        AppEvent::Tick
    );
    assert_eq!(
        map_mouse_event(mouse_event(
            MouseEventKind::Up(crossterm::event::MouseButton::Left),
            20,
            10
        )),
        AppEvent::Tick
    );
}
