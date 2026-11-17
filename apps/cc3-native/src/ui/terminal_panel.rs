//! Bottom terminal drawer: a vt100 screen fed by the server's `terminal.attach` stream; keys go to `terminal.write`.
use dioxus::prelude::*;

pub const COLS: u16 = 110;
pub const ROWS: u16 = 14;

/// One styled run of a screen row.
#[derive(Clone, PartialEq)]
pub struct Run {
    pub text: String,
    pub style: String,
}

fn color(c: vt100::Color, default: &str) -> String {
    const BASE: [&str; 16] = [
        "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5", "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
    ];
    match c {
        vt100::Color::Default => default.to_string(),
        vt100::Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        vt100::Color::Idx(i) if i < 16 => BASE[i as usize].to_string(),
        vt100::Color::Idx(i) if i >= 232 => {
            let v = 8 + (i - 232) as u32 * 10;
            format!("#{v:02x}{v:02x}{v:02x}")
        }
        vt100::Color::Idx(i) => {
            let n = (i - 16) as u32;
            let level = |x: u32| if x == 0 { 0 } else { 55 + x * 40 };
            format!("#{:02x}{:02x}{:02x}", level(n / 36), level(n / 6 % 6), level(n % 6))
        }
    }
}

/// Rows of the screen as runs of equal attributes.
pub fn screen_runs(screen: &vt100::Screen) -> Vec<Vec<Run>> {
    let (rows, cols) = screen.size();
    let (cursor_row, cursor_col) = screen.cursor_position();
    let show_cursor = !screen.hide_cursor();
    (0..rows)
        .map(|r| {
            let mut runs: Vec<Run> = Vec::new();
            for c in 0..cols {
                let Some(cell) = screen.cell(r, c) else { continue };
                if cell.is_wide_continuation() {
                    continue;
                }
                let (mut fg, mut bg) = (color(cell.fgcolor(), "var(--foreground)"), color(cell.bgcolor(), "transparent"));
                if cell.inverse() {
                    std::mem::swap(&mut fg, &mut bg);
                    if bg == "transparent" {
                        bg = "var(--foreground)".into();
                    }
                    if fg == "var(--foreground)" {
                        fg = "var(--background)".into();
                    }
                }
                if show_cursor && r == cursor_row && c == cursor_col {
                    fg = "var(--background)".into();
                    bg = "var(--foreground)".into();
                }
                let style = format!("color: {fg}; background-color: {bg};{}{}", if cell.bold() { " font-weight: 700;" } else { "" }, if cell.underline() { " text-decoration: underline;" } else { "" });
                let text = if cell.has_contents() { cell.contents() } else { " ".to_string() };
                match runs.last_mut() {
                    Some(last) if last.style == style => last.text.push_str(&text),
                    _ => runs.push(Run { text, style }),
                }
            }
            runs
        })
        .collect()
}

/// Bytes a key press sends to the shell (`None`: not a terminal key).
pub fn key_bytes(key: &str, ctrl: bool, alt: bool) -> Option<String> {
    let named = match key {
        "Enter" => "\r",
        "Backspace" => "\x7f",
        "Tab" => "\t",
        "Escape" => "\x1b",
        "ArrowUp" => "\x1b[A",
        "ArrowDown" => "\x1b[B",
        "ArrowRight" => "\x1b[C",
        "ArrowLeft" => "\x1b[D",
        "Home" => "\x1b[H",
        "End" => "\x1b[F",
        "Delete" => "\x1b[3~",
        "PageUp" => "\x1b[5~",
        "PageDown" => "\x1b[6~",
        _ => "",
    };
    if !named.is_empty() {
        return Some(named.to_string());
    }
    let mut chars = key.chars();
    let (c, rest) = (chars.next()?, chars.next());
    if rest.is_some() {
        return None;
    }
    if ctrl && c.is_ascii_alphabetic() {
        return Some(((c.to_ascii_uppercase() as u8) & 0x1f) as char).map(String::from);
    }
    if alt {
        return Some(format!("\x1b{c}"));
    }
    Some(c.to_string())
}

#[component]
pub fn TerminalPanel(rows: Vec<Vec<Run>>, status: String, onkey: EventHandler<(String, bool, bool)>) -> Element {
    rsx! {
        div {
            "data-slot": "terminal-panel",
            "tabindex": "0",
            "class": "flex flex-col border-t bg-background text-foreground outline-none",
            "style": "position: fixed; left: 256px; right: 0; bottom: 0; height: 320px; z-index: 30;",
            onmounted: move |e| {
                spawn(async move {
                    let _ = e.set_focus(true).await;
                });
            },
            onkeydown: move |e| {
                let m = e.modifiers();
                let key = e.key().to_string();
                if (m.ctrl() && key.eq_ignore_ascii_case("k")) || key == "Escape" {
                    return;
                }
                e.prevent_default();
                onkey.call((key, m.ctrl(), m.alt()));
            },
            div {
                "class": "flex h-9 shrink-0 items-center gap-2 border-b px-3 text-sm",
                span { "class": "font-medium", "Terminal" }
                span { "class": "text-xs font-normal text-muted-foreground", "{status}" }
            }
            div {
                "class": "min-h-0 flex-1 overflow-hidden px-3 py-2 font-mono",
                "style": "font-size: 12px; line-height: 20px;",
                for (i, row) in rows.iter().enumerate() {
                    div {
                        key: "{i}",
                        "class": "whitespace-pre",
                        "style": "height: 20px;",
                        for (j, run) in row.iter().enumerate() {
                            span { key: "{j}", "style": "{run.style}", "{run.text}" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_runs_follow_ansi_colors() {
        let mut parser = vt100::Parser::new(ROWS, COLS, 0);
        parser.process(b"plain \x1b[31mred\x1b[0m\r\nsecond");
        let rows = screen_runs(parser.screen());
        assert_eq!(rows[0][0].text, "plain ");
        assert!(rows[0][1].style.contains("#cd3131"));
        assert!(rows[1].iter().any(|r| r.text.contains("second")));
    }

    #[test]
    fn keys_map_to_bytes() {
        assert_eq!(key_bytes("Enter", false, false).as_deref(), Some("\r"));
        assert_eq!(key_bytes("c", true, false).as_deref(), Some("\x03"));
        assert_eq!(key_bytes("ArrowUp", false, false).as_deref(), Some("\x1b[A"));
        assert_eq!(key_bytes("a", false, false).as_deref(), Some("a"));
        assert_eq!(key_bytes("Shift", false, false), None);
    }
}
