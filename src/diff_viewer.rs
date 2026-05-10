use gpui::prelude::*;
use gpui::{AnyElement, ParentElement, SharedString, Styled, div, px};

const BG_ADDED: u32 = 0x1A3A1A;
const BG_REMOVED: u32 = 0x3A1A1A;
const BG_CONTEXT: u32 = 0x222222;
const BG_EMPTY: u32 = 0x141414;
const BG_HUNK: u32 = 0x1A2A3A;
const TEXT_ADDED: u32 = 0x6BCB77;
const TEXT_REMOVED: u32 = 0xE74C3C;
const TEXT_CONTEXT: u32 = 0xCCCCCC;
const TEXT_HUNK: u32 = 0x6C9FD8;
const TEXT_LINE_NO: u32 = 0x555555;
const LINE_NO_W: f32 = 40.0;
const BORDER: u32 = 0x333333;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub line_no: Option<usize>,
}

#[derive(Clone)]
pub enum SideBySideRow {
    Hunk {
        header: String,
    },
    Line {
        left: Option<DiffLine>,
        right: Option<DiffLine>,
    },
}

pub fn parse_diff(raw: &str) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();
    let mut pending_removes: Vec<DiffLine> = Vec::new();
    let mut old_no: Option<usize> = None;
    let mut new_no: Option<usize> = None;
    let mut in_content = false;

    for line in raw.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        }
        if line.starts_with("@@") {
            flush_removes(&mut pending_removes, &mut rows);
            if let Some((os, ns)) = parse_hunk_positions(line) {
                old_no = Some(os);
                new_no = Some(ns);
            }
            rows.push(SideBySideRow::Hunk {
                header: line.to_string(),
            });
            in_content = true;
            continue;
        }

        if !in_content {
            continue;
        }

        if line.starts_with('-') {
            let no = old_no;
            old_no = old_no.map(|n| n + 1);
            pending_removes.push(DiffLine {
                kind: DiffLineKind::Removed,
                content: line[1..].to_string(),
                line_no: no,
            });
        } else if line.starts_with('+') {
            let no = new_no;
            new_no = new_no.map(|n| n + 1);
            let added = DiffLine {
                kind: DiffLineKind::Added,
                content: line[1..].to_string(),
                line_no: no,
            };
            if let Some(removed) = pending_removes.pop() {
                rows.push(SideBySideRow::Line {
                    left: Some(removed),
                    right: Some(added),
                });
            } else {
                rows.push(SideBySideRow::Line {
                    left: None,
                    right: Some(added),
                });
            }
        } else {
            flush_removes(&mut pending_removes, &mut rows);
            let content = line.strip_prefix(' ').unwrap_or(line);
            let o = old_no;
            let n = new_no;
            old_no = o.map(|v| v + 1);
            new_no = n.map(|v| v + 1);
            rows.push(SideBySideRow::Line {
                left: Some(DiffLine {
                    kind: DiffLineKind::Context,
                    content: content.to_string(),
                    line_no: o,
                }),
                right: Some(DiffLine {
                    kind: DiffLineKind::Context,
                    content: content.to_string(),
                    line_no: n,
                }),
            });
        }
    }

    flush_removes(&mut pending_removes, &mut rows);
    rows
}

fn flush_removes(pending: &mut Vec<DiffLine>, rows: &mut Vec<SideBySideRow>) {
    for rl in pending.drain(..) {
        rows.push(SideBySideRow::Line {
            left: Some(rl),
            right: None,
        });
    }
}

fn parse_hunk_positions(header: &str) -> Option<(usize, usize)> {
    let trimmed = header.trim_start_matches('@').trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let old_start: usize = parts[0]
        .trim_start_matches('-')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    let new_start: usize = parts[1]
        .trim_start_matches('+')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    Some((old_start, new_start))
}

pub fn is_binary_or_error(raw: &str) -> bool {
    raw.contains("binary file") || raw.contains("not found in diff")
}

