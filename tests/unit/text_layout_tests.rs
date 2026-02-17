    use super::*;

    #[test]
    fn wraps_by_word_when_possible() {
        let wrapped = wrap_word_with_positions("hello world", 6);
        assert_eq!(wrapped.rendered, "hello \nworld");
        assert_eq!(wrapped.line_count, 2);
    }

    #[test]
    fn breaks_long_words_when_needed() {
        let wrapped = wrap_word_with_positions("abcdefghij", 4);
        assert_eq!(wrapped.rendered, "abcd\nefgh\nij");
        assert_eq!(wrapped.line_count, 3);
    }

    #[test]
    fn produces_cursor_positions_for_each_char_boundary() {
        let wrapped = wrap_word_with_positions("abc def", 4);
        assert_eq!(wrapped.positions.len(), "abc def".chars().count() + 1);
        assert_eq!(wrapped.positions[0], (0, 0));
    }
