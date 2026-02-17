    use super::*;

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
