//! Structured Data Algebra as a pure Rust library.
//!
//! `sda-lib` parses, validates, formats, and evaluates standalone SDA programs.
//! The public API is intentionally small so host applications can bind JSON input,
//! run a program, and recover canonical JSON output without embedding CLI concerns.
//!
//! ```rust
//! let output = sda_lib::run("input<\"name\">!", serde_json::json!({"name": "Ada"}))?;
//! assert_eq!(output, serde_json::json!({"$type": "ok", "$value": "Ada"}));
//! # Ok::<(), sda_lib::SdaError>(())
//! ```

mod ast;
mod format;
mod lexer;
mod number;
mod parser;

pub mod eval;
pub mod stdlib;

pub use eval::EvalError;
pub use lexer::LexError;
pub use number::{ExactNum, ParseNumError};
pub use parser::ParseError;

use ast::Expr;
use std::collections::HashMap;
use thiserror::Error;

pub type Env = HashMap<String, Value>;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct SdaRuntime;

impl SdaRuntime {
    #[must_use]
    pub fn name() -> &'static str {
        "sda-lib"
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Num(ExactNum),
    Str(String),
    Bytes(Vec<u8>),
    Seq(Vec<Value>),
    Set(Vec<Value>),
    Bag(Vec<Value>),
    Map(Vec<(String, Value)>),
    Prod(Vec<(String, Value)>),
    BagKV(Vec<(Value, Value)>),
    Bind(Box<Value>, Box<Value>),
    Some_(Box<Value>),
    None_,
    Ok_(Box<Value>),
    Fail_(String, String),
    Lambda(String, Box<Expr>, Box<Env>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Num(a), Value::Num(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::Seq(a), Value::Seq(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => set_equal(a, b),
            (Value::Bag(a), Value::Bag(b)) => multiset_equal(a, b),
            (Value::Map(a), Value::Map(b)) => kv_extensional_equal(a, b),
            (Value::Prod(a), Value::Prod(b)) => kv_extensional_equal(a, b),
            (Value::BagKV(a), Value::BagKV(b)) => bagkv_equal(a, b),
            (Value::Bind(k1, v1), Value::Bind(k2, v2)) => k1 == k2 && v1 == v2,
            (Value::Some_(a), Value::Some_(b)) => a == b,
            (Value::None_, Value::None_) => true,
            (Value::Ok_(a), Value::Ok_(b)) => a == b,
            (Value::Fail_(c1, m1), Value::Fail_(c2, m2)) => c1 == c2 && m1 == m2,
            (Value::Lambda(_, _, _), _) => false,
            (_, Value::Lambda(_, _, _)) => false,
            _ => false,
        }
    }
}

fn set_equal(left: &[Value], right: &[Value]) -> bool {
    left.len() == right.len()
        && left.iter().all(|left_value| right.iter().any(|right_value| left_value == right_value))
}

fn multiset_equal(left: &[Value], right: &[Value]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut matched = vec![false; right.len()];
    for left_value in left {
        let mut found = false;
        for (index, right_value) in right.iter().enumerate() {
            if !matched[index] && left_value == right_value {
                matched[index] = true;
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }

    true
}

fn kv_extensional_equal(left: &[(String, Value)], right: &[(String, Value)]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter().all(|(left_key, left_value)| {
        right
            .iter()
            .find(|(right_key, _)| right_key == left_key)
            .is_some_and(|(_, right_value)| left_value == right_value)
    })
}

fn bagkv_equal(left: &[(Value, Value)], right: &[(Value, Value)]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut matched = vec![false; right.len()];
    for (left_key, left_value) in left {
        let mut found = false;
        for (index, (right_key, right_value)) in right.iter().enumerate() {
            if !matched[index] && left_key == right_key && left_value == right_value {
                matched[index] = true;
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }

    true
}

#[derive(Debug, Error)]
pub enum SdaError {
    #[error("Lex error: {0}")]
    Lex(#[from] LexError),
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("Eval error: {0}")]
    Eval(#[from] EvalError),
}

pub fn run(expr: &str, input: serde_json::Value) -> Result<serde_json::Value, SdaError> {
    run_with_input_binding(expr, "input", input)
}

pub fn run_with_input_binding(
    expr: &str,
    binding_name: &str,
    input: serde_json::Value,
) -> Result<serde_json::Value, SdaError> {
    let input_val = from_json(input);
    let program = parse_source(expr)?;
    let mut env = Env::new();
    env.insert(binding_name.to_string(), input_val);
    let result = eval::eval_program(&program, &mut env)?;
    Ok(to_json(result.unwrap_or(Value::Null)))
}

pub fn check(expr: &str) -> Result<(), SdaError> {
    parse_source(expr).map(|_| ())
}

pub fn format_source(expr: &str) -> Result<String, SdaError> {
    let program = parse_source(expr)?;
    Ok(format::format_program(&program))
}

fn parse_source(expr: &str) -> Result<ast::Program, SdaError> {
    let expr_normalized = {
        let trimmed = expr.trim_end();
        if trimmed.ends_with(';') {
            trimmed.to_string()
        } else {
            format!("{};", trimmed)
        }
    };
    let tokens = lexer::lex(&expr_normalized)?;
    let program = parser::parse(tokens)?;
    Ok(program)
}

pub fn from_json(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            Value::Num(ExactNum::parse_literal(&n.to_string()).expect("serde_json number should parse exactly"))
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => Value::Seq(arr.into_iter().map(from_json).collect()),
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::String(ty)) = obj.get("$type") {
                match ty.as_str() {
                    "bytes" => {
                        if let Some(serde_json::Value::String(base16)) = obj.get("$base16") {
                            return Value::Bytes(
                                decode_base16(base16)
                                    .expect("canonical bytes wrapper should contain valid base16"),
                            );
                        }
                    }
                    "map" => {
                        if let Some(serde_json::Value::Object(entries)) = obj.get("$entries") {
                            return Value::Map(
                                entries
                                    .iter()
                                    .map(|(k, v)| (k.clone(), from_json(v.clone())))
                                    .collect(),
                            );
                        }
                    }
                    "num" => {
                        if let Some(serde_json::Value::String(value)) = obj.get("$value") {
                            return Value::Num(
                                ExactNum::parse_canonical(value)
                                    .expect("canonical numeric wrapper should parse")
                            );
                        }
                    }
                    "set" => {
                        if let Some(serde_json::Value::Array(items)) = obj.get("$items") {
                            return Value::Set(items.iter().cloned().map(from_json).collect());
                        }
                    }
                    "bag" => {
                        if let Some(serde_json::Value::Array(items)) = obj.get("$items") {
                            return Value::Bag(items.iter().cloned().map(from_json).collect());
                        }
                    }
                    "prod" => {
                        if let Some(serde_json::Value::Object(fields)) = obj.get("$fields") {
                            let entries = fields
                                .iter()
                                .map(|(k, v)| (k.clone(), from_json(v.clone())))
                                .collect();
                            return Value::Prod(entries);
                        }
                    }
                    "bagkv" => {
                        if let Some(serde_json::Value::Array(items)) = obj.get("$items") {
                            let pairs = items
                                .iter()
                                .filter_map(|item| {
                                    if let serde_json::Value::Array(pair) = item {
                                        if pair.len() == 2 {
                                            return Some((
                                                from_json(pair[0].clone()),
                                                from_json(pair[1].clone()),
                                            ));
                                        }
                                    }
                                    None
                                })
                                .collect();
                            return Value::BagKV(pairs);
                        }
                    }
                    "bind" => {
                        if let (Some(k), Some(v)) = (obj.get("$key"), obj.get("$val")) {
                            return Value::Bind(
                                Box::new(from_json(k.clone())),
                                Box::new(from_json(v.clone())),
                            );
                        }
                    }
                    "some" => {
                        if let Some(inner) = obj.get("$value") {
                            return Value::Some_(Box::new(from_json(inner.clone())));
                        }
                    }
                    "none" => return Value::None_,
                    "ok" => {
                        if let Some(inner) = obj.get("$value") {
                            return Value::Ok_(Box::new(from_json(inner.clone())));
                        }
                    }
                    "fail" => {
                        let code = obj
                            .get("$code")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string();
                        let msg = obj
                            .get("$msg")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string();
                        return Value::Fail_(code, msg);
                    }
                    _ => {}
                }
            }
            Value::Map(obj.into_iter().map(|(k, v)| (k, from_json(v))).collect())
        }
    }
}

fn canonical_json_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) | serde_json::Value::String(_) => {
            serde_json::to_string(value).expect("canonical JSON rendering should succeed")
        }
        serde_json::Value::Array(items) => {
            let rendered_items = items
                .iter()
                .map(canonical_json_text)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{rendered_items}]")
        }
        serde_json::Value::Object(entries) => {
            let mut sorted_entries: Vec<_> = entries.iter().collect();
            sorted_entries.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
            let rendered_entries = sorted_entries
                .into_iter()
                .map(|(key, value)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key)
                            .expect("canonical JSON key rendering should succeed"),
                        canonical_json_text(value)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{rendered_entries}}}")
        }
    }
}

