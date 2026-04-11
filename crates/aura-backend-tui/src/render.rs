//! Render HIR views to ratatui widgets.

use crate::eval::{Scope, eval_expr};
use crate::value::Value;
use aura_core::design::ResolvedDesign;
use aura_core::hir::*;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

/// Identifier of an interactive element — its index within the focusable list.
pub type FocusId = usize;

/// A focusable element discovered during the render pass.
#[derive(Debug, Clone)]
pub enum Focusable {
    Button { action: HIRActionExpr },
    TextField { binding: String },
    Checkbox { binding: String },
    Toggle { binding: String },
}

/// Per-frame render context: focus list + currently focused element.
pub struct RenderCtx<'a> {
    pub state: &'a Scope,
    pub components: &'a [HIRComponent],
    pub focus_list: Vec<Focusable>,
    pub focused: Option<FocusId>,
}

impl<'a> RenderCtx<'a> {
    pub fn new(state: &'a Scope, components: &'a [HIRComponent], focused: Option<FocusId>) -> Self {
        Self {
            state,
            components,
            focus_list: Vec::new(),
            focused,
        }
    }

    fn next_focus_id(&self) -> FocusId {
        self.focus_list.len()
    }

    fn is_focused(&self, id: FocusId) -> bool {
        self.focused == Some(id)
    }
}

/// Top-level entry point: render a screen's root view into an area.
pub fn render_view(frame: &mut Frame, area: Rect, view: &HIRView, ctx: &mut RenderCtx) {
    emit_view(frame, area, view, ctx);
}

