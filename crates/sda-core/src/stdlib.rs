use crate::number::ExactNum;
use crate::eval::{apply_lambda, ensure_comparable, EvalError};
use crate::Value;

pub fn call_stdlib(name: &str, args: Vec<Value>) -> Option<Result<Value, EvalError>> {
    match name {
        "typeOf" => Some(stdlib_type_of(args)),
        "keys" => Some(stdlib_keys(args)),
        "values" => Some(stdlib_values(args)),
        "count" => Some(stdlib_count(args)),
        "normalizeUnique" => Some(stdlib_normalize_unique(args)),
        "normalizeFirst" => Some(stdlib_normalize_first(args)),
        "normalizeLast" => Some(stdlib_normalize_last(args)),
        "Bind" => Some(stdlib_bind(args)),
        "asBagKV" => Some(stdlib_as_bag_kv(args)),
        "mapOpt" => Some(stdlib_map_opt(args)),
        "bindOpt" => Some(stdlib_bind_opt(args)),
        "orElseOpt" => Some(stdlib_or_else_opt(args)),
        "mapRes" => Some(stdlib_map_res(args)),
        "bindRes" => Some(stdlib_bind_res(args)),
        "orElseRes" => Some(stdlib_or_else_res(args)),
        _ => None,
    }
}

fn wrong_shape() -> Value {
    Value::Fail_("t_sda_wrong_shape".to_string(), "wrong shape".to_string())
}

fn check_arity(_name: &str, args: &[Value], expected: usize) -> Result<(), EvalError> {
    if args.len() != expected {
        Err(EvalError::ArityMismatch {
            expected,
            got: args.len(),
        })
    } else {
        Ok(())
    }
}

fn stdlib_type_of(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("typeOf", &args, 1)?;
    let type_str = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Num(_) => "num",
        Value::Str(_) => "str",
        Value::Bytes(_) => "bytes",
        Value::Seq(_) => "seq",
        Value::Set(_) => "set",
        Value::Bag(_) => "bag",
        Value::Map(_) => "map",
        Value::Prod(_) => "prod",
        Value::BagKV(_) => "bagkv",
        Value::Bind(_, _) => "bind",
        Value::Some_(_) => "some",
        Value::None_ => "none",
        Value::Ok_(_) => "ok",
        Value::Fail_(_, _) => "fail",
        Value::Lambda(_, _, _) => "fn",
    };
    Ok(Value::Str(type_str.to_string()))
}