pub fn render_side_by_side(rows: &[SideBySideRow]) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .border_t_1()
        .border_color(gpui::rgb(BORDER))
        .child(render_column_header())
        .children(rows.iter().enumerate().map(|(i, row)| match row {
            SideBySideRow::Hunk { header } => render_hunk_row(header),
            SideBySideRow::Line { left, right } => render_line_row(i, left, right),
        }))
        .into_any()
}

fn render_column_header() -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_row()
        .bg(gpui::rgb(0x252525))
        .border_b_1()
        .border_color(gpui::rgb(BORDER))
        .child(
            div()
                .flex_1()
                .px(px(8.0))
                .py(px(2.0))
                .text_color(gpui::rgb(0x888888))
                .text_size(px(10.0))
                .font_weight(gpui::FontWeight::BOLD)
                .font_family("monospace")
                .border_r_1()
                .border_color(gpui::rgb(BORDER))
                .child("OLD"),
        )
        .child(
            div()
                .flex_1()
                .px(px(8.0))
                .py(px(2.0))
                .text_color(gpui::rgb(0x888888))
                .text_size(px(10.0))
                .font_weight(gpui::FontWeight::BOLD)
                .font_family("monospace")
                .child("NEW"),
        )
        .into_any()
}

fn render_hunk_row(header: &str) -> AnyElement {
    div()
        .w_full()
        .px(px(8.0))
        .py(px(2.0))
        .bg(gpui::rgb(BG_HUNK))
        .text_color(gpui::rgb(TEXT_HUNK))
        .text_size(px(11.0))
        .font_family("monospace")
        .child(header.to_string())
        .into_any()
}

fn render_line_row(index: usize, left: &Option<DiffLine>, right: &Option<DiffLine>) -> AnyElement {
    div()
        .id(SharedString::from(format!("diff-row-{}", index)))
        .w_full()
        .flex()
        .flex_row()
        .child(render_half(left, true))
        .child(render_half(right, false))
        .into_any()
}

fn render_half(line: &Option<DiffLine>, is_left: bool) -> AnyElement {
    match line {
        Some(dl) => {
            let bg = match dl.kind {
                DiffLineKind::Added => BG_ADDED,
                DiffLineKind::Removed => BG_REMOVED,
                DiffLineKind::Context => BG_CONTEXT,
            };
            let color = match dl.kind {
                DiffLineKind::Added => TEXT_ADDED,
                DiffLineKind::Removed => TEXT_REMOVED,
                DiffLineKind::Context => TEXT_CONTEXT,
            };
            let prefix = match dl.kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Context => " ",
            };
            let no_str = dl.line_no.map(|n| n.to_string()).unwrap_or_default();
            build_half(
                bg,
                color,
                prefix.to_string(),
                dl.content.clone(),
                no_str,
                is_left,
            )
        }
        None => build_half(
            BG_EMPTY,
            0x000000,
            String::new(),
            String::new(),
            String::new(),
            is_left,
        ),
    }
}

fn build_half(
    bg: u32,
    text_color: u32,
    prefix: String,
    content: String,
    line_no: String,
    is_left: bool,
) -> AnyElement {
    let mut left = div()
        .flex_1()
        .flex()
        .flex_row()
        .bg(gpui::rgb(bg))
        .child(
            div()
                .w(px(LINE_NO_W))
                .px(px(4.0))
                .text_color(gpui::rgb(TEXT_LINE_NO))
                .text_size(px(11.0))
                .font_family("monospace")
                .child(line_no),
        )
        .child(
            div()
                .flex_1()
                .px(px(4.0))
                .text_color(gpui::rgb(text_color))
                .text_size(px(11.0))
                .font_family("monospace")
                .overflow_hidden()
                .whitespace_nowrap()
                .child(format!("{}{}", prefix, content)),
        );

    if is_left {
        left = left.border_r_1().border_color(gpui::rgb(BORDER));
    }

    left.into_any()
}