fn emit_view(frame: &mut Frame, area: Rect, view: &HIRView, ctx: &mut RenderCtx) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    match view {
        HIRView::Column(layout) => {
            emit_layout(frame, area, &layout.children, Direction::Vertical, ctx)
        }
        HIRView::Row(layout) => {
            emit_layout(frame, area, &layout.children, Direction::Horizontal, ctx)
        }
        HIRView::Stack(layout) => {
            // Stack overlays — render last child on top of the area.
            if let Some(last) = layout.children.last() {
                emit_view(frame, area, last, ctx);
            }
        }
        HIRView::Grid(layout) => {
            emit_layout(frame, area, &layout.children, Direction::Horizontal, ctx)
        }
        HIRView::Scroll(layout) => {
            emit_layout(frame, area, &layout.children, Direction::Vertical, ctx)
        }
        HIRView::Wrap(layout) => {
            emit_layout(frame, area, &layout.children, Direction::Horizontal, ctx)
        }
        HIRView::Group(children) => emit_layout(frame, area, children, Direction::Vertical, ctx),

        HIRView::Text(text) => {
            let content = value_to_string(&eval_expr(&text.content, ctx.state, &Scope::new()));
            let style = design_to_style(&text.design);
            let alignment = text_alignment(&text.design);
            let para = Paragraph::new(Line::from(Span::styled(content, style)))
                .alignment(alignment)
                .wrap(Wrap { trim: true });
            frame.render_widget(para, area);
        }
        HIRView::Heading(heading) => {
            let content = value_to_string(&eval_expr(&heading.content, ctx.state, &Scope::new()));
            let mut style = design_to_style(&heading.design);
            style = style.add_modifier(Modifier::BOLD);
            if style.fg.is_none() {
                style = style.fg(Color::White);
            }
            let alignment = text_alignment(&heading.design);
            let para =
                Paragraph::new(Line::from(Span::styled(content, style))).alignment(alignment);
            frame.render_widget(para, area);
        }
        HIRView::Button(button) => {
            let id = ctx.next_focus_id();
            let focused = ctx.is_focused(id);
            ctx.focus_list.push(Focusable::Button {
                action: button.action.clone(),
            });

            let label = value_to_string(&eval_expr(&button.label, ctx.state, &Scope::new()));
            let display = if focused {
                format!("▶ {} ◀", label)
            } else {
                format!("  {}  ", label)
            };
            let mut style = design_to_style(&button.design);
            if style.fg.is_none() {
                style = style.fg(Color::Cyan);
            }
            if focused {
                style = style.add_modifier(Modifier::BOLD).bg(Color::DarkGray);
            }
            let border_style = if focused {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);
            let para = Paragraph::new(display)
                .style(style)
                .alignment(ratatui::layout::Alignment::Center)
                .block(block);
            frame.render_widget(para, area);
        }
        HIRView::TextField(field) => {
            let id = ctx.next_focus_id();
            let focused = ctx.is_focused(id);
            ctx.focus_list.push(Focusable::TextField {
                binding: field.binding.clone(),
            });

            let current = ctx
                .state
                .get(&field.binding)
                .map(|v| v.to_string())
                .unwrap_or_default();
            let (text, style) = if current.is_empty() {
                (
                    field
                        .placeholder
                        .clone()
                        .unwrap_or_else(|| "...".to_string()),
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                (current, Style::default().fg(Color::White))
            };
            let border_style = if focused {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let display = if focused {
                format!("{}▎", text)
            } else {
                text
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);
            let para = Paragraph::new(display).style(style).block(block);
            frame.render_widget(para, area);
        }
        HIRView::TextArea(field) => {
            let id = ctx.next_focus_id();
            let focused = ctx.is_focused(id);
            ctx.focus_list.push(Focusable::TextField {
                binding: field.binding.clone(),
            });

            let current = ctx
                .state
                .get(&field.binding)
                .map(|v| v.to_string())
                .unwrap_or_default();
            let (text, style) = if current.is_empty() {
                (
                    field
                        .placeholder
                        .clone()
                        .unwrap_or_else(|| "...".to_string()),
                    Style::default().fg(Color::DarkGray),
                )
            } else {
                (current, Style::default().fg(Color::White))
            };
            let border_style = if focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);
            let para = Paragraph::new(text)
                .style(style)
                .wrap(Wrap { trim: false })
                .block(block);
            frame.render_widget(para, area);
        }
        HIRView::Checkbox(cb) => {
            let id = ctx.next_focus_id();
            let focused = ctx.is_focused(id);
            ctx.focus_list.push(Focusable::Checkbox {
                binding: cb.binding.clone(),
            });
            let checked = ctx
                .state
                .get(&cb.binding)
                .map(|v| v.as_bool())
                .unwrap_or(false);
            let marker = if checked { "[x]" } else { "[ ]" };
            let mut style = Style::default();
            if focused {
                style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
            }
            let para = Paragraph::new(format!("{} {}", marker, cb.binding)).style(style);
            frame.render_widget(para, area);
        }
        HIRView::Toggle(toggle) => {
            let id = ctx.next_focus_id();
            let focused = ctx.is_focused(id);
            ctx.focus_list.push(Focusable::Toggle {
                binding: toggle.binding.clone(),
            });
            let on = ctx
                .state
                .get(&toggle.binding)
                .map(|v| v.as_bool())
                .unwrap_or(false);
            let marker = if on { "●" } else { "○" };
            let label = toggle.label.as_deref().unwrap_or(&toggle.binding);
            let mut style = if on {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            if focused {
                style = style.add_modifier(Modifier::BOLD);
            }
            let para = Paragraph::new(format!("{} {}", marker, label)).style(style);
            frame.render_widget(para, area);
        }
        HIRView::Slider(slider) => {
            let current = ctx
                .state
                .get(&slider.binding)
                .map(|v| v.as_float())
                .unwrap_or(slider.min);
            let ratio =
                ((current - slider.min) / (slider.max - slider.min).max(0.0001)).clamp(0.0, 1.0);
            let gauge = Gauge::default()
                .ratio(ratio)
                .gauge_style(Style::default().fg(Color::Blue))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(gauge, area);
        }
        HIRView::Divider(_) => {
            let block = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, area);
        }
        HIRView::Spacer => {}
        HIRView::Icon(icon) => {
            let name = value_to_string(&eval_expr(&icon.name, ctx.state, &Scope::new()));
            let emoji = icon_to_emoji(&name);
            frame.render_widget(Paragraph::new(emoji), area);
        }
        HIRView::Badge(badge) => {
            let content = value_to_string(&eval_expr(&badge.content, ctx.state, &Scope::new()));
            let para = Paragraph::new(format!("[{}]", content)).style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(para, area);
        }
        HIRView::Progress(progress) => {
            let v = eval_expr(&progress.value, ctx.state, &Scope::new()).as_float();
            let ratio = v.clamp(0.0, 1.0);
            let gauge = Gauge::default()
                .ratio(ratio)
                .gauge_style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(gauge, area);
        }
        HIRView::Image(_) => {
            let para = Paragraph::new("[image]").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(para, area);
        }
        HIRView::Avatar(_) => {
            let para = Paragraph::new("(@)").style(Style::default().fg(Color::Cyan));
            frame.render_widget(para, area);
        }
        HIRView::Conditional(cond) => {
            let result = eval_expr(&cond.condition, ctx.state, &Scope::new());
            if result.as_bool() {
                emit_view(frame, area, &cond.then_view, ctx);
            } else if let Some(ref else_view) = cond.else_view {
                emit_view(frame, area, else_view, ctx);
            }
        }
        HIRView::Each(each) => {
            let iter = eval_expr(&each.iterable, ctx.state, &Scope::new());
            let items = match iter {
                Value::List(v) => v,
                _ => Vec::new(),
            };
            if items.is_empty() {
                return;
            }
            // Give each item an equal share of vertical space.
            let constraints: Vec<Constraint> = items
                .iter()
                .map(|_| Constraint::Length(estimate_height(&each.body)))
                .collect();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);
            for (i, item) in items.iter().enumerate() {
                let mut locals = Scope::new();
                locals.insert(each.item_name.clone(), item.clone());
                if let Some(ref idx_name) = each.index_name {
                    locals.insert(idx_name.clone(), Value::Int(i as i64));
                }
                emit_view_with_locals(frame, chunks[i], &each.body, ctx, &locals);
            }
        }
        HIRView::Switch(switch) => {
            if let Some(first_case) = switch.cases.first() {
                emit_view(frame, area, &first_case.view, ctx);
            }
        }
        HIRView::ComponentRef(comp_ref) => {
            if let Some(comp) = ctx.components.iter().find(|c| c.name == comp_ref.name) {
                // Clone the component's view so we can borrow ctx mutably.
                let view = comp.view.clone();
                emit_view(frame, area, &view, ctx);
            }
        }
        _ => {}
    }
}

