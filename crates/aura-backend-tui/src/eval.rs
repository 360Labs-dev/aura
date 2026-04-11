//! Expression and statement evaluation for the TUI runtime.

use crate::value::Value;
use aura_core::ast::{BinOp, UnaryOp};
use aura_core::hir::*;
use std::collections::HashMap;

pub type Scope = HashMap<String, Value>;

/// Evaluate an HIR expression against a state scope and optional local scope.
pub fn eval_expr(expr: &HIRExpr, state: &Scope, locals: &Scope) -> Value {
    match expr {
        HIRExpr::IntLit(n) => Value::Int(*n),
        HIRExpr::FloatLit(f) => Value::Float(*f),
        HIRExpr::StringLit(s) => Value::Str(s.clone()),
        HIRExpr::PercentLit(f) => Value::Float(*f),
        HIRExpr::BoolLit(b) => Value::Bool(*b),
        HIRExpr::Nil => Value::Nil,
        HIRExpr::Var(name, _) => locals
            .get(name)
            .or_else(|| state.get(name))
            .cloned()
            .unwrap_or(Value::Nil),
        HIRExpr::MemberAccess(obj, field, _) => {
            let v = eval_expr(obj, state, locals);
            member_access(&v, field)
        }
        HIRExpr::Index(obj, idx, _) => {
            let list = eval_expr(obj, state, locals);
            let index = eval_expr(idx, state, locals).as_int();
            match list {
                Value::List(items) => items.get(index as usize).cloned().unwrap_or(Value::Nil),
                Value::Str(s) => s
                    .chars()
                    .nth(index as usize)
                    .map(|c| Value::Str(c.to_string()))
                    .unwrap_or(Value::Nil),
                _ => Value::Nil,
            }
        }
        HIRExpr::BinOp(l, op, r, _) => {
            let lv = eval_expr(l, state, locals);
            let rv = eval_expr(r, state, locals);
            eval_binop(&lv, *op, &rv)
        }
        HIRExpr::UnaryOp(op, e, _) => {
            let v = eval_expr(e, state, locals);
            match op {
                UnaryOp::Neg => match v {
                    Value::Int(n) => Value::Int(-n),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Nil,
                },
                UnaryOp::Not => Value::Bool(!v.as_bool()),
            }
        }
        HIRExpr::Conditional(cond, then_e, else_e, _) => {
            if eval_expr(cond, state, locals).as_bool() {
                eval_expr(then_e, state, locals)
            } else {
                eval_expr(else_e, state, locals)
            }
        }
        HIRExpr::NilCoalesce(a, b, _) => {
            let v = eval_expr(a, state, locals);
            if matches!(v, Value::Nil) {
                eval_expr(b, state, locals)
            } else {
                v
            }
        }
        HIRExpr::Call(callee, args, _) => {
            // Handle a few built-in method-like calls.
            if let HIRExpr::MemberAccess(receiver, method, _) = callee.as_ref() {
                let recv = eval_expr(receiver, state, locals);
                let evaluated_args: Vec<Value> =
                    args.iter().map(|a| eval_expr(a, state, locals)).collect();
                return eval_method(&recv, method, &evaluated_args);
            }
            Value::Nil
        }
        _ => Value::Nil,
    }
}

fn member_access(v: &Value, field: &str) -> Value {
    match v {
        Value::List(items) => match field {
            "count" | "length" | "size" => Value::Int(items.len() as i64),
            "isEmpty" => Value::Bool(items.is_empty()),
            "first" => items.first().cloned().unwrap_or(Value::Nil),
            "last" => items.last().cloned().unwrap_or(Value::Nil),
            _ => Value::Nil,
        },
        Value::Str(s) => match field {
            "count" | "length" | "size" => Value::Int(s.chars().count() as i64),
            "isEmpty" => Value::Bool(s.is_empty()),
            _ => Value::Nil,
        },
        _ => Value::Nil,
    }
}

fn eval_method(receiver: &Value, method: &str, _args: &[Value]) -> Value {
    match receiver {
        Value::List(items) => match method {
            "count" | "length" | "size" => Value::Int(items.len() as i64),
            "isEmpty" => Value::Bool(items.is_empty()),
            _ => Value::Nil,
        },
        _ => Value::Nil,
    }
}

fn eval_binop(l: &Value, op: BinOp, r: &Value) -> Value {
    use BinOp::*;
    match op {
        Add => match (l, r) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
            (Value::Str(a), b) => Value::Str(format!("{}{}", a, b)),
            (a, Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
            _ => Value::Nil,
        },
        Sub => Value::Float(l.as_float() - r.as_float()),
        Mul => Value::Float(l.as_float() * r.as_float()),
        Div => {
            let rv = r.as_float();
            if rv == 0.0 {
                Value::Nil
            } else {
                Value::Float(l.as_float() / rv)
            }
        }
        Mod => Value::Int(l.as_int() % r.as_int().max(1)),
        Eq => Value::Bool(values_eq(l, r)),
        NotEq => Value::Bool(!values_eq(l, r)),
        Lt => Value::Bool(l.as_float() < r.as_float()),
        Gt => Value::Bool(l.as_float() > r.as_float()),
        LtEq => Value::Bool(l.as_float() <= r.as_float()),
        GtEq => Value::Bool(l.as_float() >= r.as_float()),
        And => Value::Bool(l.as_bool() && r.as_bool()),
        Or => Value::Bool(l.as_bool() || r.as_bool()),
        Range => Value::Nil,
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => (*a as f64) == *b,
        (Value::Str(a), Value::Str(b)) => a == b,
        _ => false,
    }
}

