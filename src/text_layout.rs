#[derive(Debug, Clone)]
pub struct WrappedText {
    pub rendered: String,
    pub positions: Vec<(u16, u16)>,
    pub line_count: u16,
}

pub fn wrap_word_with_positions(text: &str, width: u16) -> WrappedText {
    let width = width.max(1);
    let width_usize = width as usize;
    let chars: Vec<char> = text.chars().collect();
    let word_lengths = word_lengths_from_each_index(&chars);
    let mut rendered = String::new();
    let mut positions = Vec::with_capacity(chars.len() + 1);
    let mut line = 0u16;
    let mut col = 0usize;

    positions.push((line, 0));

    for (idx, ch) in chars.iter().copied().enumerate() {
        if ch == '\n' {
            rendered.push('\n');
            line = line.saturating_add(1);
            col = 0;
            positions.push((line, 0));
            continue;
        }

        if should_wrap_before_word(&chars, &word_lengths, idx, col, width_usize) {
            rendered.push('\n');
            line = line.saturating_add(1);
            col = 0;
        } else if col >= width_usize {
            rendered.push('\n');
            line = line.saturating_add(1);
            col = 0;
        }

        rendered.push(ch);
        col = col.saturating_add(1);
        if col >= width_usize {
            rendered.push('\n');
            line = line.saturating_add(1);
            col = 0;
        }

        positions.push((line, col as u16));
    }

    let line_count = positions
        .iter()
        .map(|(l, _)| *l)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    WrappedText {
        rendered,
        positions,
        line_count,
    }
}

fn word_lengths_from_each_index(chars: &[char]) -> Vec<usize> {
    let mut out = vec![0usize; chars.len()];
    let mut run = 0usize;
    for (idx, ch) in chars.iter().copied().enumerate().rev() {
        if ch == '\n' || ch.is_whitespace() {
            run = 0;
            out[idx] = 0;
            continue;
        }
        run = run.saturating_add(1);
        out[idx] = run;
    }
    out
}

fn should_wrap_before_word(
    chars: &[char],
    word_lengths: &[usize],
    idx: usize,
    col: usize,
    width: usize,
) -> bool {
    if col == 0 {
        return false;
    }
    let ch = chars[idx];
    if ch.is_whitespace() {
        return false;
    }
    if idx > 0 {
        let prev = chars[idx - 1];
        if !prev.is_whitespace() && prev != '\n' {
            return false;
        }
    }

    let word_len = word_lengths[idx];

    word_len <= width && col.saturating_add(word_len) > width
}

#[cfg(test)]
#[path = "../tests/unit/text_layout_tests.rs"]
mod tests;
