use relay_price_calc_m::calc::{compare, format_coefficient, parse_number, Prices, Verdict};
use relay_price_calc_m::model::{FormState, ModelInput, FIELDS};

fn prices(values: [f64; 5]) -> Prices {
    Prices {
        input: values[0],
        output: values[1],
        cache_write: values[2],
        cache_read: values[3],
        cache_rate_percent: values[4],
    }
}

fn input(values: [&str; 5]) -> ModelInput {
    ModelInput {
        values: values.map(|value| value.to_owned()),
    }
}

fn form_with_coefficients(first: &str, second: &str) -> FormState {
    FormState {
        models: [
            input(["", first, "", "", ""]),
            input(["", second, "", "", ""]),
        ],
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(actual.is_finite(), "expected a finite result, got {actual}");
    assert!(
        (actual - expected).abs() <= expected.abs() * 1e-14,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn upstream_updated_example_has_coefficient_6_84() {
    // (3 + 15) * 20% + (0.3 + 3.75) * 80% = 6.84.
    let example = prices([3.0, 15.0, 3.75, 0.3, 80.0]);
    assert_eq!(example.validate(), Ok(()));
    assert_close(example.coefficient().unwrap(), 6.84);
    assert_eq!(format_coefficient(example.coefficient().unwrap()), "6.84");
    assert_close(
        input(["3", "15", "3.75", "0.3", "80"])
            .coefficient()
            .unwrap(),
        6.84,
    );
}

#[test]
fn cache_rate_boundaries_and_intermediate_values_follow_the_formula() {
    for (rate, expected) in [(0.0, 18.0), (25.0, 14.5125), (80.0, 6.84), (100.0, 4.05)] {
        let value = prices([3.0, 15.0, 3.75, 0.3, rate]);
        assert_eq!(value.validate(), Ok(()), "cache rate {rate}");
        assert_close(value.coefficient().unwrap(), expected);
    }
}

#[test]
fn each_price_uses_the_updated_cache_weight() {
    for (field, expected) in [(0, 8.0), (1, 8.0), (2, 2.0), (3, 2.0)] {
        let mut values = [0.0; 5];
        values[field] = 10.0;
        values[4] = 20.0;
        assert_close(prices(values).coefficient().unwrap(), expected);
    }
}

#[test]
fn updated_formula_changes_the_winner_against_an_output_only_model() {
    let form = FormState {
        models: [
            input(["3", "15", "3.75", "0.3", "80"]),
            input(["", "10", "", "", ""]),
        ],
    };
    assert_eq!(form.coefficient_text(0), "6.84");
    assert_eq!(form.summary().winner, 1);
    assert!(form.summary().detail.contains("31.6%"));
}

#[test]
fn zero_prices_stay_zero_at_every_cache_rate() {
    for rate in [0.0, 0.5, 50.0, 100.0] {
        assert_eq!(prices([0.0, 0.0, 0.0, 0.0, rate]).coefficient(), Ok(0.0));
    }
}

#[test]
fn prices_reject_non_finite_numbers_in_every_field() {
    for (field, name) in FIELDS.iter().enumerate() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut values = [0.0; 5];
            values[field] = bad;
            let value = prices(values);
            let expected = format!("“{name}”不是有效数字");
            assert_eq!(
                value.validate().unwrap_err(),
                expected,
                "field {field}, {bad}"
            );
            assert_eq!(value.coefficient().unwrap_err(), expected);
        }
    }
}

#[test]
fn prices_reject_negative_numbers_in_every_field() {
    for (field, name) in FIELDS.iter().enumerate() {
        let mut values = [0.0; 5];
        values[field] = -0.01;
        let value = prices(values);
        let expected = format!("“{name}”不能是负数");
        assert_eq!(value.validate().unwrap_err(), expected);
        assert_eq!(value.coefficient().unwrap_err(), expected);
    }
}

#[test]
fn cache_rate_cannot_exceed_one_hundred_percent() {
    for rate in [100.000001, 101.0, f64::MAX] {
        let value = prices([0.0, 0.0, 0.0, 0.0, rate]);
        assert_eq!(value.validate().unwrap_err(), "“平均缓存率”不能超过 100%");
        assert_eq!(
            value.coefficient().unwrap_err(),
            "“平均缓存率”不能超过 100%"
        );
    }
}

#[test]
fn coefficient_rejects_overflow_after_validating_finite_inputs() {
    for values in [
        [f64::MAX, f64::MAX, 0.0, 0.0, 0.0],
        [0.0, 0.0, f64::MAX, f64::MAX, 100.0],
        // Match upstream evaluation order: the sums overflow even at zero weight.
        [f64::MAX, f64::MAX, 0.0, 0.0, 100.0],
        [0.0, 0.0, f64::MAX, f64::MAX, 0.0],
        [f64::MAX, f64::MAX, f64::MAX, f64::MAX, 50.0],
    ] {
        let value = prices(values);
        assert_eq!(value.validate(), Ok(()), "{values:?}");
        assert_eq!(value.coefficient().unwrap_err(), "数值过大，无法计算");
    }
}

#[test]
fn finite_extremes_and_inactive_cache_components_do_not_overflow() {
    for (values, expected) in [
        ([f64::MAX, 0.0, 0.0, 0.0, 0.0], f64::MAX),
        ([f64::MAX, 1.0, 2.0, 3.0, 100.0], 5.0),
        ([1.0, 3.0, 2.0, f64::MAX, 0.0], 4.0),
    ] {
        assert_close(prices(values).coefficient().unwrap(), expected);
    }
}

#[test]
fn blank_input_is_zero_including_unicode_whitespace() {
    for text in ["", " ", "\t\n", "　", " \t　\n"] {
        assert_eq!(parse_number(text), Ok(0.0), "{text:?}");
    }
}

#[test]
fn parser_accepts_decimal_exponent_fullwidth_and_percent_notation() {
    for (text, expected) in [
        ("16.59", 16.59),
        (" 16.59 ", 16.59),
        ("１２３４５６７８９０", 1_234_567_890.0),
        ("１６．５９", 16.59),
        ("１６。５９", 16.59),
        ("　８０％　", 80.0),
        ("80%", 80.0),
        ("80 %", 80.0),
        ("８０%", 80.0),
        ("80％", 80.0),
        ("０．５％", 0.5),
        (".5", 0.5),
        ("5.", 5.0),
        ("+2.5", 2.5),
        ("1e-12", 1e-12),
    ] {
        assert_close(parse_number(text).unwrap(), expected);
    }
}

#[test]
fn parser_rejects_invalid_and_non_finite_numbers() {
    for text in [
        "abc",
        "1,000",
        "1 2",
        "1.2.3",
        ".",
        "1%2",
        "%1",
        "１．２。３",
        "NaN",
        "nan",
        "inf",
        "+inf",
        "-inf",
        "Infinity",
        "1e309",
        "-1e309",
    ] {
        let error = parse_number(text).expect_err(text);
        assert!(error.contains("不是有效数字"), "{text:?}: {error}");
    }
}

#[test]
fn parsing_a_negative_number_does_not_bypass_business_validation() {
    assert_eq!(parse_number(" -1.5 "), Ok(-1.5));
    for field in 0..FIELDS.len() {
        let mut value = ModelInput::default();
        value.values[field] = "-1.5".into();
        assert_eq!(value.field_error(field), "不能是负数");
        assert!(value.coefficient().is_err());
    }
    assert_eq!(prices([-0.0; 5]).coefficient(), Ok(0.0));
}

#[test]
fn compare_accepts_exact_equality_and_relative_epsilon_at_multiple_scales() {
    for value in [
        0.0,
        -0.0,
        f64::from_bits(1),
        1e-200,
        1.0,
        16.59,
        1e200,
        f64::MAX,
    ] {
        assert_eq!(compare(value, value), Verdict::Equal, "{value}");
    }
    for scale in [1e-200, 1.0, 1e200] {
        let near = scale * (1.0 + 5e-13);
        assert_ne!(scale, near, "the epsilon case must not be exactly equal");
        assert_eq!(compare(scale, near), Verdict::Equal, "scale {scale}");
        assert_eq!(compare(near, scale), Verdict::Equal, "scale {scale}");
    }
}

#[test]
fn compare_distinguishes_values_outside_relative_epsilon_in_both_directions() {
    for scale in [1e-200, 1.0, 1e200] {
        let higher = scale * (1.0 + 2e-12);
        assert_eq!(compare(scale, higher), Verdict::Model1Cheaper, "{scale}");
        assert_eq!(compare(higher, scale), Verdict::Model2Cheaper, "{scale}");
    }
}

#[test]
fn compare_does_not_treat_tiny_nonzero_values_as_zero_or_equal() {
    for (lower, higher) in [
        (0.0, f64::from_bits(1)),
        (0.0, 1e-300),
        (1e-300, 2e-300),
        (1e-15, 2e-15),
    ] {
        assert_eq!(compare(lower, higher), Verdict::Model1Cheaper);
        assert_eq!(compare(higher, lower), Verdict::Model2Cheaper);
    }
}

#[test]
fn coefficient_formatting_trims_rounds_and_preserves_tiny_nonzero_values() {
    for (value, expected) in [
        (0.0, "0"),
        (-0.0, "0"),
        (1.0, "1"),
        (1000.0, "1000"),
        (1.2, "1.2"),
        (16.59, "16.59"),
        (16.59001, "16.59"),
        (1.23456, "1.2346"),
        (1.99999, "2"),
        (0.0001, "0.0001"),
        (0.00009999, "9.999e-5"),
        (1e-8, "1.000e-8"),
        (1e-300, "1.000e-300"),
        (-1e-8, "-1.000e-8"),
    ] {
        assert_eq!(format_coefficient(value), expected, "{value}");
    }
}

#[test]
fn empty_model_input_has_no_errors_and_a_zero_coefficient() {
    for model in [ModelInput::default(), input(["", " ", "\t", "　", "\n"])] {
        for field in 0..FIELDS.len() {
            assert_eq!(model.field_error(field), "");
        }
        assert_eq!(model.coefficient(), Ok(0.0));
    }
}

#[test]
fn model_input_normalizes_all_fields_before_calculating() {
    let model = input(["３", "１５", "３．７５", "０。３", "８０％"]);
    for field in 0..FIELDS.len() {
        assert_eq!(model.field_error(field), "");
    }
    assert_close(model.coefficient().unwrap(), 6.84);
}

#[test]
fn field_errors_are_localized_and_do_not_leak_to_other_fields() {
    for (field, name) in FIELDS.iter().enumerate() {
        for (text, expected) in [
            ("oops", "请输入有效数字"),
            ("NaN", "请输入有效数字"),
            ("-1", "不能是负数"),
        ] {
            let mut model = ModelInput::default();
            model.values[field] = text.into();
            for other in 0..FIELDS.len() {
                assert_eq!(
                    model.field_error(other),
                    if other == field { expected } else { "" }
                );
            }
            assert_eq!(
                model.coefficient().unwrap_err(),
                format!("{name}：{expected}")
            );
        }
    }
    let model = input(["101", "101", "101", "101", "１００．１％"]);
    for field in 0..4 {
        assert_eq!(
            model.field_error(field),
            "",
            "price fields are not percentages"
        );
    }
    assert_eq!(model.field_error(4), "不能超过 100%");
    assert_eq!(
        model.coefficient().unwrap_err(),
        "平均缓存率：不能超过 100%"
    );
}

#[test]
fn field_error_handles_unknown_indexes() {
    for field in [5, 6, usize::MAX] {
        assert_eq!(ModelInput::default().field_error(field), "未知字段");
    }
}

#[test]
fn model_input_reports_the_first_bad_field_in_field_order() {
    let mut model = input(["bad", "-2", "", "", "101"]);
    assert_eq!(model.coefficient().unwrap_err(), "输入：请输入有效数字");
    model.values[0].clear();
    assert_eq!(model.coefficient().unwrap_err(), "输出：不能是负数");
    model.values[1].clear();
    assert_eq!(
        model.coefficient().unwrap_err(),
        "平均缓存率：不能超过 100%"
    );
    model.values[4] = "100".into();
    assert_eq!(model.coefficient(), Ok(0.0));
}

#[test]
fn model_input_propagates_arithmetic_overflow_without_fabricating_a_number() {
    let model = input(["1e308", "1e308", "", "", "0"]);
    for field in 0..FIELDS.len() {
        assert_eq!(
            model.field_error(field),
            "",
            "each input is individually finite"
        );
    }
    assert_eq!(model.coefficient().unwrap_err(), "数值过大，无法计算");
}

#[test]
fn default_form_shows_zero_coefficients_and_an_equal_summary() {
    let form = FormState::default();
    assert_eq!(form.models, [ModelInput::default(), ModelInput::default()]);
    assert_eq!(form.coefficient_text(0), "0");
    assert_eq!(form.coefficient_text(1), "0");
    let summary = form.summary();
    assert_eq!(summary.winner, 0);
    assert_eq!(summary.headline, "两个模型一样贵");
    assert_eq!(summary.detail, "价格系数相同");
}

#[test]
fn setting_each_field_recalculates_coefficients_and_winners_immediately() {
    let mut form = FormState::default();
    for (model, field, text, first, second, winner) in [
        (0, 0, "3", 3.0, 0.0, 2),
        (0, 1, "15", 18.0, 0.0, 2),
        (0, 2, "3.75", 18.0, 0.0, 2),
        (0, 3, "0.3", 18.0, 0.0, 2),
        (0, 4, "80", 6.84, 0.0, 2),
        (1, 1, "20", 6.84, 20.0, 1),
        (1, 1, "10", 6.84, 10.0, 1),
        (0, 4, "100", 4.05, 10.0, 1),
        (1, 1, "4.05", 4.05, 4.05, 0),
        (0, 1, "", 4.05, 4.05, 0),
        (0, 2, "", 0.3, 4.05, 1),
    ] {
        form.set(model, field, text.into());
        assert_eq!(form.models[model].values[field], text);
        assert_close(form.models[0].coefficient().unwrap(), first);
        assert_close(form.models[1].coefficient().unwrap(), second);
        assert_eq!(form.coefficient_text(0), format_coefficient(first));
        assert_eq!(form.coefficient_text(1), format_coefficient(second));
        assert_eq!(
            form.summary().winner,
            winner,
            "after setting {model}/{field} to {text}"
        );
    }
}

#[test]
fn setting_unknown_indexes_leaves_the_form_unchanged() {
    let mut form = form_with_coefficients("3", "4");
    let before = form.clone();
    for (model, field) in [
        (2, 0),
        (0, 5),
        (usize::MAX, 0),
        (0, usize::MAX),
        (usize::MAX, usize::MAX),
    ] {
        form.set(model, field, "bad".into());
        assert_eq!(form, before);
        assert_eq!(form.summary(), before.summary());
    }
}

#[test]
fn invalid_form_fields_only_hide_the_affected_model_coefficient() {
    for bad_model in 0..2 {
        for field in 0..FIELDS.len() {
            let mut form = FormState {
                models: [
                    input(["2", "3", "4", "5", "50"]),
                    input(["6", "7", "8", "9", "25"]),
                ],
            };
            let other = 1 - bad_model;
            let other_before = form.models[other].clone();
            let other_text = form.coefficient_text(other);
            form.set(bad_model, field, "bad".into());
            assert_eq!(form.models[bad_model].field_error(field), "请输入有效数字");
            assert_eq!(form.coefficient_text(bad_model), "—");
            assert_eq!(form.models[other], other_before);
            assert_eq!(form.coefficient_text(other), other_text);
            let summary = form.summary();
            assert_eq!(summary.winner, -1);
            assert_eq!(summary.headline, "请修正标红的数值");
            assert_eq!(summary.detail, "输入有效后会立即重新比较");
        }
    }
}

#[test]
fn fixing_invalid_input_restores_live_comparison_in_either_model() {
    for bad_model in 0..2 {
        for (field, bad, correction) in [(0, "bad", "0"), (1, "-1", "3"), (4, "101", "0")] {
            let mut form = form_with_coefficients("3", "4");
            let baseline = form.summary();
            let coefficients = [form.coefficient_text(0), form.coefficient_text(1)];
            form.set(bad_model, field, bad.into());
            assert_eq!(form.summary().winner, -1);
            assert_eq!(form.coefficient_text(bad_model), "—");
            // Restore the original output in model 2 as well as in model 1.
            let correction = if field == 1 && bad_model == 1 {
                "4"
            } else {
                correction
            };
            form.set(bad_model, field, correction.into());
            assert_eq!(form.models[bad_model].field_error(field), "");
            assert_eq!(form.summary(), baseline);
            assert_eq!(form.coefficient_text(0), coefficients[0]);
            assert_eq!(form.coefficient_text(1), coefficients[1]);
        }
    }
}

#[test]
fn clear_resets_both_models_errors_and_summary_and_is_idempotent() {
    for invalid in [false, true] {
        let mut form = FormState {
            models: [
                input(["3", "15", "3.75", "0.3", "80"]),
                input(["6", "7", "8", "9", "25"]),
            ],
        };
        if invalid {
            form.set(1, 4, "101".into());
            assert_eq!(form.summary().winner, -1);
        }
        for _ in 0..2 {
            form.clear();
            assert_eq!(form, FormState::default());
            assert_eq!(form.coefficient_text(0), "0");
            assert_eq!(form.coefficient_text(1), "0");
            assert_eq!(form.summary(), FormState::default().summary());
            for model in &form.models {
                for field in 0..FIELDS.len() {
                    assert_eq!(model.field_error(field), "");
                }
            }
        }
    }
}

#[test]
fn summary_uses_calculated_values_and_relative_epsilon_for_equality() {
    for form in [
        form_with_coefficients("16.59", "16.59"),
        form_with_coefficients("1", "1.0000000000005"),
        form_with_coefficients("1e-200", "1.0000000000005e-200"),
        FormState {
            models: [
                input(["2", "3", "", "", "0"]),
                input(["", "", "2", "3", "100"]),
            ],
        },
    ] {
        let summary = form.summary();
        assert_eq!(summary.winner, 0);
        assert_eq!(summary.headline, "两个模型一样贵");
        assert_eq!(summary.detail, "价格系数相同");
    }
}

#[test]
fn summary_finds_either_winner_and_uses_the_more_expensive_model_as_baseline() {
    for (first, second, winner, expected_detail) in [
        ("3", "4", 1, "便宜约 25.0% · 以较贵模型为基准"),
        ("4", "3", 2, "便宜约 25.0% · 以较贵模型为基准"),
        ("1", "4", 1, "便宜约 75.0% · 以较贵模型为基准"),
        ("4", "1", 2, "便宜约 75.0% · 以较贵模型为基准"),
    ] {
        let summary = form_with_coefficients(first, second).summary();
        assert_eq!(summary.winner, winner);
        assert_eq!(summary.headline, format!("模型{winner}更便宜"));
        assert_eq!(summary.detail, expected_detail);
    }
}

#[test]
fn summary_reports_one_hundred_percent_saving_for_zero_versus_positive() {
    for positive in ["3", "1e-15", "1e-300"] {
        for (first, second, winner) in [("0", positive, 1), (positive, "0", 2)] {
            let summary = form_with_coefficients(first, second).summary();
            assert_eq!(summary.winner, winner);
            assert_eq!(summary.headline, format!("模型{winner}更便宜"));
            assert_eq!(summary.detail, "便宜约 100.0% · 以较贵模型为基准");
        }
    }
}

#[test]
fn summary_does_not_claim_zero_difference_when_displayed_coefficients_match() {
    for (lower, higher) in [("1", "1.00000000001"), ("1e-200", "1.0000001e-200")] {
        for (first, second, winner) in [(lower, higher, 1), (higher, lower, 2)] {
            let form = form_with_coefficients(first, second);
            assert_eq!(form.coefficient_text(0), form.coefficient_text(1));
            let summary = form.summary();
            assert_eq!(summary.winner, winner);
            assert_eq!(summary.headline, format!("模型{winner}更便宜"));
            assert_eq!(summary.detail, "便宜不足 0.1% · 以较贵模型为基准");
            assert!(!summary.detail.contains("约 0.0%"));
        }
    }
}

#[test]
fn summary_rounding_distinguishes_tiny_savings_from_a_rounded_tenth_percent() {
    for (lower, expected) in [
        ("49.98", "便宜不足 0.1% · 以较贵模型为基准"),
        ("49.97", "便宜约 0.1% · 以较贵模型为基准"),
    ] {
        let summary = form_with_coefficients(lower, "50").summary();
        assert_eq!(summary.winner, 1);
        assert_eq!(summary.detail, expected);
    }
}

#[test]
fn form_overflow_is_not_displayed_as_zero_and_does_not_corrupt_the_other_model() {
    for bad_model in 0..2 {
        let mut form = form_with_coefficients("3", "4");
        let other = 1 - bad_model;
        let other_before = form.models[other].clone();
        let other_text = form.coefficient_text(other);
        form.set(bad_model, 0, "1e308".into());
        form.set(bad_model, 1, "1e308".into());
        assert_eq!(
            form.models[bad_model].coefficient().unwrap_err(),
            "数值过大，无法计算"
        );
        assert_eq!(form.coefficient_text(bad_model), "—");
        assert_eq!(form.summary().winner, -1);
        assert_eq!(form.models[other], other_before);
        assert_eq!(form.coefficient_text(other), other_text);
        form.set(bad_model, 0, "".into());
        form.set(bad_model, 1, if bad_model == 0 { "3" } else { "4" }.into());
        assert_eq!(form.summary().winner, 1);
    }
}
