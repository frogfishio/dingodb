use crate::ast::{BinOpKind, Expr, Program, SelectMode, Stmt, UnOpKind};

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for stmt in &program.stmts {
        match stmt {
            Stmt::Let(name, expr) => {
                out.push_str("let ");
                out.push_str(name);
                out.push_str(" = ");
                out.push_str(&format_expr(expr, 0));
                out.push_str(";\n");
            }
            Stmt::Expr(expr) => {
                out.push_str(&format_expr(expr, 0));
                out.push_str(";\n");
            }
        }
    }
    out
}

fn format_expr(expr: &Expr, min_prec: u8) -> String {
    match expr {
        Expr::Null => "null".to_string(),
        Expr::Bool(true) => "true".to_string(),
        Expr::Bool(false) => "false".to_string(),
        Expr::Num(value) => value.to_string(),
        Expr::Str(value) => format_string(value),
        Expr::Bytes(bytes) => format!("Bytes({})", format_string(&encode_base16(bytes))),
        Expr::Ident(name) => name.clone(),
        Expr::Placeholder => "_".to_string(),
        Expr::Seq(items) => format_delimited("Seq[", items, "]"),
        Expr::Set(items) => format_delimited("Set{", items, "}"),
        Expr::Bag(items) => format_delimited("Bag{", items, "}"),
        Expr::Map(entries) => {
            let items = entries
                .iter()
                .map(|(key, value)| format!("{} -> {}", format_string(key), format_expr(value, 0)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Map{{{items}}}")
        }
        Expr::Prod(fields) => {
            let items = fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", format_expr(value, 0)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Prod{{{items}}}")
        }
        Expr::BagKV(entries) => {
            let items = entries
                .iter()
                .map(|(key, value)| format!("{} -> {}", format_selector_key(key), format_expr(value, 0)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("BagKV{{{items}}}")
        }
        Expr::Some_(value) => format!("Some({})", format_expr(value, 0)),
        Expr::None_ => "None".to_string(),
        Expr::Ok_(value) => format!("Ok({})", format_expr(value, 0)),
        Expr::Fail_(code, msg) => format!("Fail({}, {})", format_expr(code, 0), format_expr(msg, 0)),
        Expr::BinOp(op, lhs, rhs) => {
            let prec = precedence(expr);
            let left = format_expr(lhs, prec);
            let right_prec = if is_right_sensitive_binary(op) { prec + 1 } else { prec };
            let right = format_expr(rhs, right_prec);
            let rendered = format!("{left} {} {right}", format_binop(op));
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::UnOp(op, inner) => {
            let prec = precedence(expr);
            let operand = format_expr(inner, prec);
            let rendered = match op {
                UnOpKind::Neg => format!("-{operand}"),
                UnOpKind::Not => format!("not {operand}"),
            };
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::Pipe(lhs, rhs) => {
            let prec = precedence(expr);
            let rendered = format!("{} |> {}", format_expr(lhs, prec), format_expr(rhs, prec + 1));
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::Lambda(param, body) => {
            let prec = precedence(expr);
            let rendered = format!("{param} => {}", format_expr(body, prec));
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::Call(callee, args) => {
            let prec = precedence(expr);
            let rendered = format!(
                "{}({})",
                format_expr(callee, prec),
                args.iter()
                    .map(|arg| format_expr(arg, 0))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::Select(target, selector, mode) => {
            let prec = precedence(expr);
            let suffix = match mode {
                SelectMode::Plain => "",
                SelectMode::Optional => "?",
                SelectMode::Required => "!",
            };
            let rendered = format!(
                "{}<{}>{suffix}",
                format_expr(target, prec),
                format_string(selector)
            );
            wrap_if_needed(rendered, prec, min_prec)
        }
        Expr::Comprehension {
            yield_expr,
            binding,
            collection,
            pred,
        } => {
            let mut rendered = String::from("{");
            if let Some(yield_expr) = yield_expr {
                rendered.push_str(" yield ");
                rendered.push_str(&format_expr(yield_expr, 0));
                rendered.push_str(" | ");
            } else {
                rendered.push(' ');
            }
            rendered.push_str(binding);
            rendered.push_str(" in ");
            rendered.push_str(&format_expr(collection, 0));
            if let Some(pred) = pred {
                rendered.push_str(" | ");
                rendered.push_str(&format_expr(pred, 0));
            }
            rendered.push_str(" }");
            rendered
        }
    }
}

fn format_delimited(prefix: &str, items: &[Expr], suffix: &str) -> String {
    let body = items.iter().map(|item| format_expr(item, 0)).collect::<Vec<_>>().join(", ");
    format!("{prefix}{body}{suffix}")
}

fn format_selector_key(key: &str) -> String {
    if is_identifier(key) {
        key.to_string()
    } else {
        format_string(key)
    }
}

fn format_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn format_binop(op: &BinOpKind) -> &'static str {
    match op {
        BinOpKind::Add => "+",
        BinOpKind::Sub => "-",
        BinOpKind::Mul => "*",
        BinOpKind::Div => "/",
        BinOpKind::Concat => "++",
        BinOpKind::Eq => "=",
        BinOpKind::Neq => "!=",
        BinOpKind::Lt => "<",
        BinOpKind::Le => "<=",
        BinOpKind::Gt => ">",
        BinOpKind::Ge => ">=",
        BinOpKind::And => "and",
        BinOpKind::Or => "or",
        BinOpKind::Union => "union",
        BinOpKind::Inter => "inter",
        BinOpKind::Diff => "diff",
        BinOpKind::BUnion => "bunion",
        BinOpKind::BDiff => "bdiff",
        BinOpKind::In => "in",
    }
}

fn precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Pipe(_, _) => 1,
        Expr::BinOp(BinOpKind::Or, _, _) => 2,
        Expr::BinOp(BinOpKind::And, _, _) => 3,
        Expr::BinOp(BinOpKind::Eq | BinOpKind::Neq | BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge, _, _) => 4,
        Expr::BinOp(BinOpKind::Union | BinOpKind::Inter | BinOpKind::Diff | BinOpKind::BUnion | BinOpKind::BDiff | BinOpKind::In, _, _) => 5,
        Expr::BinOp(BinOpKind::Add | BinOpKind::Sub | BinOpKind::Concat, _, _) => 6,
        Expr::BinOp(BinOpKind::Mul | BinOpKind::Div, _, _) => 7,
        Expr::UnOp(_, _) => 8,
        Expr::Call(_, _) | Expr::Select(_, _, _) => 9,
        Expr::Lambda(_, _) => 1,
        _ => 10,
    }
}

fn is_right_sensitive_binary(op: &BinOpKind) -> bool {
    matches!(op, BinOpKind::Sub | BinOpKind::Div | BinOpKind::In)
}

fn wrap_if_needed(rendered: String, prec: u8, min_prec: u8) -> String {
    if prec < min_prec {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_alphanumeric() || ch == '_')
}

fn encode_base16(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}