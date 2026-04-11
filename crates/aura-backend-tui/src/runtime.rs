//! TUI runtime: event loop, state, input handling.

use crate::eval::{Scope, run_action_expr};
use crate::render::{Focusable, RenderCtx, render_view};
use crate::value::Value;
use aura_core::hir::*;
use aura_core::types::{AuraType, PrimitiveType};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("No screens in module")]
    NoScreens,
}

/// Run an Aura HIR module directly in the terminal using ratatui.
///
/// Sets up crossterm raw mode + alternate screen, builds state from the first
/// screen's `state` declarations, then enters an event loop that renders the
/// view and handles input. Press `q` or `Esc` to quit.
pub fn run_tui(module: &HIRModule) -> Result<(), TuiError> {
    let screen = module.screens.first().ok_or(TuiError::NoScreens)?;

    // Initial state from the screen's state declarations.
    let mut state = init_state_inner(screen);

    // Terminal setup.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut focused: Option<usize> = None;
    let mut last_focus_count: usize = 0;
    let app_name = module.app.name.clone();

    let result = (|| -> Result<(), TuiError> {
        loop {
            // Draw phase — collect focusables.
            let mut collected_focus: Vec<Focusable> = Vec::new();
            terminal.draw(|frame| {
                let size = frame.area();
                let outer = Block::default()
                    .title(format!(" {} ", app_name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray));
                let inner = outer.inner(size);
                frame.render_widget(outer, size);

                // Reserve a single-line help bar at the bottom.
                let (content_area, help_area) = split_help_bar(inner);

                let mut ctx = RenderCtx::new(&state, &module.components, focused);
                render_view(frame, content_area, &screen.view, &mut ctx);
                collected_focus = ctx.focus_list;

                // Help bar.
                let help = ratatui::widgets::Paragraph::new(
                    "Tab: focus   Enter: activate   Space: toggle   q/Esc: quit",
                )
                .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(help, help_area);
            })?;

            last_focus_count = collected_focus.len();
            // Clamp focused index if the focus list shrank.
            if let Some(i) = focused {
                if i >= last_focus_count {
                    focused = if last_focus_count == 0 { None } else { Some(0) };
                }
            } else if last_focus_count > 0 {
                focused = Some(0);
            }

            // Event phase.
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Tab => {
                    if last_focus_count > 0 {
                        focused = Some(focused.map(|i| (i + 1) % last_focus_count).unwrap_or(0));
                    }
                }
                KeyCode::BackTab => {
                    if last_focus_count > 0 {
                        focused = Some(
                            focused
                                .map(|i| if i == 0 { last_focus_count - 1 } else { i - 1 })
                                .unwrap_or(0),
                        );
                    }
                }
                KeyCode::Enter => {
                    if let Some(f) = focused.and_then(|i| collected_focus.get(i).cloned()) {
                        match f {
                            Focusable::Button { action } => {
                                run_action_expr(&action, &screen.actions, &mut state);
                            }
                            Focusable::Checkbox { binding } | Focusable::Toggle { binding } => {
                                toggle_bool(&mut state, &binding);
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(f) = focused.and_then(|i| collected_focus.get(i).cloned()) {
                        match f {
                            Focusable::Checkbox { binding } | Focusable::Toggle { binding } => {
                                toggle_bool(&mut state, &binding);
                            }
                            Focusable::TextField { binding } => {
                                append_char(&mut state, &binding, ' ');
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // If a text field is focused, type into it.
                    // 'q' is reserved as quit *unless* a text field has focus.
                    if let Some(f) = focused.and_then(|i| collected_focus.get(i).cloned()) {
                        if let Focusable::TextField { binding } = f {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                break;
                            }
                            append_char(&mut state, &binding, c);
                            continue;
                        }
                    }
                    if c == 'q' {
                        break;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(f) = focused.and_then(|i| collected_focus.get(i).cloned()) {
                        if let Focusable::TextField { binding } = f {
                            pop_char(&mut state, &binding);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })();

    // Terminal teardown (always run, even on error).
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn split_help_bar(area: Rect) -> (Rect, Rect) {
    if area.height < 2 {
        return (area, Rect::new(area.x, area.y, area.width, 0));
    }
    let content = Rect::new(area.x, area.y, area.width, area.height - 1);
    let help = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    (content, help)
}

fn toggle_bool(state: &mut Scope, binding: &str) {
    let current = state.get(binding).map(|v| v.as_bool()).unwrap_or(false);
    state.insert(binding.to_string(), Value::Bool(!current));
}

fn append_char(state: &mut Scope, binding: &str, c: char) {
    let mut s = state
        .get(binding)
        .map(|v| v.to_string())
        .unwrap_or_default();
    s.push(c);
    state.insert(binding.to_string(), Value::Str(s));
}

fn pop_char(state: &mut Scope, binding: &str) {
    let mut s = state
        .get(binding)
        .map(|v| v.to_string())
        .unwrap_or_default();
    s.pop();
    state.insert(binding.to_string(), Value::Str(s));
}

fn default_value_for_type(ty: &AuraType) -> Value {
    match ty {
        AuraType::Primitive(PrimitiveType::Int) => Value::Int(0),
        AuraType::Primitive(PrimitiveType::Float) => Value::Float(0.0),
        AuraType::Primitive(PrimitiveType::Percent) => Value::Float(0.0),
        AuraType::Primitive(PrimitiveType::Bool) => Value::Bool(false),
        AuraType::Primitive(PrimitiveType::Text) => Value::Str(String::new()),
        AuraType::List(_) | AuraType::Set(_) => Value::List(Vec::new()),
        AuraType::Optional(_) => Value::Nil,
        _ => Value::Nil,
    }
}

fn init_state_inner(screen: &HIRScreen) -> Scope {
    let mut state = Scope::new();
    for st in &screen.state {
        let v = if let Some(ref init) = st.initial {
            crate::eval::eval_expr(init, &state, &Scope::new())
        } else {
            default_value_for_type(&st.state_type)
        };
        state.insert(st.name.clone(), v);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str) -> HIRModule {
        let result = aura_core::parser::parse(source);
        assert!(
            result.errors.is_empty(),
            "Parse errors: {:?}",
            result.errors
        );
        aura_core::hir::build_hir(result.program.as_ref().unwrap())
    }

    #[test]
    fn test_state_initializes_from_declarations() {
        let module = compile(
            "\
app Counter
  screen Main
    state count: int = 10
    view
      text \"hello\"",
        );
        let state = init_state_inner(&module.screens[0]);
        assert_eq!(state.get("count"), Some(&Value::Int(10)));
    }

    #[test]
    fn test_action_mutates_state() {
        let module = compile(
            "\
app Counter
  screen Main
    state count: int = 0
    view
      text \"hi\"
    action increment
      count = count + 1",
        );
        let screen = &module.screens[0];
        let mut state = init_state_inner(screen);
        let action = HIRActionExpr::Call("increment".to_string(), vec![]);
        run_action_expr(&action, &screen.actions, &mut state);
        run_action_expr(&action, &screen.actions, &mut state);
        assert_eq!(state.get("count"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_toggle_bool_flips_state() {
        let mut state = Scope::new();
        state.insert("flag".to_string(), Value::Bool(false));
        toggle_bool(&mut state, "flag");
        assert_eq!(state.get("flag"), Some(&Value::Bool(true)));
        toggle_bool(&mut state, "flag");
        assert_eq!(state.get("flag"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_text_input_appends_and_pops() {
        let mut state = Scope::new();
        state.insert("name".to_string(), Value::Str(String::new()));
        append_char(&mut state, "name", 'H');
        append_char(&mut state, "name", 'i');
        assert_eq!(state.get("name"), Some(&Value::Str("Hi".to_string())));
        pop_char(&mut state, "name");
        assert_eq!(state.get("name"), Some(&Value::Str("H".to_string())));
    }
}