fn canonicalize_set_items(items: Vec<Value>) -> Vec<serde_json::Value> {
    let mut unique = Vec::new();
    for item in items {
        if !unique.iter().any(|existing| existing == &item) {
            unique.push(item);
        }
    }

    let mut rendered = unique.into_iter().map(to_json).collect::<Vec<_>>();
    rendered.sort_by_key(canonical_json_text);
    rendered
}

fn canonicalize_bag_items(items: Vec<Value>) -> Vec<serde_json::Value> {
    let mut rendered = items.into_iter().map(to_json).collect::<Vec<_>>();
    rendered.sort_by_key(canonical_json_text);
    rendered
}

fn canonicalize_map_entries(entries: Vec<(String, Value)>) -> Vec<(String, serde_json::Value)> {
    let mut mapped_entries: Vec<(String, serde_json::Value)> =
        entries.into_iter().map(|(key, value)| (key, to_json(value))).collect();
    mapped_entries.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
    mapped_entries
}

fn canonicalize_bagkv_items(pairs: Vec<(Value, Value)>) -> Vec<serde_json::Value> {
    let mut rendered = pairs
        .into_iter()
        .map(|(key, value)| serde_json::json!([to_json(key), to_json(value)]))
        .collect::<Vec<_>>();
    rendered.sort_by_key(canonical_json_text);
    rendered
}

pub fn to_json(v: Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Num(n) => n.to_json_value(),
        Value::Str(s) => serde_json::Value::String(s),
        Value::Bytes(bytes) => serde_json::json!({
            "$type": "bytes",
            "$base16": encode_base16(&bytes)
        }),
        Value::Seq(items) => serde_json::Value::Array(items.into_iter().map(to_json).collect()),
        Value::Set(items) => serde_json::json!({
            "$type": "set",
            "$items": canonicalize_set_items(items)
        }),
        Value::Bag(items) => serde_json::json!({
            "$type": "bag",
            "$items": canonicalize_bag_items(items)
        }),
        Value::Map(entries) => {
            let mapped_entries = canonicalize_map_entries(entries);
            if should_wrap_map(&mapped_entries) {
                let entries_obj: serde_json::Map<String, serde_json::Value> =
                    mapped_entries.into_iter().collect();
                serde_json::json!({
                    "$type": "map",
                    "$entries": entries_obj
                })
            } else {
                let mut map = serde_json::Map::new();
                for (k, v) in mapped_entries {
                    map.insert(k, v);
                }
                serde_json::Value::Object(map)
            }
        }
        Value::Prod(fields) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "$type".to_string(),
                serde_json::Value::String("prod".to_string()),
            );
            let fields_map: serde_json::Map<String, serde_json::Value> =
                fields.into_iter().map(|(k, v)| (k, to_json(v))).collect();
            map.insert("$fields".to_string(), serde_json::Value::Object(fields_map));
            serde_json::Value::Object(map)
        }
        Value::BagKV(pairs) => serde_json::json!({
            "$type": "bagkv",
            "$items": canonicalize_bagkv_items(pairs)
        }),
        Value::Bind(k, v) => serde_json::json!({
            "$type": "bind",
            "$key": to_json(*k),
            "$val": to_json(*v)
        }),
        Value::Some_(inner) => serde_json::json!({
            "$type": "some",
            "$value": to_json(*inner)
        }),
        Value::None_ => serde_json::json!({"$type": "none"}),
        Value::Ok_(inner) => serde_json::json!({
            "$type": "ok",
            "$value": to_json(*inner)
        }),
        Value::Fail_(code, msg) => serde_json::json!({
            "$type": "fail",
            "$code": code,
            "$msg": msg
        }),
        Value::Lambda(_, _, _) => serde_json::json!({"$type": "fn"}),
    }
}

