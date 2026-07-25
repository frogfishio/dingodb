use sda_lib::{run, run_with_input_binding, SdaError};

fn run_json(expr: &str) -> serde_json::Value {
    run(expr, serde_json::Value::Null).expect("run failed")
}

fn run_json_with_input(expr: &str, binding: &str, input: serde_json::Value) -> serde_json::Value {
    run_with_input_binding(expr, binding, input).expect("run failed")
}

fn assert_same_result(expr_a: &str, expr_b: &str) {
    assert_eq!(run_json(expr_a), run_json(expr_b));
}

fn assert_fail(expr: &str, code: &str, msg: &str) {
    assert_eq!(
        run_json(expr),
        serde_json::json!({
            "$type": "fail",
            "$code": code,
            "$msg": msg,
        })
    );
}

fn assert_parse_error(expr: &str, expected_code: &str, expected_msg: &str) {
    let err = run(expr, serde_json::Value::Null).expect_err("expected parse error");
    match err {
        SdaError::Parse(parse_err) => {
            let rendered = parse_err.to_string();
            assert!(
                rendered.contains(expected_code),
                "missing code in error: {rendered}"
            );
            assert!(
                rendered.contains(expected_msg),
                "missing msg in error: {rendered}"
            );
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

mod section_6_eliminators {
    use super::*;

    #[test]
    fn wrong_shape_on_total_map_projection() {
        assert_fail(
            r#"Map{"name" -> "Ada"}<"name">;"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn missing_key_on_required_map_projection() {
        assert_fail(r#"Map{}<"name">!;"#, "t_sda_missing_key", "missing key");
    }

    #[test]
    fn duplicate_key_on_optional_bagkv_projection_is_none() {
        assert_eq!(
            run_json(r#"BagKV{"k" -> 1, "k" -> 2}<"k">?;"#),
            serde_json::json!({"$type": "none"})
        );
    }

    #[test]
    fn duplicate_key_on_required_bagkv_projection() {
        assert_fail(
            r#"BagKV{"k" -> 1, "k" -> 2}<"k">!;"#,
            "t_sda_duplicate_key",
            "duplicate key",
        );
    }

    #[test]
    fn null_value_is_present_for_optional_map_projection() {
        assert_eq!(
            run_json(r#"Map{"x" -> null}<"x">?;"#),
            serde_json::json!({"$type": "some", "$value": null})
        );
    }

    #[test]
    fn missing_key_is_none_for_optional_map_projection() {
        assert_eq!(
            run_json(r#"Map{}<"x">?;"#),
            serde_json::json!({"$type": "none"})
        );
    }

    #[test]
    fn unknown_field_on_total_prod_projection() {
        assert_fail(
            r#"Prod{name: "Ada"}<age>;"#,
            "t_sda_unknown_field",
            "unknown field",
        );
    }

    #[test]
    fn optional_prod_projection_is_wrong_shape() {
        assert_fail(
            r#"Prod{name: "Ada"}<name>?;"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn required_prod_projection_is_wrong_shape() {
        assert_fail(
            r#"Prod{name: "Ada"}<name>!;"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }
}

mod section_7_normalization {
    use super::*;

    #[test]
    fn wrong_shape_on_normalize_unique() {
        assert_fail(
            r#"normalizeUnique(Seq[1, 2]);"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn normalize_unique_ok_on_unique_keys() {
        assert_eq!(
            run_json(r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#),
            serde_json::json!({"$type": "ok", "$value": {"a": 1, "b": 2}})
        );
    }

    #[test]
    fn normalize_unique_fails_on_duplicates() {
        assert_fail(
            r#"normalizeUnique(BagKV{"k" -> 1, "k" -> 2});"#,
            "t_sda_duplicate_key",
            "duplicate key",
        );
    }

    #[test]
    fn normalize_unique_is_order_independent() {
        assert_same_result(
            r#"normalizeUnique(BagKV{"b" -> 2, "a" -> 1});"#,
            r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#,
        );
    }

    /// §7.2 / §14.1: core exposes no first-wins or last-wins over unordered BagKV.
    #[test]
    fn normalize_first_is_not_core() {
        assert_fail(
            r#"normalizeFirst(BagKV{"k" -> 1, "k" -> 2});"#,
            "t_sda_unbound_name",
            "unbound name",
        );
    }

    #[test]
    fn normalize_last_is_not_core() {
        assert_fail(
            r#"normalizeLast(BagKV{"k" -> 1, "k" -> 2});"#,
            "t_sda_unbound_name",
            "unbound name",
        );
    }
}

mod section_8_algebra {
    use super::*;

    #[test]
    fn division_by_zero_is_stable() {
        assert_fail("1 / 0;", "t_sda_div_by_zero", "division by zero");
    }

    #[test]
    fn set_union_is_canonical() {
        assert_eq!(
            run_json("Set{3, 1} union Set{2, 1};"),
            serde_json::json!({"$type": "set", "$items": [1, 2, 3]})
        );
    }

    #[test]
    fn set_intersection_is_canonical() {
        assert_eq!(
            run_json("Set{3, 1, 2} inter Set{2, 3, 4};"),
            serde_json::json!({"$type": "set", "$items": [2, 3]})
        );
    }

    #[test]
    fn set_difference_is_canonical() {
        assert_eq!(
            run_json("Set{3, 1, 2} diff Set{2};"),
            serde_json::json!({"$type": "set", "$items": [1, 3]})
        );
    }

    #[test]
    fn bag_union_adds_multiplicities_canonically() {
        assert_eq!(
            run_json("Bag{3, 1, 2} bunion Bag{2, 1};"),
            serde_json::json!({"$type": "bag", "$items": [1, 1, 2, 2, 3]})
        );
    }

    #[test]
    fn bag_difference_subtracts_multiplicities_canonically() {
        assert_eq!(
            run_json("Bag{3, 1, 2, 2, 1} bdiff Bag{2, 1, 4};"),
            serde_json::json!({"$type": "bag", "$items": [1, 2, 3]})
        );
    }

    #[test]
    fn set_union_is_commutative_over_nested_value_corpus() {
        let cases = [
            ("Set{3, 1}", "Set{2, 1}"),
            (
                r#"Set{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"}"#,
                r#"Set{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a"}"#,
            ),
            (
                r#"Set{Bytes("00ff"), false, null}"#,
                r#"Set{null, Bytes("00ff"), true}"#,
            ),
        ];

        for (left, right) in cases {
            assert_same_result(
                &format!("{left} union {right};"),
                &format!("{right} union {left};"),
            );
        }
    }

    #[test]
    fn set_union_nested_values_has_single_canonical_encoding() {
        assert_eq!(
            run_json(
                r#"Set{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"} union Set{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a"};"#,
            ),
            serde_json::json!({
                "$type": "set",
                "$items": [
                    "a",
                    "z",
                    [1, 2],
                    {"a": 1, "b": 2}
                ]
            })
        );
    }

    #[test]
    fn set_intersection_is_associative_over_nested_value_corpus() {
        let cases = [(
            r#"Set{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z", Bytes("00ff")}"#,
            r#"Set{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a", Bytes("00ff")}"#,
            r#"Set{Seq[1, 2], Bytes("00ff"), true}"#,
        )];

        for (left, middle, right) in cases {
            assert_same_result(
                &format!("({left} inter {middle}) inter {right};"),
                &format!("{left} inter ({middle} inter {right});"),
            );
        }
    }

    #[test]
    fn set_difference_self_is_empty_over_corpus() {
        let cases = [
            "Set{3, 1, 2}",
            r#"Set{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"}"#,
            r#"Set{Bytes("00ff"), false, null}"#,
        ];

        for set_expr in cases {
            assert_eq!(
                run_json(&format!("{set_expr} diff {set_expr};")),
                serde_json::json!({"$type": "set", "$items": []})
            );
        }
    }

    #[test]
    fn bag_union_is_commutative_over_nested_value_corpus() {
        let cases = [
            ("Bag{3, 1, 2}", "Bag{2, 1}"),
            (
                r#"Bag{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"}"#,
                r#"Bag{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a"}"#,
            ),
            (
                r#"Bag{Bytes("00ff"), false, null}"#,
                r#"Bag{null, Bytes("00ff"), true}"#,
            ),
        ];

        for (left, right) in cases {
            assert_same_result(
                &format!("{left} bunion {right};"),
                &format!("{right} bunion {left};"),
            );
        }
    }

    #[test]
    fn bag_union_nested_values_has_single_canonical_encoding() {
        assert_eq!(
            run_json(
                r#"Bag{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"} bunion Bag{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a"};"#,
            ),
            serde_json::json!({
                "$type": "bag",
                "$items": [
                    "a",
                    "z",
                    [1, 2],
                    [1, 2],
                    {"a": 1, "b": 2},
                    {"a": 1, "b": 2}
                ]
            })
        );
    }

    #[test]
    fn bag_union_is_associative_over_nested_value_corpus() {
        let cases = [(
            r#"Bag{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"}"#,
            r#"Bag{Map{"a" -> 1, "b" -> 2}, Seq[1, 2], "a"}"#,
            r#"Bag{Seq[1, 2], Bytes("00ff"), true}"#,
        )];

        for (left, middle, right) in cases {
            assert_same_result(
                &format!("({left} bunion {middle}) bunion {right};"),
                &format!("{left} bunion ({middle} bunion {right});"),
            );
        }
    }

    #[test]
    fn bag_difference_self_is_empty_over_corpus() {
        let cases = [
            "Bag{3, 1, 2}",
            r#"Bag{Map{"b" -> 2, "a" -> 1}, Seq[1, 2], "z"}"#,
            r#"Bag{Bytes("00ff"), false, null}"#,
        ];

        for bag_expr in cases {
            assert_eq!(
                run_json(&format!("{bag_expr} bdiff {bag_expr};")),
                serde_json::json!({"$type": "bag", "$items": []})
            );
        }
    }

    #[test]
    fn bag_difference_floors_at_zero_over_nested_value_corpus() {
        assert_eq!(
            run_json(
                r#"Bag{Map{"b" -> 2, "a" -> 1}, Map{"a" -> 1, "b" -> 2}, Seq[1, 2], Seq[1, 2]} bdiff Bag{Map{"a" -> 1, "b" -> 2}, Map{"a" -> 1, "b" -> 2}, Map{"a" -> 1, "b" -> 2}, Seq[1, 2]};"#,
            ),
            serde_json::json!({
                "$type": "bag",
                "$items": [[1, 2]]
            })
        );
    }

    #[test]
    fn map_canonical_serialization_is_order_independent() {
        assert_same_result(r#"Map{"b" -> 2, "a" -> 1};"#, r#"Map{"a" -> 1, "b" -> 2};"#);
    }
}

mod section_9_comprehensions {
    use super::*;

    #[test]
    fn seq_comprehension_filters_in_place() {
        assert_eq!(
            run_json(r#"{ a in Seq[1, 2, 3] | a > 1 };"#),
            serde_json::json!([2, 3])
        );
    }

    #[test]
    fn seq_comprehension_yield_projects_values() {
        assert_eq!(
            run_json(r#"{ yield a + 1 | a in Seq[1, 2, 3] | a < 3 };"#),
            serde_json::json!([2, 3])
        );
    }

    #[test]
    fn set_comprehension_preserves_set_carrier() {
        assert_eq!(
            run_json(r#"{ a in Set{1, 2, 3} | a > 1 };"#),
            serde_json::json!({"$type": "set", "$items": [2, 3]})
        );
    }

    #[test]
    fn bag_comprehension_preserves_bag_carrier() {
        assert_eq!(
            run_json(r#"{ a in Bag{1, 1, 2} | a > 1 };"#),
            serde_json::json!({"$type": "bag", "$items": [2]})
        );
    }

    #[test]
    fn bagkv_comprehension_exposes_bind_values() {
        assert_eq!(
            run_json(r#"{ yield a<val> | a in BagKV{"x" -> 1, "y" -> 2} };"#),
            serde_json::json!({"$type": "bag", "$items": [1, 2]})
        );
    }

    #[test]
    fn non_iterable_comprehension_source_is_wrong_shape() {
        assert_fail(r#"{ a in 1 | true };"#, "t_sda_wrong_shape", "wrong shape");
    }

    #[test]
    fn non_bool_comprehension_predicate_is_wrong_shape() {
        assert_fail(
            r#"{ a in Seq[1] | 1 };"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }
}

mod section_10_pipe {
    use super::*;

    #[test]
    fn unbound_placeholder_is_stable() {
        assert_fail("_;", "t_sda_unbound_placeholder", "unbound placeholder");
    }

    #[test]
    fn unbound_bullet_placeholder_is_stable() {
        assert_fail("•;", "t_sda_unbound_placeholder", "unbound placeholder");
    }

    #[test]
    fn placeholder_pipeline_composes_explicitly() {
        assert_eq!(
            run_json(r#"Seq[1, 2] |> _ ++ Seq[3];"#),
            serde_json::json!([1, 2, 3])
        );
    }

    #[test]
    fn nested_pipes_bind_to_nearest_enclosing_pipe() {
        // Inner `_` is 1; outer `_` is 10 → 11.
        assert_eq!(run_json(r#"10 |> (_ + (1 |> _));"#), serde_json::json!(11));
        // Both `_` in the inner pipe bind to nearest left (1), not outer 10.
        assert_eq!(run_json(r#"10 |> (1 |> _ + _);"#), serde_json::json!(2));
    }

    #[test]
    fn bullet_and_underscore_are_synonyms_in_pipe() {
        assert_same_result(r#"5 |> _ + 1;"#, r#"5 |> • + 1;"#);
    }

    #[test]
    fn pipe_does_not_insert_implicit_argument() {
        assert_fail(
            r#"BagKV{"k" -> 1} |> normalizeUnique();"#,
            "t_sda_arity_mismatch",
            "arity mismatch",
        );
    }
}

mod section_11_core_functions {
    use super::*;

    #[test]
    fn keys_helper_returns_wrong_shape_for_non_map() {
        assert_fail(r#"keys(Seq[1]);"#, "t_sda_wrong_shape", "wrong shape");
    }

    #[test]
    fn values_helper_returns_wrong_shape_for_non_map() {
        assert_fail(r#"values(Seq[1]);"#, "t_sda_wrong_shape", "wrong shape");
    }

    #[test]
    fn count_helper_returns_wrong_shape_for_non_bag() {
        assert_fail(r#"count(1, Seq[1]);"#, "t_sda_wrong_shape", "wrong shape");
    }

    #[test]
    fn bind_opt_returns_wrong_shape_for_non_option() {
        assert_fail(
            r#"bindOpt(1, x => Some(x));"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn bind_res_returns_wrong_shape_for_non_result() {
        assert_fail(
            r#"bindRes(1, x => Ok(x));"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn or_else_opt_preserves_option_wrapper() {
        assert_eq!(
            run_json(r#"orElseOpt(Some(1), Some(2));"#),
            serde_json::json!({"$type": "some", "$value": 1})
        );
    }

    #[test]
    fn or_else_res_preserves_result_wrapper() {
        assert_eq!(
            run_json(r#"orElseRes(Ok(1), Ok(2));"#),
            serde_json::json!({"$type": "ok", "$value": 1})
        );
    }
}

mod section_11_standalone_helpers {
    use super::*;

    #[test]
    fn membership_on_seq_is_supported() {
        assert_eq!(run_json("2 in Seq[1, 2, 3];"), serde_json::json!(true));
    }

    #[test]
    fn membership_on_map_uses_string_keys() {
        assert_eq!(
            run_json(r#""name" in Map{"name" -> 1};"#),
            serde_json::json!(true)
        );
    }

    #[test]
    fn membership_on_prod_uses_field_names() {
        assert_eq!(
            run_json(r#""name" in Prod{name: 1};"#),
            serde_json::json!(true)
        );
    }

    #[test]
    fn membership_on_map_requires_string_probe() {
        assert_fail(
            r#"1 in Map{"name" -> 1};"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn keys_returns_set_of_map_keys() {
        assert_eq!(
            run_json(r#"keys(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!({"$type": "set", "$items": ["a", "b"]})
        );
    }

    #[test]
    fn values_returns_seq_in_ascending_key_order() {
        assert_eq!(
            run_json(r#"values(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!([1, 2])
        );
    }

    #[test]
    fn count_returns_bag_multiplicity() {
        assert_eq!(run_json(r#"count(1, Bag{1, 2, 1});"#), serde_json::json!(2));
    }

    #[test]
    fn typeof_returns_kind_tag() {
        assert_eq!(run_json(r#"typeOf(1);"#), serde_json::json!("num"));
        assert_eq!(run_json(r#"typeOf(Some(1));"#), serde_json::json!("some"));
    }
}

mod section_13_worked_examples {
    use super::*;

    #[test]
    fn jsonish_filter_example_replays() {
        let input = serde_json::json!([
            {"$type": "prod", "$fields": {"name": "steve", "city": "la"}},
            {"$type": "prod", "$fields": {"name": "steve", "city": "ny"}},
            {"$type": "prod", "$fields": {"name": "ada", "city": "la"}}
        ]);

        assert_eq!(
            run_json_with_input(
                r#"{ a in A | a<name> = "steve" and a<city> in Set{"la","ny"} };"#,
                "A",
                input,
            ),
            serde_json::json!([
                {"$type": "prod", "$fields": {"city": "la", "name": "steve"}},
                {"$type": "prod", "$fields": {"city": "ny", "name": "steve"}}
            ])
        );
    }

    #[test]
    fn explicit_bind_comprehension_example_replays() {
        assert_eq!(
            run_json(r#"{ yield Bind("x", 1) | a in Seq[1] };"#),
            serde_json::json!([{"$type": "bind", "$key": "x", "$val": 1}])
        );
    }
}

mod section_12_static_selector_errors {
    use super::*;

    #[test]
    fn selector_not_static_tag_is_stable() {
        assert_parse_error("{a b};", "t_sda_selector_not_static", "selector not static");
    }

    #[test]
    fn duplicate_label_tag_is_stable() {
        assert_parse_error(
            "{a a};",
            "t_sda_duplicate_label_in_selector",
            "duplicate label",
        );
    }

    #[test]
    fn reserved_placeholder_in_let_is_stable() {
        assert_parse_error(
            "let _ = 1;",
            "t_sda_reserved_placeholder",
            "reserved placeholder",
        );
    }

    #[test]
    fn reserved_placeholder_as_lambda_param_is_stable() {
        assert_parse_error(
            "_ => 1;",
            "t_sda_reserved_placeholder",
            "reserved placeholder",
        );
    }

    #[test]
    fn invalid_map_key_tag_is_stable() {
        assert_parse_error("Map{a -> 1};", "t_sda_invalid_map_key", "invalid map key");
    }

    #[test]
    fn invalid_bagkv_key_tag_is_stable() {
        assert_parse_error(
            "BagKV{1 -> 1};",
            "t_sda_invalid_bagkv_key",
            "invalid bagkv key",
        );
    }

    #[test]
    fn invalid_generator_binding_reports_generator_shape() {
        assert_parse_error(
            "{ 1 in Seq[1] };",
            "Expected generator expression `name in collection`",
            "generator expression `name in collection`",
        );
    }

    #[test]
    fn general_bind_sugar_is_not_required_in_standalone() {
        assert_parse_error(r#"{ yield "x" -> 1 | a in Seq[1] };"#, "Expected", "Arrow");
    }
}

mod section_12_invocation_failures {
    use super::*;

    #[test]
    fn unbound_name_is_stable() {
        assert_fail("missing;", "t_sda_unbound_name", "unbound name");
    }

    #[test]
    fn not_callable_is_stable() {
        assert_fail("1(2);", "t_sda_not_callable", "not callable");
    }

    #[test]
    fn lambda_arity_mismatch_is_stable() {
        assert_fail("(x => x)(1, 2);", "t_sda_arity_mismatch", "arity mismatch");
    }
}

/// Automated coverage for SDA_SPEC §14.1 minimal suite outline.
mod section_14_1_minimal_suite {
    use super::*;

    #[test]
    fn placeholder_scoping_unbound_and_nested() {
        assert_fail("_;", "t_sda_unbound_placeholder", "unbound placeholder");
        assert_fail("•;", "t_sda_unbound_placeholder", "unbound placeholder");
        assert_eq!(run_json(r#"10 |> (_ + (1 |> _));"#), serde_json::json!(11));
        assert_eq!(run_json(r#"10 |> (1 |> _ + _);"#), serde_json::json!(2));
    }

    #[test]
    fn bagkv_duplicate_optional_and_required() {
        assert_eq!(
            run_json(r#"BagKV{"k" -> 1, "k" -> 2}<"k">?;"#),
            serde_json::json!({"$type": "none"})
        );
        assert_fail(
            r#"BagKV{"k" -> 1, "k" -> 2}<"k">!;"#,
            "t_sda_duplicate_key",
            "duplicate key",
        );
    }

    #[test]
    fn normalization_unique_and_no_ordered_policies() {
        assert_eq!(
            run_json(r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#),
            serde_json::json!({"$type": "ok", "$value": {"a": 1, "b": 2}})
        );
        assert_fail(
            r#"normalizeUnique(BagKV{"k" -> 1, "k" -> 2});"#,
            "t_sda_duplicate_key",
            "duplicate key",
        );
        assert_same_result(
            r#"normalizeUnique(BagKV{"b" -> 2, "a" -> 1});"#,
            r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#,
        );
        assert_fail(
            r#"normalizeFirst(BagKV{"k" -> 1});"#,
            "t_sda_unbound_name",
            "unbound name",
        );
        assert_fail(
            r#"normalizeLast(BagKV{"k" -> 1});"#,
            "t_sda_unbound_name",
            "unbound name",
        );
    }

    #[test]
    fn equality_core_cases() {
        assert_eq!(
            run_json(r#"Prod{a: 1, b: 2} = Prod{b: 2, a: 1};"#),
            serde_json::json!(true)
        );
        assert_eq!(
            run_json(r#"BagKV{"k" -> 1, "k" -> 2} = BagKV{"k" -> 2, "k" -> 1};"#),
            serde_json::json!(true)
        );
        assert_eq!(run_json(r#"Some(null) = None;"#), serde_json::json!(false));
        assert_eq!(
            run_json(r#"Ok(1) = Fail("x", "y");"#),
            serde_json::json!(false)
        );
    }

    #[test]
    fn standalone_helper_profile_cases() {
        assert_eq!(
            run_json(r#"keys(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!({"$type": "set", "$items": ["a", "b"]})
        );
        assert_eq!(
            run_json(r#"values(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!([1, 2])
        );
        assert_eq!(run_json(r#"count(1, Bag{1, 2, 1});"#), serde_json::json!(2));
    }

    #[test]
    fn carrier_preservation_seq_set_bag() {
        assert_eq!(
            run_json(r#"{ a in Seq[1, 2, 3] | a > 1 };"#),
            serde_json::json!([2, 3])
        );
        assert_eq!(
            run_json(r#"{ a in Set{1, 2, 3} | a > 1 };"#),
            serde_json::json!({"$type": "set", "$items": [2, 3]})
        );
        assert_eq!(
            run_json(r#"{ a in Bag{1, 1, 2} | a > 1 };"#),
            serde_json::json!({"$type": "bag", "$items": [2]})
        );
    }

    #[test]
    fn null_vs_absence() {
        assert_eq!(
            run_json(r#"Map{"x" -> null}<"x">?;"#),
            serde_json::json!({"$type": "some", "$value": null})
        );
        assert_eq!(
            run_json(r#"Map{}<"x">?;"#),
            serde_json::json!({"$type": "none"})
        );
    }

    #[test]
    fn unicode_ascii_operator_synonyms() {
        assert_same_result(r#"Map{"a" -> 1};"#, r#"Map{"a" → 1};"#);
        assert_same_result(r#"(x => x + 1)(2);"#, r#"(x ↦ x + 1)(2);"#);
        assert_same_result(r#"5 |> _ + 1;"#, r#"5 |> • + 1;"#);
    }

    #[test]
    fn bind_constructor_is_standalone_binding_form() {
        assert_eq!(
            run_json(r#"Bind("k", 1);"#),
            serde_json::json!({"$type": "bind", "$key": "k", "$val": 1})
        );
        assert_eq!(
            run_json(r#"Bind("k", 1) = Bind("k", 1);"#),
            serde_json::json!(true)
        );
    }
}

/// Full SDA_SPEC §14 MUST lock (beyond the §14.1 minimal outline).
///
/// Each test maps to one of the 13 normative MUST bullets. Together with the
/// golden corpus under `tests/sda/` this freezes standalone behavior under
/// [`sda_lib::CONFORMANCE_CORPUS_TAG`].
mod section_14_must_lock {
    use super::*;

    #[test]
    fn must_1_three_eliminators() {
        // ?, !, and total form — Map optional/required + Prod total.
        assert_eq!(
            run_json(r#"Map{"x" -> null}<"x">?;"#),
            serde_json::json!({"$type": "some", "$value": null})
        );
        assert_fail(r#"Map{}<"name">!;"#, "t_sda_missing_key", "missing key");
        assert_eq!(
            run_json(r#"Prod{name: "Ada"}<name>;"#),
            serde_json::json!("Ada")
        );
    }

    #[test]
    fn must_2_normalization_semantics() {
        assert_eq!(
            run_json(r#"normalizeUnique(BagKV{"a" -> 1, "b" -> 2});"#),
            serde_json::json!({"$type": "ok", "$value": {"a": 1, "b": 2}})
        );
        assert_fail(
            r#"normalizeUnique(BagKV{"k" -> 1, "k" -> 2});"#,
            "t_sda_duplicate_key",
            "duplicate key",
        );
        assert_fail(
            r#"normalizeFirst(BagKV{"k" -> 1});"#,
            "t_sda_unbound_name",
            "unbound name",
        );
    }

    #[test]
    fn must_3_carrier_preservation() {
        assert_eq!(
            run_json(r#"{ a in Seq[1, 2] | true };"#),
            serde_json::json!([1, 2])
        );
        assert_eq!(
            run_json(r#"{ a in Set{1, 2} | true };"#),
            serde_json::json!({"$type": "set", "$items": [1, 2]})
        );
        assert_eq!(
            run_json(r#"{ a in Bag{1, 1} | true };"#),
            serde_json::json!({"$type": "bag", "$items": [1, 1]})
        );
    }

    #[test]
    fn must_4_stable_error_codes() {
        assert_fail("1 / 0;", "t_sda_div_by_zero", "division by zero");
        assert_fail("_;", "t_sda_unbound_placeholder", "unbound placeholder");
        assert_fail("missing;", "t_sda_unbound_name", "unbound name");
        assert_fail("1(2);", "t_sda_not_callable", "not callable");
    }

    #[test]
    fn must_5_unicode_and_ascii_spellings() {
        assert_same_result(r#"Map{"a" -> 1};"#, r#"Map{"a" → 1};"#);
        assert_same_result(r#"(x => x)(1);"#, r#"(x ↦ x)(1);"#);
        assert_same_result("2 in Set{1, 2};", "2 ∈ Set{1, 2};");
        assert_same_result("true and false;", "true ∧ false;");
        assert_same_result("true or false;", "true ∨ false;");
        assert_same_result("not false;", "¬false;");
        assert_same_result("1 != 2;", "1 ≠ 2;");
        assert_same_result("1 <= 2;", "1 ≤ 2;");
        assert_same_result("2 >= 1;", "2 ≥ 1;");
        assert_same_result("Set{1} union Set{2};", "Set{1} ∪ Set{2};");
        assert_same_result("Set{1, 2} inter Set{2};", "Set{1, 2} ∩ Set{2};");
        assert_same_result("Set{1, 2} diff Set{2};", r#"Set{1, 2} \ Set{2};"#);
        assert_same_result("Bag{1} bunion Bag{2};", "Bag{1} ⊎ Bag{2};");
        assert_same_result("Bag{1, 1} bdiff Bag{1};", "Bag{1, 1} ⊖ Bag{1};");
        assert_same_result(r#"5 |> _ + 1;"#, r#"5 |> • + 1;"#);
        let input = serde_json::json!({"name": "Ada"});
        assert_eq!(
            run_json_with_input(r#"input<"name">!;"#, "input", input.clone()),
            run_json_with_input(r#"input⟨"name"⟩!;"#, "input", input)
        );
    }

    #[test]
    fn must_6_pipe_scoping_and_left_associativity() {
        assert_eq!(run_json(r#"10 |> (_ + (1 |> _));"#), serde_json::json!(11));
        assert_eq!(run_json(r#"1 |> _ + 1 |> _ * 2;"#), serde_json::json!(4));
        assert_fail(
            r#"BagKV{"k" -> 1} |> normalizeUnique();"#,
            "t_sda_arity_mismatch",
            "arity mismatch",
        );
    }

    #[test]
    fn must_7_bagkv_comprehension_binding() {
        assert_eq!(
            run_json(r#"{ yield a<val> | a in BagKV{"x" -> 1, "y" -> 2} };"#),
            serde_json::json!({"$type": "bag", "$items": [1, 2]})
        );
        assert_eq!(
            run_json(r#"{ yield a<key> | a in BagKV{"x" -> 1} };"#),
            serde_json::json!({"$type": "bag", "$items": ["x"]})
        );
    }

    #[test]
    fn must_8_prod_vs_map_access_distinction() {
        assert_eq!(
            run_json(r#"Prod{name: "Ada"}<name>;"#),
            serde_json::json!("Ada")
        );
        assert_fail(
            r#"Map{"name" -> "Ada"}<"name">;"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
        assert_fail(
            r#"Prod{name: "Ada"}<name>?;"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
    }

    #[test]
    fn must_9_standalone_helper_contracts() {
        assert_eq!(
            run_json(r#"keys(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!({"$type": "set", "$items": ["a", "b"]})
        );
        assert_eq!(
            run_json(r#"values(Map{"b" -> 2, "a" -> 1});"#),
            serde_json::json!([1, 2])
        );
        assert_eq!(run_json(r#"count(1, Bag{1, 2, 1});"#), serde_json::json!(2));
        assert_eq!(run_json(r#"typeOf(null);"#), serde_json::json!("null"));
        assert_eq!(
            run_json(r#"bindOpt(Some(1), x => Some(x + 1));"#),
            serde_json::json!({"$type": "some", "$value": 2})
        );
        assert_fail(
            r#"bindOpt(Some(1), x => x);"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
        assert_fail(
            r#"bindRes(Ok(1), x => x);"#,
            "t_sda_wrong_shape",
            "wrong shape",
        );
        assert_eq!(
            run_json(r#"bindRes(Ok(1), x => Ok(x + 1));"#),
            serde_json::json!({"$type": "ok", "$value": 2})
        );
    }

    #[test]
    fn must_10_equality_across_core_kinds() {
        assert_eq!(run_json("null = null;"), serde_json::json!(true));
        assert_eq!(run_json("true = false;"), serde_json::json!(false));
        assert_eq!(run_json(r#""a" = "a";"#), serde_json::json!(true));
        assert_eq!(
            run_json(r#"Bytes("00ff") = Bytes("00ff");"#),
            serde_json::json!(true)
        );
        assert_eq!(run_json("Seq[1, 2] = Seq[1, 2];"), serde_json::json!(true));
        assert_eq!(run_json("Set{1, 2} = Set{2, 1};"), serde_json::json!(true));
        assert_eq!(
            run_json("Bag{1, 1, 2} = Bag{2, 1, 1};"),
            serde_json::json!(true)
        );
        assert_eq!(
            run_json(r#"Map{"a" -> 1} = Map{"a" -> 1};"#),
            serde_json::json!(true)
        );
        assert_eq!(
            run_json(r#"Prod{a: 1, b: 2} = Prod{b: 2, a: 1};"#),
            serde_json::json!(true)
        );
        assert_eq!(
            run_json(r#"BagKV{"k" -> 1, "k" -> 2} = BagKV{"k" -> 2, "k" -> 1};"#),
            serde_json::json!(true)
        );
        assert_eq!(
            run_json(r#"Bind("k", 1) = Bind("k", 1);"#),
            serde_json::json!(true)
        );
        assert_eq!(run_json("Some(1) = Some(1);"), serde_json::json!(true));
        assert_eq!(run_json("None = None;"), serde_json::json!(true));
        assert_eq!(run_json("Ok(1) = Ok(1);"), serde_json::json!(true));
        assert_eq!(
            run_json(r#"Fail("c", "m") = Fail("c", "m");"#),
            serde_json::json!(true)
        );
        assert_eq!(run_json("Some(null) = None;"), serde_json::json!(false));
    }

    #[test]
    fn must_11_static_and_eval_error_codes_align() {
        // Static parse errors use stable tags; runtime Fail uses the same family.
        assert_parse_error(
            "let _ = 1;",
            "t_sda_reserved_placeholder",
            "reserved placeholder",
        );
        assert_fail("_;", "t_sda_unbound_placeholder", "unbound placeholder");
        assert_fail(r#"Map{}<"x">!;"#, "t_sda_missing_key", "missing key");
    }

    #[test]
    fn must_12_pipe_is_placeholder_composition() {
        assert_eq!(
            run_json(r#"Seq[1, 2] |> _ ++ Seq[3];"#),
            serde_json::json!([1, 2, 3])
        );
        // No implicit insertion of the left value as a call argument.
        assert_fail(
            r#"1 |> (x => x + 1)();"#,
            "t_sda_arity_mismatch",
            "arity mismatch",
        );
    }

    #[test]
    fn must_13_bind_constructor_not_general_arrow_sugar() {
        assert_eq!(
            run_json(r#"Bind("k", 1);"#),
            serde_json::json!({"$type": "bind", "$key": "k", "$val": 1})
        );
        assert_parse_error(r#"{ yield "x" -> 1 | a in Seq[1] };"#, "Expected", "Arrow");
    }

    #[test]
    fn frozen_corpus_tag_is_stable() {
        assert_eq!(sda_lib::CONFORMANCE_CORPUS_TAG, "sda-standalone-v1.0");
        assert_eq!(
            sda_lib::SdaRuntime::conformance_corpus_tag(),
            "sda-standalone-v1.0"
        );
        let version_file = include_str!("sda/VERSION").trim();
        assert_eq!(version_file, sda_lib::CONFORMANCE_CORPUS_TAG);
    }
}

/// Golden-vector runner for `tests/sda/section14_must.json`.
mod section_14_golden_corpus {
    use super::*;

    #[test]
    fn golden_section14_must_vectors() {
        let raw = include_str!("sda/section14_must.json");
        let corpus: serde_json::Value =
            serde_json::from_str(raw).expect("parse golden corpus JSON");
        let tag = corpus["tag"].as_str().expect("corpus.tag string");
        assert_eq!(
            tag,
            sda_lib::CONFORMANCE_CORPUS_TAG,
            "golden corpus tag must match CONFORMANCE_CORPUS_TAG"
        );
        let cases = corpus["cases"].as_array().expect("corpus.cases array");
        assert!(!cases.is_empty(), "golden corpus must not be empty");

        for case in cases {
            let id = case["id"].as_str().unwrap_or("<missing id>");
            let source = case["source"]
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: missing source"));
            let expected = case["expected"].clone();
            let got = if case.get("input").is_some() && !case["input"].is_null() {
                // When the golden entry includes an `input` field, bind it as `input`.
                run_json_with_input(source, "input", case["input"].clone())
            } else {
                run_json(source)
            };
            assert_eq!(
                got, expected,
                "golden case {id} failed\nsource: {source}\ngot: {got}\nexpected: {expected}"
            );
        }
    }
}