/// Render a view with extra local variables (for `each` loops).
fn emit_view_with_locals(
    frame: &mut Frame,
    area: Rect,
    view: &HIRView,
    ctx: &mut RenderCtx,
    locals: &Scope,
) {
    // For text-bearing widgets, evaluate against the locals + state.
    match view {
        HIRView::Text(text) => {
            let content = value_to_string(&eval_expr(&text.content, ctx.state, locals));
            let style = design_to_style(&text.design);
            let para =
                Paragraph::new(Line::from(Span::styled(content, style))).wrap(Wrap { trim: true });
            frame.render_widget(para, area);
        }
        HIRView::Heading(heading) => {
            let content = value_to_string(&eval_expr(&heading.content, ctx.state, locals));
            let style = design_to_style(&heading.design).add_modifier(Modifier::BOLD);
            let para = Paragraph::new(Line::from(Span::styled(content, style)));
            frame.render_widget(para, area);
        }
        _ => emit_view(frame, area, view, ctx),
    }
}

fn emit_layout(
    frame: &mut Frame,
    area: Rect,
    children: &[HIRView],
    direction: Direction,
    ctx: &mut RenderCtx,
) {
    if children.is_empty() {
        return;
    }
    let constraints = build_constraints(children, direction);
    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area);
    for (chunk, child) in chunks.iter().zip(children.iter()) {
        emit_view(frame, *chunk, child, ctx);
    }
}