fn should_wrap_map(entries: &[(String, serde_json::Value)]) -> bool {
    let Some((_, type_value)) = entries.iter().find(|(key, _)| key == "$type") else {
        return false;
    };

    match type_value {
        serde_json::Value::String(tag) => reserved_json_tag(tag),
        _ => false,
    }
}

fn reserved_json_tag(tag: &str) -> bool {
    matches!(
        tag,
        "map"
            | "num"
            | "bytes"
            | "set"
            | "bag"
            | "prod"
            | "bagkv"
            | "bind"
            | "some"
            | "none"
            | "ok"
            | "fail"
            | "fn"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(expr: &str) -> serde_json::Value {
        run(expr, serde_json::Value::Null).expect("run failed")
    }

    fn ri(expr: &str, input: serde_json::Value) -> serde_json::Value {
        run(expr, input).expect("run failed")
    }

    fn rib(expr: &str, binding_name: &str, input: serde_json::Value) -> serde_json::Value {
        run_with_input_binding(expr, binding_name, input).expect("run failed")
    }

    fn assert_same(expr_a: &str, expr_b: &str) {
        assert_eq!(r(expr_a), r(expr_b));
    }

    fn assert_same_input(expr_a: &str, expr_b: &str, input: serde_json::Value) {
        assert_eq!(ri(expr_a, input.clone()), ri(expr_b, input));
    }

    fn assert_json_round_trip(value: Value, expected_json: serde_json::Value) {
        let encoded = to_json(value.clone());
        assert_eq!(encoded, expected_json);
        assert_eq!(from_json(encoded), value);
    }

    fn num(src: &str) -> Value {
        Value::Num(ExactNum::parse_literal(src).expect("valid exact number literal"))
    }

    fn bytes(src: &str) -> Value {
        Value::Bytes(decode_base16(src).expect("valid base16 bytes literal"))
    }

    #[test]
    fn test_null_literal() {
        assert_eq!(r("null;"), serde_json::Value::Null);
    }

    #[test]
    fn test_bool_literals() {
        assert_eq!(r("true;"), serde_json::Value::Bool(true));
        assert_eq!(r("false;"), serde_json::Value::Bool(false));
    }

    #[test]
    fn test_num_arithmetic() {
        assert_eq!(r("1 + 2;"), serde_json::json!(3));
        assert_eq!(r("10 - 3;"), serde_json::json!(7));
        assert_eq!(r("4 * 5;"), serde_json::json!(20));
        assert_eq!(r("10 / 2;"), serde_json::json!(5));
        assert_eq!(r("1 / 0;"), serde_json::json!({"$type": "fail", "$code": "t_sda_div_by_zero", "$msg": "division by zero"}));
    }

    #[test]
    fn test_non_terminating_rational_uses_wrapper() {
        assert_eq!(
            r("1 / 3;"),
            serde_json::json!({
                "$type": "num",
                "$value": "1/3"
            })
        );
    }

    #[test]
    fn test_numeric_wrapper_round_trips_exactly() {
        let result = ri(
            "input = 1 / 3;",
            serde_json::json!({
                "$type": "num",
                "$value": "1/3"
            }),
        );
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_public_run_no_longer_binds_placeholder() {
        let result = run("_;", serde_json::json!({"name": "steve"})).expect("run failed");
        assert_eq!(
            result,
            serde_json::json!({
                "$type": "fail",
                "$code": "t_sda_unbound_placeholder",
                "$msg": "unbound placeholder"
            })
        );
    }

    #[test]
    fn test_unbound_name_is_stable() {
        assert_eq!(
            r("missing;"),
            serde_json::json!({"$type": "fail", "$code": "t_sda_unbound_name", "$msg": "unbound name"})
        );
    }

    #[test]
    fn test_not_callable_is_stable() {
        assert_eq!(
            r("1(2);"),
            serde_json::json!({"$type": "fail", "$code": "t_sda_not_callable", "$msg": "not callable"})
        );
    }

    #[test]
    fn test_arity_mismatch_is_stable() {
        assert_eq!(
            r("(x => x)(1, 2);"),
            serde_json::json!({"$type": "fail", "$code": "t_sda_arity_mismatch", "$msg": "arity mismatch"})
        );
        assert_eq!(
            r("normalizeUnique();"),
            serde_json::json!({"$type": "fail", "$code": "t_sda_arity_mismatch", "$msg": "arity mismatch"})
        );
    }

    #[test]
    fn test_custom_input_binding_name() {
        let result = rib(r#"root<"name">!;"#, "root", serde_json::json!({"name": "Ada"}));
        assert_eq!(result, serde_json::json!({"$type": "ok", "$value": "Ada"}));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(r(r#""hello" ++ " world";"#), serde_json::json!("hello world"));
    }

    #[test]
    fn test_seq_literal() {
        let result = r("seq[1, 2, 3];");
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_set_literal() {
        let result = r("set{1, 2, 2, 3};");
        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj["$type"], serde_json::json!("set"));
            if let serde_json::Value::Array(items) = &obj["$items"] {
                assert_eq!(items.len(), 3);
            }
        }
    }

    #[test]
    fn test_map_literal() {
        let result = r(r#"map{"a" -> 1, "b" -> 2};"#);
        assert_eq!(result, serde_json::json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_bytes_literal_and_equality() {
        assert_eq!(
            r(r#"Bytes("00ff");"#),
            serde_json::json!({"$type": "bytes", "$base16": "00ff"})
        );
        assert_eq!(r(r#"Bytes("00ff") = Bytes("00FF");"#), serde_json::json!(true));
    }

    #[test]
    fn test_bytes_literal_rejects_invalid_hex() {
        let err = run(r#"Bytes("0fg");"#, serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::InvalidBytesLiteral { .. })));
    }

    #[test]
    fn test_reserved_placeholder_in_let_is_stable_parse_error() {
        let err = run("let _ = 1;", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::ReservedPlaceholder)));
    }

    #[test]
    fn test_reserved_placeholder_as_lambda_param_is_stable_parse_error() {
        let err = run("_ => 1;", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::ReservedPlaceholder)));
    }

    #[test]
    fn test_invalid_map_key_is_stable_parse_error() {
        let err = run(r#"Map{a -> 1};"#, serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::InvalidMapKey)));
    }

    #[test]
    fn test_invalid_bagkv_key_is_stable_parse_error() {
        let err = run(r#"BagKV{1 -> 1};"#, serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::InvalidBagkvKey)));
    }

    #[test]
    fn test_plain_map_json_bridge_stays_plain_object() {
        assert_json_round_trip(
            Value::Map(vec![
                ("a".to_string(), num("1")),
                ("b".to_string(), Value::Str("x".to_string())),
            ]),
            serde_json::json!({"a": 1, "b": "x"}),
        );
    }

    #[test]
    fn test_reserved_tag_map_uses_explicit_map_wrapper() {
        assert_json_round_trip(
            Value::Map(vec![
                ("$type".to_string(), Value::Str("set".to_string())),
                (
                    "$items".to_string(),
                    Value::Seq(vec![num("1"), num("2")]),
                ),
            ]),
            serde_json::json!({
                "$type": "map",
                "$entries": {
                    "$type": "set",
                    "$items": [1, 2]
                }
            }),
        );
    }

    #[test]
    fn test_unknown_tag_object_remains_plain_map() {
        let value = from_json(serde_json::json!({
            "$type": "custom",
            "x": 1
        }));
        assert_eq!(
            value,
            Value::Map(vec![
                ("$type".to_string(), Value::Str("custom".to_string())),
                ("x".to_string(), num("1")),
            ])
        );
    }

    #[test]
    fn test_reserved_bytes_tag_map_uses_explicit_map_wrapper() {
        assert_json_round_trip(
            Value::Map(vec![
                ("$type".to_string(), Value::Str("bytes".to_string())),
                ("$base16".to_string(), Value::Str("not-a-wrapper".to_string())),
            ]),
            serde_json::json!({
                "$type": "map",
                "$entries": {
                    "$type": "bytes",
                    "$base16": "not-a-wrapper"
                }
            }),
        );
    }

    #[test]
    fn test_wrapper_json_bridge_round_trips() {
        assert_json_round_trip(
            bytes("00ff"),
            serde_json::json!({"$type": "bytes", "$base16": "00ff"}),
        );
        assert_json_round_trip(
            Value::Set(vec![Value::Str("a".to_string()), Value::Str("b".to_string())]),
            serde_json::json!({"$type": "set", "$items": ["a", "b"]}),
        );
        assert_json_round_trip(
            Value::Bag(vec![num("1"), num("1")]),
            serde_json::json!({"$type": "bag", "$items": [1, 1]}),
        );
        assert_json_round_trip(
            Value::Prod(vec![("name".to_string(), Value::Str("Ada".to_string()))]),
            serde_json::json!({"$type": "prod", "$fields": {"name": "Ada"}}),
        );
        assert_json_round_trip(
            Value::BagKV(vec![(Value::Str("k".to_string()), num("2"))]),
            serde_json::json!({"$type": "bagkv", "$items": [["k", 2]]}),
        );
        assert_json_round_trip(
            Value::Bind(
                Box::new(Value::Str("k".to_string())),
                Box::new(num("2")),
            ),
            serde_json::json!({"$type": "bind", "$key": "k", "$val": 2}),
        );
        assert_json_round_trip(
            Value::Some_(Box::new(Value::Str("x".to_string()))),
            serde_json::json!({"$type": "some", "$value": "x"}),
        );
        assert_json_round_trip(Value::None_, serde_json::json!({"$type": "none"}));
        assert_json_round_trip(
            Value::Ok_(Box::new(Value::Bool(true))),
            serde_json::json!({"$type": "ok", "$value": true}),
        );
        assert_json_round_trip(
            Value::Fail_("code".to_string(), "msg".to_string()),
            serde_json::json!({"$type": "fail", "$code": "code", "$msg": "msg"}),
        );
    }

    #[test]
    fn test_map_equality_is_extensional() {
        assert_eq!(
            r(r#"Map{"a" -> 1, "b" -> 2} = Map{"b" -> 2, "a" -> 1};"#),
            serde_json::Value::Bool(true)
        );
    }

    #[test]
    fn test_bag_equality_is_extensional_with_multiplicity() {
        assert_eq!(
            r(r#"Bag{1, 2, 1} = Bag{1, 1, 2};"#),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            r(r#"Bag{1, 2, 1} = Bag{1, 2, 2};"#),
            serde_json::Value::Bool(false)
        );
    }

    #[test]
    fn test_set_union_is_canonical_and_commutative() {
        let left_first = r("Set{3, 1} union Set{2, 1};");
        let right_first = r("Set{2, 1} union Set{3, 1};");
        let expected = serde_json::json!({"$type": "set", "$items": [1, 2, 3]});
        assert_eq!(left_first, expected);
        assert_eq!(right_first, expected);
    }

    #[test]
    fn test_set_intersection_is_canonical_and_idempotent() {
        let intersection = r("Set{3, 1, 2} inter Set{2, 3, 4};");
        let idempotent = r("Set{3, 1, 2} inter Set{3, 1, 2};");
        assert_eq!(intersection, serde_json::json!({"$type": "set", "$items": [2, 3]}));
        assert_eq!(idempotent, serde_json::json!({"$type": "set", "$items": [1, 2, 3]}));
    }

    #[test]
    fn test_set_difference_is_canonical_and_self_difference_is_empty() {
        assert_eq!(
            r("Set{3, 1, 2} diff Set{2};"),
            serde_json::json!({"$type": "set", "$items": [1, 3]})
        );
        assert_eq!(
            r("Set{3, 1, 2} diff Set{3, 1, 2};"),
            serde_json::json!({"$type": "set", "$items": []})
        );
    }

    #[test]
    fn test_bag_union_is_canonical_and_commutative() {
        let left_first = r("Bag{3, 1, 2} bunion Bag{2, 1};");
        let right_first = r("Bag{2, 1} bunion Bag{3, 1, 2};");
        let expected = serde_json::json!({"$type": "bag", "$items": [1, 1, 2, 2, 3]});
        assert_eq!(left_first, expected);
        assert_eq!(right_first, expected);
    }

    #[test]
    fn test_bag_difference_is_canonical_and_floors_at_zero() {
        assert_eq!(
            r("Bag{3, 1, 2, 2, 1} bdiff Bag{2, 1, 4};"),
            serde_json::json!({"$type": "bag", "$items": [1, 2, 3]})
        );
        assert_eq!(
            r("Bag{1, 1} bdiff Bag{1, 1, 1};"),
            serde_json::json!({"$type": "bag", "$items": []})
        );
    }

    #[test]
    fn test_set_algebra_is_associative_where_expected() {
        assert_eq!(
            r("(Set{3, 1} union Set{2}) union Set{4, 1};"),
            r("Set{3, 1} union (Set{2} union Set{4, 1});")
        );
        assert_eq!(
            r("(Set{3, 1, 2} inter Set{2, 3, 4}) inter Set{3, 5};"),
            r("Set{3, 1, 2} inter (Set{2, 3, 4} inter Set{3, 5});")
        );
    }

    #[test]
    fn test_bag_union_is_associative() {
        assert_eq!(
            r("(Bag{3, 1} bunion Bag{2}) bunion Bag{2, 1};"),
            r("Bag{3, 1} bunion (Bag{2} bunion Bag{2, 1});")
        );
    }

    #[test]
    fn test_prod_equality_is_extensional() {
        assert_eq!(
            r(r#"Prod{a: 1, b: 2} = Prod{b: 2, a: 1};"#),
            serde_json::Value::Bool(true)
        );
    }

    #[test]
    fn test_let_binding() {
        assert_eq!(r("let x = 42; x;"), serde_json::json!(42));
    }

    #[test]
    fn test_lambda_and_call() {
        assert_eq!(r("let f = x => x + 1; f(5);"), serde_json::json!(6));
    }

    #[test]
    fn test_pipe() {
        assert_eq!(r("5 |> _ + 1;"), serde_json::json!(6));
    }

    #[test]
    fn test_comprehension() {
        let result = r("{ x | x in seq[1, 2, 3] };");
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(r("1 < 2;"), serde_json::Value::Bool(true));
        assert_eq!(r("2 > 3;"), serde_json::Value::Bool(false));
        assert_eq!(r("1 = 1;"), serde_json::Value::Bool(true));
        assert_eq!(r("1 != 2;"), serde_json::Value::Bool(true));
    }

    #[test]
    fn test_some_none() {
        let result = r("some(42);");
        assert_eq!(result, serde_json::json!({"$type": "some", "$value": 42}));
        let result2 = r("none;");
        assert_eq!(result2, serde_json::json!({"$type": "none"}));
    }

    #[test]
    fn test_type_of() {
        assert_eq!(r(r#"typeOf(null);"#), serde_json::json!("null"));
        assert_eq!(r(r#"typeOf(42);"#), serde_json::json!("num"));
        assert_eq!(r(r#"typeOf("hello");"#), serde_json::json!("str"));
        assert_eq!(r(r#"typeOf(Bytes("00ff"));"#), serde_json::json!("bytes"));
    }

    #[test]
    fn test_unicode_ascii_parity_membership_and_logic() {
        assert_same("2 in Set{1, 2, 3};", "2 ∈ Set{1, 2, 3};");
        assert_same("true and false;", "true ∧ false;");
        assert_same("true or false;", "true ∨ false;");
        assert_same("not false;", "¬false;");
    }

    #[test]
    fn test_unicode_ascii_parity_comparisons() {
        assert_same("1 != 2;", "1 ≠ 2;");
        assert_same("1 <= 2;", "1 ≤ 2;");
        assert_same("2 >= 1;", "2 ≥ 1;");
    }

    #[test]
    fn test_unicode_ascii_parity_lambda_and_placeholder() {
        assert_same("let f = x => x + 1; f(5);", "let f = x ↦ x + 1; f(5);");
        assert_same("5 |> _ + 1;", "5 |> • + 1;");
    }

    #[test]
    fn test_unicode_ascii_parity_selector_and_comprehension_bar() {
        assert_same_input(
            r#"input<"name">!;"#,
            r#"input⟨"name"⟩!;"#,
            serde_json::json!({"name": "Ada"}),
        );
        assert_same(
            r#"{ x | x in Seq[1, 2, 3] | x >= 2 };"#,
            r#"{ x ∣ x ∈ Seq[1, 2, 3] ∣ x ≥ 2 };"#,
        );
    }

    #[test]
    fn test_unicode_ascii_parity_binding_and_bag_operators() {
        assert_same(r#"Map{"a" -> 1};"#, r#"Map{"a" → 1};"#);
        assert_same(r#"BagKV{"a" -> 1};"#, r#"BagKV{"a" → 1};"#);
        assert_same("Bag{1, 2} bunion Bag{2, 3};", "Bag{1, 2} ⊎ Bag{2, 3};");
        assert_same("Bag{1, 2, 2} bdiff Bag{2};", "Bag{1, 2, 2} ⊖ Bag{2};");
    }

    #[test]
    fn test_line_comments_are_ignored() {
        assert_eq!(r("1 + ;; comment\n 2;"), serde_json::json!(3));
        assert_eq!(r("Seq[1, ;; keep going\n 2, 3];"), serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_whitespace_is_insensitive() {
        assert_eq!(
            r(" \n\t let   x = 1 ; \n\t x   +   2 ; \n"),
            serde_json::json!(3)
        );
        assert_eq!(
            ri(" \n input < \"name\" > ! ; \n", serde_json::json!({"name": "Ada"})),
            serde_json::json!({"$type": "ok", "$value": "Ada"})
        );
    }

    #[test]
    fn test_required_string_escapes() {
        assert_eq!(r(r#""line\nindent\tquote\"slash\\";"#), serde_json::json!("line\nindent\tquote\"slash\\"));
        assert_eq!(r(r#"";; not a comment";"#), serde_json::json!(";; not a comment"));
    }

    #[test]
    fn test_keys_returns_set() {
        let result = r(r#"keys(Map{"b" -> 2, "a" -> 1});"#);
        assert_eq!(
            result,
            serde_json::json!({
                "$type": "set",
                "$items": ["a", "b"]
            })
        );
    }

    #[test]
    fn test_values_map_uses_canonical_key_order() {
        let result = r(r#"values(Map{"b" -> 2, "a" -> 1, "c" -> 3});"#);
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_values_map_from_json_input_uses_canonical_key_order() {
        let result = ri(r#"values(input);"#, serde_json::json!({"z": 3, "a": 1, "m": 2}));
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_values_prod_is_not_a_standalone_helper_contract() {
        assert_eq!(
            r(r#"values(Prod{b: 2, a: 1});"#),
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_select() {
        let result = ri(r#"input<"name">;"#, serde_json::json!({"name": "Alice"}));
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_total_selector_on_prod_is_allowed() {
        let result = r(r#"Prod{name: "Alice"}<name>;"#);
        assert_eq!(result, serde_json::json!("Alice"));
    }

    #[test]
    fn test_total_selector_on_prod_missing_field_returns_unknown_field() {
        let result = r(r#"Prod{name: "Alice"}<age>;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_unknown_field", "$msg": "unknown field"})
        );
    }

    #[test]
    fn test_total_selector_on_map_is_wrong_shape() {
        let result = ri(r#"input<"name">;"#, serde_json::json!({"name": "Alice"}));
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_total_selector_on_bagkv_is_wrong_shape() {
        let result = r(r#"BagKV{"k" -> 1}<"k">;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_optional_selector_on_non_keyed_bag_is_wrong_shape() {
        let result = r(r#"Bag{1, 2}<"k">?;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_required_selector_on_non_keyed_bag_is_wrong_shape() {
        let result = r(r#"Bag{1, 2}<"k">!;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_optional_selector_on_prod_is_wrong_shape() {
        let result = r(r#"Prod{name: "Alice"}<name>?;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_required_selector_on_prod_is_wrong_shape() {
        let result = r(r#"Prod{name: "Alice"}<name>!;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_keys_prod_is_not_a_standalone_helper_contract() {
        assert_eq!(
            r(r#"keys(Prod{b: 2, a: 1});"#),
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_count_seq_is_not_a_standalone_helper_contract() {
        assert_eq!(
            r(r#"count(1, Seq[1, 2, 1]);"#),
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_or_else_opt_preserves_some_wrapper() {
        assert_eq!(
            r(r#"orElseOpt(Some(1), Some(2));"#),
            serde_json::json!({"$type": "some", "$value": 1})
        );
        assert_eq!(
            r(r#"orElseOpt(None, Some(2));"#),
            serde_json::json!({"$type": "some", "$value": 2})
        );
    }

    #[test]
    fn test_or_else_res_preserves_ok_wrapper() {
        assert_eq!(
            r(r#"orElseRes(Ok(1), Ok(2));"#),
            serde_json::json!({"$type": "ok", "$value": 1})
        );
        assert_eq!(
            r(r#"orElseRes(Fail("x", "y"), Ok(2));"#),
            serde_json::json!({"$type": "ok", "$value": 2})
        );
    }

    #[test]
    fn test_bagkv_equality_is_extensional_with_multiplicity() {
        assert_eq!(
            r(r#"BagKV{"a" -> 1, "b" -> 2, "a" -> 1} = BagKV{"b" -> 2, "a" -> 1, "a" -> 1};"#),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            r(r#"BagKV{"a" -> 1, "b" -> 2, "a" -> 1} = BagKV{"b" -> 2, "a" -> 1};"#),
            serde_json::Value::Bool(false)
        );
    }

    #[test]
    fn test_bind_option_result_equality_is_pointwise() {
        assert_eq!(r(r#"Bind("a", 1) = Bind("a", 1);"#), serde_json::Value::Bool(true));
        assert_eq!(r(r#"Some(Null) = None;"#), serde_json::Value::Bool(false));
        assert_eq!(r(r#"Ok(1) = Ok(1);"#), serde_json::Value::Bool(true));
        assert_eq!(r(r#"Ok(1) = Fail("x", "y");"#), serde_json::Value::Bool(false));
    }

    #[test]
    fn test_function_values_are_not_comparable() {
        assert_eq!(
            r(r#"let f = x => x; f = f;"#),
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );

        assert_eq!(
            r(r#"Set{x => x};"#),
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_bagkv_optional_missing_returns_none() {
        let result = r(r#"BagKV{"k" -> 1}<"missing">?;"#);
        assert_eq!(result, serde_json::json!({"$type": "none"}));
    }

    #[test]
    fn test_bagkv_required_missing_returns_fail() {
        let result = r(r#"BagKV{"k" -> 1}<"missing">!;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_missing_key", "$msg": "missing key"})
        );
    }

    #[test]
    fn test_placeholder_scoping_pipe() {
        assert_eq!(r("5 |> _ + 1;"), serde_json::json!(6));
    }

    #[test]
    fn test_required_selector_ok() {
        let result = ri(r#"input<"name">!;"#, serde_json::json!({"name": "steve"}));
        assert_eq!(result, serde_json::json!({"$type": "ok", "$value": "steve"}));
    }

    #[test]
    fn test_required_selector_missing() {
        let result = ri(r#"input<"missing">!;"#, serde_json::json!({"name": "steve"}));
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_missing_key", "$msg": "missing key"})
        );
    }

    #[test]
    fn test_optional_selector_present() {
        let result = ri(r#"input<"name">?;"#, serde_json::json!({"name": "steve"}));
        assert_eq!(result, serde_json::json!({"$type": "some", "$value": "steve"}));
    }

    #[test]
    fn test_optional_selector_missing() {
        let result = ri(r#"input<"missing">?;"#, serde_json::json!({"name": "steve"}));
        assert_eq!(result, serde_json::json!({"$type": "none"}));
    }

    #[test]
    fn test_null_vs_absence_some_null() {
        let result = ri(r#"input<"x">?;"#, serde_json::json!({"x": null}));
        assert_eq!(result, serde_json::json!({"$type": "some", "$value": null}));
    }

    #[test]
    fn test_null_vs_absence_none() {
        let result = ri(r#"input<"x">?;"#, serde_json::json!({}));
        assert_eq!(result, serde_json::json!({"$type": "none"}));
    }

    #[test]
    fn test_bagkv_duplicate_optional() {
        let result = r(r#"BagKV{"k" -> 1, "k" -> 2}<"k">?;"#);
        assert_eq!(result, serde_json::json!({"$type": "none"}));
    }

    #[test]
    fn test_bagkv_duplicate_required() {
        let result = r(r#"BagKV{"k" -> 1, "k" -> 2}<"k">!;"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_duplicate_key", "$msg": "duplicate key"})
        );
    }

    #[test]
    fn test_normalize_unique_ok() {
        let result = r(r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#);
        assert_eq!(result, serde_json::json!({"$type": "ok", "$value": {"a": 1, "b": 2}}));
    }

    #[test]
    fn test_normalize_unique_fail() {
        let result = r(r#"normalizeUnique(BagKV{"k" -> 1, "k" -> 2});"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_duplicate_key", "$msg": "duplicate key"})
        );
    }

    #[test]
    fn test_normalize_unique_wrong_shape_returns_fail() {
        let result = r(r#"normalizeUnique(Seq[1, 2]);"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_normalize_first() {
        let result = r(r#"normalizeFirst(BagKV{"k" -> 1, "k" -> 2});"#);
        assert_eq!(result, serde_json::json!({"k": 1}));
    }

    #[test]
    fn test_normalize_first_wrong_shape_returns_fail() {
        let result = r(r#"normalizeFirst(Seq[1, 2]);"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_normalize_last() {
        let result = r(r#"normalizeLast(BagKV{"k" -> 1, "k" -> 2});"#);
        assert_eq!(result, serde_json::json!({"k": 2}));
    }

    #[test]
    fn test_normalize_last_wrong_shape_returns_fail() {
        let result = r(r#"normalizeLast(Seq[1, 2]);"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_normalize_unique_non_string_bagkv_key_returns_fail() {
        let result = ri(
            r#"normalizeUnique(input);"#,
            serde_json::json!({
                "$type": "bagkv",
                "$items": [[1, 2]]
            }),
        );
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_carrier_preservation_seq() {
        let result = ri(r#"{ x | x in input | x > 2 };"#, serde_json::json!([1, 2, 3, 4]));
        assert_eq!(result, serde_json::json!([3, 4]));
    }

    #[test]
    fn test_carrier_preservation_set() {
        let result = r(r#"{ x | x in Set{1, 2, 3} | x > 1 };"#);
        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj["$type"], serde_json::json!("set"));
            if let serde_json::Value::Array(items) = &obj["$items"] {
                assert_eq!(items.len(), 2);
            } else {
                panic!("expected $items array");
            }
        } else {
            panic!("expected set object, got {:?}", result);
        }
    }

    #[test]
    fn test_carrier_preservation_bag() {
        let result = r(r#"{ x | x in Bag{1, 2, 2, 3} | x > 1 };"#);
        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj["$type"], serde_json::json!("bag"));
            if let serde_json::Value::Array(items) = &obj["$items"] {
                assert_eq!(items.len(), 3);
            } else {
                panic!("expected $items array");
            }
        } else {
            panic!("expected bag object, got {:?}", result);
        }
    }

    #[test]
    fn test_comprehension_with_yield() {
        let result = r("{ yield x * 2 | x in Seq[1, 2, 3] };");
        assert_eq!(result, serde_json::json!([2, 4, 6]));
    }

    #[test]
    fn test_comprehension_with_general_shorthand_projection() {
        let result = r("{ x + 1 | x in Seq[1, 2, 3] };");
        assert_eq!(result, serde_json::json!([2, 3, 4]));
    }

    #[test]
    fn test_bagkv_comprehension_filter_returns_bag_of_bindings() {
        let result = r(r#"{ b | b in BagKV{"a" -> 1, "b" -> 2} | b<key> = "b" };"#);
        assert_eq!(
            result,
            serde_json::json!({
                "$type": "bag",
                "$items": [
                    {"$type": "bind", "$key": "b", "$val": 2}
                ]
            })
        );
    }

    #[test]
    fn test_bagkv_comprehension_projection_returns_bag() {
        let result = r(r#"{ b<val> | b in BagKV{"a" -> 1, "b" -> 2} };"#);
        assert_eq!(
            result,
            serde_json::json!({
                "$type": "bag",
                "$items": [1, 2]
            })
        );
    }

    #[test]
    fn test_title_case_keywords() {
        assert_eq!(r("Seq[1, 2, 3];"), serde_json::json!([1, 2, 3]));
        assert_eq!(r(r#"Map{"a" -> 1};"#), serde_json::json!({"a": 1}));
    }

    #[test]
    fn test_map_rejects_identifier_keys() {
        let err = run("Map{a -> 1};", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(_)));
    }

    #[test]
    fn test_bagkv_accepts_identifier_keys_as_selector_shorthand() {
        let result = r(r#"BagKV{content_type -> 1};"#);
        assert_eq!(
            result,
            serde_json::json!({
                "$type": "bagkv",
                "$items": [
                    ["content_type", 1]
                ]
            })
        );
    }

    #[test]
    fn test_bagkv_rejects_non_selector_keys() {
        let err = run("BagKV{1 -> 2};", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(_)));
    }

    #[test]
    fn test_static_selector_literal_is_rejected() {
        let err = run("{a b};", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::SelectorNotStatic)));
    }

    #[test]
    fn test_static_selector_duplicate_label_is_rejected() {
        let err = run("{a a};", serde_json::Value::Null).unwrap_err();
        assert!(matches!(err, SdaError::Parse(ParseError::DuplicateLabelInSelector)));
    }

    #[test]
    fn test_invalid_comprehension_generator_binding_reports_generator_shape() {
        let err = run("{ 1 in Seq[1] };", serde_json::Value::Null).unwrap_err();
        match err {
            SdaError::Parse(ParseError::Expected(expected, _, _)) => {
                assert_eq!(expected, "generator expression `name in collection`");
            }
            other => panic!("expected generator-shape parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_format_source_normalizes_spacing_and_keywords() {
        let formatted = format_source(" let x=1+2; values( input ) ").unwrap();
        assert_eq!(formatted, "let x = 1 + 2;\nvalues(input);\n");
    }

    #[test]
    fn test_format_source_canonicalizes_selectors_and_bagkv_keys() {
        let formatted = format_source(r#"BagKV{"two words" -> 1, key -> input<name>!};"#).unwrap();
        assert_eq!(formatted, "BagKV{\"two words\" -> 1, key -> input<\"name\">!};\n");
    }

    #[test]
    fn test_format_source_preserves_precedence_for_lambda_and_pipe() {
        let formatted = format_source("1 |> (x => x + 1);").unwrap();
        assert_eq!(formatted, "1 |> (x => x + 1);\n");
    }

    #[test]
    fn test_format_source_preserves_precedence_for_lambda_call_and_unary() {
        let formatted = format_source("(-(1 + 2)) + ((x => x + 1)(3));").unwrap();
        assert_eq!(formatted, "-(1 + 2) + (x => x + 1)(3);\n");
    }

    #[test]
    fn test_format_source_preserves_right_associative_sensitive_grouping() {
        let formatted = format_source("1 - (2 - 3);").unwrap();
        assert_eq!(formatted, "1 - (2 - 3);\n");
    }

    #[test]
    fn test_format_source_canonicalizes_comprehension_spacing() {
        let formatted = format_source("{yield x+1|x in Seq[1,2,3]|x>1 and x<3};").unwrap();
        assert_eq!(formatted, "{ yield x + 1 | x in Seq[1, 2, 3] | x > 1 and x < 3 };\n");
    }

    #[test]
    fn test_format_source_canonicalizes_nested_selector_and_call_chains() {
        let formatted = format_source("f(input<name>!)(g((1 + 2) * 3));").unwrap();
        assert_eq!(formatted, "f(input<\"name\">!)(g((1 + 2) * 3));\n");
    }

    #[test]
    fn test_format_source_canonicalizes_selector_after_call() {
        let formatted = format_source("(f(input))<name>?;").unwrap();
        assert_eq!(formatted, "f(input)<\"name\">?;\n");
    }

    #[test]
    fn test_unbound_placeholder_outside_pipe() {
        use crate::eval::eval_program;
        let tokens = lexer::lex("•;").unwrap();
        let prog = parser::parse(tokens).unwrap();
        let mut env = crate::Env::new();
        let result = eval_program(&prog, &mut env).unwrap();
        assert_eq!(
            result,
            Some(Value::Fail_(
                "t_sda_unbound_placeholder".to_string(),
                "unbound placeholder".to_string(),
            ))
        );
    }

    #[test]
    fn test_as_bag_kv_from_bag_of_bind() {
        use crate::eval::eval_program;
        let tokens = lexer::lex(r#"asBagKV(Bag{Bind("a", 1)});"#).unwrap();
        let prog = parser::parse(tokens).unwrap();
        let mut env = crate::Env::new();
        let result = eval_program(&prog, &mut env).unwrap();
        assert!(matches!(result, Some(Value::Ok_(_))));
    }

    #[test]
    fn test_as_bag_kv_wrong_shape_not_bag() {
        let result = r(r#"asBagKV(Seq[1, 2]);"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }

    #[test]
    fn test_as_bag_kv_wrong_shape_non_bind_element() {
        let result = r(r#"asBagKV(Bag{1, 2});"#);
        assert_eq!(
            result,
            serde_json::json!({"$type": "fail", "$code": "t_sda_wrong_shape", "$msg": "wrong shape"})
        );
    }
}

fn encode_base16(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02x}").expect("write to string");
    }
    out
}

fn decode_base16(src: &str) -> Result<Vec<u8>, String> {
    if src.len() % 2 != 0 {
        return Err("expected even-length base16 string".to_string());
    }

    let mut out = Vec::with_capacity(src.len() / 2);
    let mut chars = src.chars();
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        let hi = hi
            .to_digit(16)
            .ok_or_else(|| "expected base16 digits only".to_string())?;
        let lo = lo
            .to_digit(16)
            .ok_or_else(|| "expected base16 digits only".to_string())?;
        out.push(((hi << 4) | lo) as u8);
    }

    Ok(out)
}