fn stdlib_keys(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("keys", &args, 1)?;
    match &args[0] {
        Value::Map(entries) => Ok(Value::Set(
            entries
                .iter()
                .map(|(k, _)| Value::Str(k.clone()))
                .collect(),
        )),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_values(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("values", &args, 1)?;
    match &args[0] {
        Value::Map(entries) => {
            let mut sorted = entries.clone();
            sorted.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
            Ok(Value::Seq(sorted.into_iter().map(|(_, v)| v).collect()))
        }
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_count(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("count", &args, 2)?;
    let mut iter = args.into_iter();
    let needle = iter.next().unwrap();
    let haystack = iter.next().unwrap();
    if ensure_comparable(&needle).is_err() {
        return Ok(wrong_shape());
    }
    let n = match &haystack {
        Value::Bag(items) => {
            for item in items {
                if ensure_comparable(item).is_err() {
                    return Ok(wrong_shape());
                }
            }
            items.iter().filter(|v| *v == &needle).count()
        }
        _ => return Ok(wrong_shape()),
    };
    Ok(Value::Num(ExactNum::from_usize(n)))
}

fn stdlib_normalize_unique(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("normalizeUnique", &args, 1)?;
    match args.into_iter().next().unwrap() {
        Value::BagKV(pairs) => {
            let mut map: Vec<(String, Value)> = Vec::new();
            for (k, v) in pairs {
                let key_str = match value_as_map_key(&k) {
                    Ok(key) => key,
                    Err(_) => {
                        return Ok(Value::Fail_(
                            "t_sda_wrong_shape".to_string(),
                            "wrong shape".to_string(),
                        ))
                    }
                };
                if map.iter().any(|(existing_key, _)| existing_key == &key_str) {
                    return Ok(Value::Fail_(
                        "t_sda_duplicate_key".to_string(),
                        "duplicate key".to_string(),
                    ));
                }
                map.push((key_str, v));
            }
            Ok(Value::Ok_(Box::new(Value::Map(map))))
        }
        _ => Ok(Value::Fail_(
            "t_sda_wrong_shape".to_string(),
            "wrong shape".to_string(),
        )),
    }
}

fn stdlib_normalize_first(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("normalizeFirst", &args, 1)?;
    match args.into_iter().next().unwrap() {
        Value::BagKV(pairs) => {
            let mut map: Vec<(String, Value)> = Vec::new();
            for (k, v) in pairs {
                let key_str = match value_as_map_key(&k) {
                    Ok(key) => key,
                    Err(_) => {
                        return Ok(Value::Fail_(
                            "t_sda_wrong_shape".to_string(),
                            "wrong shape".to_string(),
                        ))
                    }
                };
                if !map.iter().any(|(existing_key, _)| existing_key == &key_str) {
                    map.push((key_str, v));
                }
            }
            Ok(Value::Map(map))
        }
        _ => Ok(Value::Fail_(
            "t_sda_wrong_shape".to_string(),
            "wrong shape".to_string(),
        )),
    }
}

fn stdlib_normalize_last(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("normalizeLast", &args, 1)?;
    match args.into_iter().next().unwrap() {
        Value::BagKV(pairs) => {
            let mut map: Vec<(String, Value)> = Vec::new();
            for (k, v) in pairs {
                let key_str = match value_as_map_key(&k) {
                    Ok(key) => key,
                    Err(_) => {
                        return Ok(Value::Fail_(
                            "t_sda_wrong_shape".to_string(),
                            "wrong shape".to_string(),
                        ))
                    }
                };
                if let Some(existing) = map.iter_mut().find(|(existing_key, _)| existing_key == &key_str) {
                    existing.1 = v;
                } else {
                    map.push((key_str, v));
                }
            }
            Ok(Value::Map(map))
        }
        _ => Ok(Value::Fail_(
            "t_sda_wrong_shape".to_string(),
            "wrong shape".to_string(),
        )),
    }
}

fn value_as_map_key(v: &Value) -> Result<String, EvalError> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        other => Err(EvalError::TypeError(format!(
            "Map key must be a string, got {other:?}"
        ))),
    }
}

fn stdlib_bind(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("Bind", &args, 2)?;
    let mut iter = args.into_iter();
    let k = iter.next().unwrap();
    let v = iter.next().unwrap();
    Ok(Value::Bind(Box::new(k), Box::new(v)))
}

fn stdlib_as_bag_kv(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("asBagKV", &args, 1)?;
    match args.into_iter().next().unwrap() {
        Value::Bag(items) => {
            let mut pairs = Vec::new();
            for item in items {
                match item {
                    Value::Bind(k, v) => match *k {
                        Value::Str(s) => pairs.push((Value::Str(s), *v)),
                        _ => {
                            return Ok(Value::Fail_(
                                "t_sda_wrong_shape".to_string(),
                                "wrong shape".to_string(),
                            ))
                        }
                    },
                    _ => {
                        return Ok(Value::Fail_(
                            "t_sda_wrong_shape".to_string(),
                            "wrong shape".to_string(),
                        ))
                    }
                }
            }
            Ok(Value::Ok_(Box::new(Value::BagKV(pairs))))
        }
        _ => Ok(Value::Fail_(
            "t_sda_wrong_shape".to_string(),
            "wrong shape".to_string(),
        )),
    }
}

fn stdlib_map_opt(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("mapOpt", &args, 2)?;
    let mut iter = args.into_iter();
    let opt = iter.next().unwrap();
    let f = iter.next().unwrap();
    match opt {
        Value::Some_(inner) => {
            let result = apply_lambda(f, vec![*inner])?;
            Ok(Value::Some_(Box::new(result)))
        }
        Value::None_ => Ok(Value::None_),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_bind_opt(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("bindOpt", &args, 2)?;
    let mut iter = args.into_iter();
    let opt = iter.next().unwrap();
    let f = iter.next().unwrap();
    match opt {
        Value::Some_(inner) => apply_lambda(f, vec![*inner]),
        Value::None_ => Ok(Value::None_),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_or_else_opt(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("orElseOpt", &args, 2)?;
    let mut iter = args.into_iter();
    let opt = iter.next().unwrap();
    let default = iter.next().unwrap();
    match opt {
        Value::Some_(inner) => Ok(Value::Some_(inner)),
        Value::None_ => Ok(default),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_map_res(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("mapRes", &args, 2)?;
    let mut iter = args.into_iter();
    let res = iter.next().unwrap();
    let f = iter.next().unwrap();
    match res {
        Value::Ok_(inner) => {
            let result = apply_lambda(f, vec![*inner])?;
            Ok(Value::Ok_(Box::new(result)))
        }
        Value::Fail_(c, m) => Ok(Value::Fail_(c, m)),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_bind_res(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("bindRes", &args, 2)?;
    let mut iter = args.into_iter();
    let res = iter.next().unwrap();
    let f = iter.next().unwrap();
    match res {
        Value::Ok_(inner) => apply_lambda(f, vec![*inner]),
        Value::Fail_(c, m) => Ok(Value::Fail_(c, m)),
        _ => Ok(wrong_shape()),
    }
}

fn stdlib_or_else_res(args: Vec<Value>) -> Result<Value, EvalError> {
    check_arity("orElseRes", &args, 2)?;
    let mut iter = args.into_iter();
    let res = iter.next().unwrap();
    let default = iter.next().unwrap();
    match res {
        Value::Ok_(inner) => Ok(Value::Ok_(inner)),
        Value::Fail_(_, _) => Ok(default),
        _ => Ok(wrong_shape()),
    }
}