fn build_constraints(children: &[HIRView], direction: Direction) -> Vec<Constraint> {
    // Horizontal: proportional widths, giving Spacers remaining space.
    if direction == Direction::Horizontal {
        let non_spacer_count = children
            .iter()
            .filter(|c| !matches!(c, HIRView::Spacer))
            .count()
            .max(1);
        return children
            .iter()
            .map(|c| {
                if matches!(c, HIRView::Spacer) {
                    Constraint::Fill(1)
                } else {
                    Constraint::Ratio(1, non_spacer_count as u32)
                }
            })
            .collect();
    }

    // Vertical: size each child by its natural height, with Spacer as Fill.
    children
        .iter()
        .map(|c| match c {
            HIRView::Spacer => Constraint::Fill(1),
            HIRView::Divider(_) => Constraint::Length(1),
            HIRView::TextField(_) | HIRView::TextArea(_) => Constraint::Length(3),
            HIRView::Button(_) => Constraint::Length(3),
            HIRView::Progress(_) | HIRView::Slider(_) => Constraint::Length(3),
            HIRView::Heading(_) => Constraint::Length(1),
            HIRView::Row(layout) => {
                let h = layout
                    .children
                    .iter()
                    .map(estimate_height)
                    .max()
                    .unwrap_or(1);
                Constraint::Length(h)
            }
            HIRView::Column(layout) => {
                let h: u16 = layout.children.iter().map(estimate_height).sum();
                Constraint::Length(h.max(1))
            }
            HIRView::Scroll(layout) => {
                let h: u16 = layout.children.iter().map(estimate_height).sum();
                Constraint::Length(h.max(1))
            }
            HIRView::Group(children) => {
                let h: u16 = children.iter().map(estimate_height).sum();
                Constraint::Length(h.max(1))
            }
            HIRView::Each(_) => Constraint::Min(3),
            _ => Constraint::Length(1),
        })
        .collect()
}

fn estimate_height(view: &HIRView) -> u16 {
    match view {
        HIRView::Button(_) => 3,
        HIRView::TextField(_) | HIRView::TextArea(_) => 3,
        HIRView::Progress(_) | HIRView::Slider(_) => 3,
        HIRView::Divider(_) => 1,
        HIRView::Spacer => 1,
        HIRView::Row(layout) => layout
            .children
            .iter()
            .map(estimate_height)
            .max()
            .unwrap_or(1),
        HIRView::Column(layout) => layout
            .children
            .iter()
            .map(estimate_height)
            .sum::<u16>()
            .max(1),
        HIRView::Group(children) => children.iter().map(estimate_height).sum::<u16>().max(1),
        HIRView::Each(each) => estimate_height(&each.body).saturating_mul(3).max(3),
        _ => 1,
    }
}

fn value_to_string(v: &Value) -> String {
    v.to_string()
}

pub fn design_to_style(design: &ResolvedDesign) -> Style {
    let mut style = Style::default();
    if let Some(ref typo) = design.typography {
        if typo.weight.unwrap_or(400) >= 700 {
            style = style.add_modifier(Modifier::BOLD);
        }
        if typo.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if typo.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        if typo.strikethrough {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }
    }
    if let Some(ref color) = design.color {
        if let Some(ref fg) = color.foreground {
            if let Some(c) = semantic_color(fg) {
                style = style.fg(c);
            }
        }
        if let Some(ref bg) = color.background {
            if let Some(c) = semantic_color(bg) {
                style = style.bg(c);
            }
        }
    }
    style
}

fn text_alignment(design: &ResolvedDesign) -> ratatui::layout::Alignment {
    use aura_core::design::TextAlignment;
    if let Some(ref typo) = design.typography {
        if let Some(align) = typo.alignment {
            return match align {
                TextAlignment::Leading => ratatui::layout::Alignment::Left,
                TextAlignment::Center => ratatui::layout::Alignment::Center,
                TextAlignment::Trailing => ratatui::layout::Alignment::Right,
            };
        }
    }
    ratatui::layout::Alignment::Left
}

fn semantic_color(name: &str) -> Option<Color> {
    match name {
        "accent" => Some(Color::Magenta),
        "danger" | "error" => Some(Color::Red),
        "warning" => Some(Color::Yellow),
        "success" => Some(Color::Green),
        "info" => Some(Color::Cyan),
        "secondary" | "muted" => Some(Color::DarkGray),
        "primary" => Some(Color::White),
        _ => None,
    }
}

fn icon_to_emoji(name: &str) -> &'static str {
    match name {
        "trash" | "trash.fill" => "🗑",
        "plus" | "plus.circle" => "+",
        "star" | "star.fill" => "⭐",
        "heart" => "❤",
        "checkmark" => "✓",
        "xmark" => "✗",
        "gear" => "⚙",
        "house" => "🏠",
        "sun.max" => "☀",
        "cloud" => "☁",
        "lock" | "lock.circle" => "🔒",
        "camera" => "📷",
        "inbox" => "📥",
        _ => "•",
    }
}