/// Run an action by name, mutating state.
pub fn run_action(name: &str, args: &[Value], actions: &[HIRAction], state: &mut Scope) {
    let Some(action) = actions.iter().find(|a| a.name == name) else {
        return;
    };
    let mut locals: Scope = HashMap::new();
    for (param, value) in action.params.iter().zip(args.iter()) {
        locals.insert(param.name.clone(), value.clone());
    }
    run_stmts(&action.body, state, &mut locals, actions);
}

/// Execute an action expression (e.g., button press).
pub fn run_action_expr(expr: &HIRActionExpr, actions: &[HIRAction], state: &mut Scope) {
    match expr {
        HIRActionExpr::Call(name, args) => {
            let empty: Scope = HashMap::new();
            let values: Vec<Value> = args.iter().map(|a| eval_expr(a, state, &empty)).collect();
            run_action(name, &values, actions, state);
        }
        HIRActionExpr::Sequence(seq) => {
            for expr in seq {
                run_action_expr(expr, actions, state);
            }
        }
        HIRActionExpr::Navigate(_) => {
            // Navigation not implemented in the TUI runtime — single-screen only.
        }
    }
}

fn run_stmts(stmts: &[HIRStmt], state: &mut Scope, locals: &mut Scope, actions: &[HIRAction]) {
    for stmt in stmts {
        match stmt {
            HIRStmt::Assign(name, expr) => {
                let v = eval_expr(expr, state, locals);
                if state.contains_key(name) {
                    state.insert(name.clone(), v);
                } else {
                    locals.insert(name.clone(), v);
                }
            }
            HIRStmt::Let(name, _, expr) => {
                let v = eval_expr(expr, state, locals);
                locals.insert(name.clone(), v);
            }
            HIRStmt::If(cond, then_body, else_body) => {
                if eval_expr(cond, state, locals).as_bool() {
                    run_stmts(then_body, state, locals, actions);
                } else if let Some(eb) = else_body {
                    run_stmts(eb, state, locals, actions);
                }
            }
            HIRStmt::Expr(expr) => {
                // If the expression is a function call referring to a known action, run it.
                if let HIRExpr::Call(callee, args, _) = expr {
                    if let HIRExpr::Var(name, _) = callee.as_ref() {
                        if actions.iter().any(|a| &a.name == name) {
                            let values: Vec<Value> =
                                args.iter().map(|a| eval_expr(a, state, locals)).collect();
                            run_action(name, &values, actions, state);
                            continue;
                        }
                    }
                }
                // Otherwise just evaluate and discard.
                let _ = eval_expr(expr, state, locals);
            }
            HIRStmt::Return(_) => return,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_core::types::AuraType;

    #[test]
    fn test_eval_int_literal() {
        let state = Scope::new();
        let locals = Scope::new();
        assert_eq!(
            eval_expr(&HIRExpr::IntLit(42), &state, &locals),
            Value::Int(42)
        );
    }

    #[test]
    fn test_eval_var_from_state() {
        let mut state = Scope::new();
        state.insert("count".to_string(), Value::Int(5));
        let locals = Scope::new();
        let expr = HIRExpr::Var(
            "count".to_string(),
            AuraType::Primitive(aura_core::types::PrimitiveType::Int),
        );
        assert_eq!(eval_expr(&expr, &state, &locals), Value::Int(5));
    }

    #[test]
    fn test_eval_binop_add() {
        let state = Scope::new();
        let locals = Scope::new();
        let expr = HIRExpr::BinOp(
            Box::new(HIRExpr::IntLit(2)),
            BinOp::Add,
            Box::new(HIRExpr::IntLit(3)),
            AuraType::Primitive(aura_core::types::PrimitiveType::Int),
        );
        assert_eq!(eval_expr(&expr, &state, &locals), Value::Int(5));
    }

    #[test]
    fn test_eval_conditional_bool() {
        let state = Scope::new();
        let locals = Scope::new();
        let expr = HIRExpr::BinOp(
            Box::new(HIRExpr::IntLit(3)),
            BinOp::Gt,
            Box::new(HIRExpr::IntLit(1)),
            AuraType::Primitive(aura_core::types::PrimitiveType::Bool),
        );
        assert_eq!(eval_expr(&expr, &state, &locals), Value::Bool(true));
    }
}
