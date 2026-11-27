use relay_price_calc_m::calc::{
    compare, format_coefficient, is_equal, parse_number, rank, saving_percent, Prices, Verdict,
};
use relay_price_calc_m::model::{FormState, ModelInput, FIELDS, MODEL_COUNT};

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
        ..ModelInput::default()
    }
}

fn form_with_coefficients(first: &str, second: &str) -> FormState {
    form_with_four_coefficients([first, second, "", ""])
}

fn form_with_four_coefficients(values: [&str; MODEL_COUNT]) -> FormState {
    let mut form = FormState::default();
    for (index, value) in values.into_iter().enumerate() {
        form.set(index, 1, value.into());
    }
    form
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
            ModelInput::default(),
            ModelInput::default(),
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
fn default_form_has_four_distinct_names_and_no_participating_models() {
    let form = FormState::default();
    assert_eq!(MODEL_COUNT, 4);
    for (index, model) in form.models.iter().enumerate() {
        assert_eq!(model.name, format!("模型{}", index + 1));
        assert!(model.is_empty());
        assert_eq!(form.model_name(index), model.name);
        assert_eq!(form.coefficient_text(index), "—");
    }
    assert!(form.ranking().is_empty());
    assert_eq!(form.summary().winner, 0);
    assert_eq!(form.summary().headline, "至少填写两个模型才能比较");
}

#[test]
fn setting_each_field_recalculates_coefficients_and_winners_immediately() {
    let mut form = FormState::default();
    for (model, field, text, first, second, winner) in [
        (0, 0, "3", 3.0, 0.0, 0),
        (0, 1, "15", 18.0, 0.0, 0),
        (0, 2, "3.75", 18.0, 0.0, 0),
        (0, 3, "0.3", 18.0, 0.0, 0),
        (0, 4, "80", 6.84, 0.0, 0),
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
        assert_eq!(
            form.coefficient_text(1),
            if form.models[1].is_empty() {
                "—".into()
            } else {
                format_coefficient(second)
            }
        );
        assert_eq!(
            form.summary().winner,
            winner,
            "after setting {model}/{field} to {text}"
        );
    }
}

#[test]
fn all_four_models_can_be_edited_and_become_the_unique_winner() {
    for index in 0..MODEL_COUNT {
        let mut form = form_with_four_coefficients(["10", "10", "10", "10"]);
        form.set(index, 1, "3".into());
        assert_eq!(form.summary().winner, (index + 1) as i32);
        assert_eq!(form.ranking()[0].index, index);
        assert_eq!(form.coefficient_text(index), "3");
    }
}

#[test]
fn setting_unknown_indexes_leaves_the_form_unchanged() {
    let mut form = form_with_coefficients("3", "4");
    let before = form.clone();
    for (model, field) in [
        (MODEL_COUNT, 0),
        (0, 5),
        (usize::MAX, 0),
        (0, usize::MAX),
        (usize::MAX, usize::MAX),
    ] {
        form.set(model, field, "bad".into());
        form.clear_model(MODEL_COUNT);
        form.rename(usize::MAX, "bad".into());
        assert_eq!(form, before);
        assert_eq!(form.summary(), before.summary());
    }
    assert_eq!(form.coefficient_text(usize::MAX), "—");
}

#[test]
fn invalid_form_fields_only_exclude_the_affected_model() {
    for bad_model in 0..MODEL_COUNT {
        for field in 0..FIELDS.len() {
            let mut form = form_with_four_coefficients(["3", "4", "5", "6"]);
            let before = form.clone();
            form.set(bad_model, field, "bad".into());
            assert_eq!(form.models[bad_model].field_error(field), "请输入有效数字");
            assert_eq!(form.coefficient_text(bad_model), "—");
            assert_eq!(form.ranking().len(), MODEL_COUNT - 1);
            assert!(!form.ranking().iter().any(|entry| entry.index == bad_model));
            assert_eq!(form.summary().winner, if bad_model == 0 { 2 } else { 1 });
            assert!(form
                .summary()
                .detail
                .contains("有 1 个模型数值有误，未参与比较"));
            for other in 0..MODEL_COUNT {
                if other != bad_model {
                    assert_eq!(form.models[other], before.models[other]);
                    assert_eq!(form.coefficient_text(other), before.coefficient_text(other));
                }
            }
        }
    }
}

#[test]
fn fixing_invalid_input_restores_live_comparison_in_every_model() {
    for bad_model in 0..MODEL_COUNT {
        for (field, bad) in [(0, "bad"), (1, "-1"), (4, "101")] {
            let mut form = form_with_four_coefficients(["3", "4", "5", "6"]);
            let baseline = form.clone();
            form.set(bad_model, field, bad.into());
            assert_eq!(form.coefficient_text(bad_model), "—");
            form.set(
                bad_model,
                field,
                baseline.models[bad_model].values[field].clone(),
            );
            assert_eq!(form, baseline);
            assert_eq!(form.summary(), baseline.summary());
            assert_eq!(form.ranking(), baseline.ranking());
        }
    }
}

#[test]
fn clear_removes_all_values_and_errors_but_preserves_all_names() {
    for invalid in [false, true] {
        let mut form = form_with_four_coefficients(["3", "4", "5", "6"]);
        for index in 0..MODEL_COUNT {
            form.rename(index, format!("自定义{index}"));
        }
        if invalid {
            form.set(1, 4, "101".into());
        }
        let names = form.models.each_ref().map(|model| model.name.clone());
        for _ in 0..2 {
            form.clear();
            assert!(form.ranking().is_empty());
            assert_eq!(form.summary(), FormState::default().summary());
            assert_eq!(
                form.models.each_ref().map(|model| model.name.clone()),
                names
            );
            for index in 0..MODEL_COUNT {
                assert_eq!(form.coefficient_text(index), "—");
                assert!(form.models[index].values.iter().all(String::is_empty));
                for field in 0..FIELDS.len() {
                    assert_eq!(form.models[index].field_error(field), "");
                }
            }
        }
    }
}

#[test]
fn clear_model_preserves_its_name_and_every_other_model() {
    for index in 0..MODEL_COUNT {
        let mut form = form_with_four_coefficients(["3", "4", "5", "6"]);
        form.rename(index, "保留名字".into());
        form.set(index, 4, "bad".into());
        let before = form.clone();
        for _ in 0..2 {
            form.clear_model(index);
            assert_eq!(form.models[index].name, "保留名字");
            assert!(form.models[index].is_empty());
            assert_eq!(form.coefficient_text(index), "—");
            for other in 0..MODEL_COUNT {
                if other != index {
                    assert_eq!(form.models[other], before.models[other]);
                }
            }
        }
    }
}

#[test]
fn summary_uses_calculated_values_and_relative_epsilon_for_equality() {
    let mut mixed = FormState::default();
    mixed.models[0] = input(["2", "3", "", "", "0"]);
    mixed.models[1] = input(["", "", "2", "3", "100"]);
    for form in [
        form_with_coefficients("16.59", "16.59"),
        form_with_coefficients("1", "1.0000000000005"),
        form_with_coefficients("1e-200", "1.0000000000005e-200"),
        mixed,
    ] {
        let summary = form.summary();
        assert_eq!(summary.winner, 0);
        assert_eq!(summary.headline, "参与比较的模型一样贵");
        assert_eq!(summary.detail, "价格系数相同");
        assert!(form
            .ranking()
            .iter()
            .all(|entry| entry.is_best && entry.place == 1 && entry.saving_text == "—"));
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
        assert_eq!(summary.headline, format!("模型{winner} 最便宜"));
        assert_eq!(summary.detail, expected_detail);
    }
}

#[test]
fn summary_reports_one_hundred_percent_saving_for_zero_versus_positive() {
    for positive in ["3", "1e-15", "1e-300"] {
        for (first, second, winner) in [("0", positive, 1), (positive, "0", 2)] {
            let form = form_with_coefficients(first, second);
            assert_eq!(form.summary().winner, winner);
            assert_eq!(form.summary().detail, "便宜约 100.0% · 以较贵模型为基准");
            assert_eq!(form.ranking()[1].saving_text, "最便宜的比它便宜约 100.0%");
        }
    }
}

#[test]
fn summary_does_not_claim_zero_difference_when_displayed_coefficients_match() {
    for (lower, higher) in [("1", "1.00000000001"), ("1e-200", "1.0000001e-200")] {
        for (first, second, winner) in [(lower, higher, 1), (higher, lower, 2)] {
            let form = form_with_coefficients(first, second);
            assert_eq!(form.coefficient_text(0), form.coefficient_text(1));
            assert_eq!(form.summary().winner, winner);
            assert_eq!(form.summary().detail, "便宜不到0.1% · 以较贵模型为基准");
            assert_eq!(form.ranking()[1].saving_text, "最便宜的比它便宜不到0.1%");
            assert!(!form.summary().detail.contains("约 0.0%"));
        }
    }
}

#[test]
fn summary_rounding_distinguishes_tiny_savings_from_a_rounded_tenth_percent() {
    for (lower, expected) in [
        ("49.98", "便宜不到0.1% · 以较贵模型为基准"),
        ("49.97", "便宜约 0.1% · 以较贵模型为基准"),
    ] {
        let summary = form_with_coefficients(lower, "50").summary();
        assert_eq!(summary.winner, 1);
        assert_eq!(summary.detail, expected);
    }
}

#[test]
fn form_overflow_is_excluded_without_corrupting_other_models() {
    for bad_model in 0..MODEL_COUNT {
        let mut form = form_with_four_coefficients(["3", "4", "5", "6"]);
        let before = form.clone();
        form.set(bad_model, 0, "1e308".into());
        form.set(bad_model, 1, "1e308".into());
        assert_eq!(
            form.models[bad_model].coefficient().unwrap_err(),
            "数值过大，无法计算"
        );
        assert_eq!(form.coefficient_text(bad_model), "—");
        assert_eq!(form.ranking().len(), 3);
        assert!(form.summary().detail.contains("有 1 个模型数值有误"));
        for other in 0..MODEL_COUNT {
            if other != bad_model {
                assert_eq!(form.models[other], before.models[other]);
            }
        }
        form.models[bad_model] = before.models[bad_model].clone();
        assert_eq!(form.summary().winner, 1);
    }
}

#[test]
fn whitespace_only_values_are_skipped_even_with_a_custom_name() {
    let mut form = FormState::default();
    form.models[0] = input(["", " ", "\t", "　", "\n"]);
    form.rename(0, "只有名字".into());
    assert!(form.models[0].is_empty());
    assert_eq!(form.coefficient_text(0), "—");
    assert!(form.ranking().is_empty());
    assert_eq!(form.summary().winner, 0);
}

#[test]
fn explicit_zero_is_a_valid_participant_but_an_empty_model_is_not() {
    let form = form_with_four_coefficients(["", "0", "", "5"]);
    let ranked = form.ranking();
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].index, 1);
    assert_eq!(ranked[0].coefficient, 0.0);
    assert_eq!(form.summary().winner, 2);
    assert_eq!(form.coefficient_text(0), "—");
    assert_eq!(form.coefficient_text(1), "0");
}

#[test]
fn a_single_participant_cannot_be_declared_the_winner() {
    for value in ["0", "3", "1e-300"] {
        for index in 0..MODEL_COUNT {
            let mut form = FormState::default();
            form.set(index, 1, value.into());
            assert_eq!(form.ranking().len(), 1);
            assert_eq!(form.summary().winner, 0);
            assert_eq!(form.summary().headline, "至少填写两个模型才能比较");
            assert!(form.summary().detail.contains(&form.model_name(index)));
        }
    }
}

#[test]
fn insufficient_valid_models_and_errors_use_error_summary_and_counts() {
    for valid in ["", "3"] {
        let form = form_with_four_coefficients(["bad", "-1", valid, ""]);
        assert_eq!(form.summary().winner, -1);
        assert_eq!(form.summary().headline, "请先修正标红的数值");
        assert!(form
            .summary()
            .detail
            .contains("有 2 个模型数值有误，未参与比较"));
    }
    let form = form_with_four_coefficients(["bad", "-1", "NaN", "inf"]);
    assert!(form.ranking().is_empty());
    assert_eq!(form.summary().winner, -1);
    assert!(form
        .summary()
        .detail
        .contains("有 4 个模型数值有误，未参与比较"));
}

#[test]
fn two_valid_models_still_compare_when_the_other_two_are_invalid() {
    let form = form_with_four_coefficients(["bad", "-1", "3", "4"]);
    assert_eq!(form.summary().winner, 3);
    assert_eq!(form.summary().headline, "模型3 最便宜");
    assert!(form
        .summary()
        .detail
        .contains("有 2 个模型数值有误，未参与比较"));
    assert_eq!(
        form.ranking()
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<_>>(),
        [2, 3]
    );
}

#[test]
fn four_model_rankings_assign_competition_places_and_stable_ties() {
    for (values, indexes, places) in [
        (["3", "1", "1", "2"], [1, 2, 3, 0], [1, 1, 3, 4]),
        (["3", "2", "2", "1"], [3, 1, 2, 0], [1, 2, 2, 4]),
        (["4", "3", "2", "1"], [3, 2, 1, 0], [1, 2, 3, 4]),
    ] {
        let ranked = form_with_four_coefficients(values).ranking();
        assert_eq!(
            ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
            indexes
        );
        assert_eq!(
            ranked.iter().map(|entry| entry.place).collect::<Vec<_>>(),
            places
        );
        assert!(ranked
            .iter()
            .filter(|entry| entry.is_best)
            .all(|entry| entry.place == 1));
    }
}

#[test]
fn tied_best_models_use_both_names_and_no_unique_winner() {
    let mut form = form_with_four_coefficients(["3", "1", "1", "2"]);
    form.rename(1, "乙".into());
    form.rename(2, "丙".into());
    assert_eq!(form.summary().winner, 0);
    assert_eq!(form.summary().headline, "乙、丙 最便宜");
    let ranked = form.ranking();
    assert!(ranked[0].is_best && ranked[1].is_best);
    assert_eq!(ranked[2].saving_text, "最便宜的比它便宜约 50.0%");
    assert_eq!(ranked[3].saving_text, "最便宜的比它便宜约 66.7%");
}

#[test]
fn all_four_equal_models_have_no_saving_or_unique_winner() {
    let form = form_with_four_coefficients(["2", "2", "2", "2"]);
    assert_eq!(form.summary().winner, 0);
    assert_eq!(form.summary().headline, "参与比较的模型一样贵");
    assert_eq!(
        form.ranking()
            .iter()
            .map(|entry| entry.index)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert!(form
        .ranking()
        .iter()
        .all(|entry| entry.is_best && entry.place == 1 && entry.saving_text == "—"));
}

#[test]
fn near_tie_groups_restore_original_order_without_merging_epsilon_chains() {
    let form = form_with_four_coefficients(["1.00000000000075", "1", "1.0000000000015", "3"]);
    let ranked = form.ranking();
    assert_eq!(
        ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert!(ranked[0].is_best && ranked[1].is_best);
    assert!(!ranked[2].is_best);
    assert_eq!(
        ranked.iter().map(|entry| entry.place).collect::<Vec<_>>(),
        [1, 1, 2, 4]
    );
    assert_eq!(form.summary().winner, 0);
    assert_eq!(form.summary().headline, "模型1、模型2 最便宜");
    assert_eq!(ranked[2].saving_text, "最便宜的比它便宜不到0.1%");
}

#[test]
fn later_near_tie_groups_also_use_their_own_minimum_and_restore_order() {
    let ranked = rank(&[
        Some(2.0000000000015),
        Some(1.0),
        Some(2.0),
        Some(2.000000000003),
    ]);
    assert_eq!(
        ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
        [1, 0, 2, 3]
    );
    assert!(!is_equal(ranked[2].coefficient, ranked[3].coefficient));
}

#[test]
fn rank_skips_missing_values_and_handles_empty_input() {
    assert!(rank(&[]).is_empty());
    assert!(rank(&[None, None]).is_empty());
    let ranked = rank(&[Some(3.0), None, Some(1.0), Some(2.0)]);
    assert_eq!(
        ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
        [2, 3, 0]
    );
    assert_eq!(
        ranked
            .iter()
            .map(|entry| entry.coefficient)
            .collect::<Vec<_>>(),
        [1.0, 2.0, 3.0]
    );
}

#[test]
fn approximate_and_exact_ties_preserve_source_order_at_multiple_scales() {
    for scale in [1e-200, 1.0, 1e200] {
        let ranked = rank(&[Some(scale * (1.0 + 5e-13)), Some(scale), Some(scale), None]);
        assert_eq!(
            ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
    }
}

#[test]
fn equality_helper_matches_compare_and_saving_handles_zero_baselines() {
    for (a, b) in [(0.3, 0.1 + 0.2), (0.0, 0.0), (0.0, 1e-300), (1.0, 2.0)] {
        assert_eq!(is_equal(a, b), compare(a, b) == Verdict::Equal);
    }
    assert_eq!(saving_percent(1.0, 4.0), 75.0);
    assert_eq!(saving_percent(0.0, 1.0), 100.0);
    assert_eq!(saving_percent(1.0, 1.0), 0.0);
    assert_eq!(saving_percent(0.0, 0.0), 0.0);
    assert_eq!(saving_percent(1.0, -1.0), 0.0);
    assert_eq!(saving_percent(2.0, 1.0), -100.0);
}

#[test]
fn ranking_savings_use_true_minimum_even_if_the_first_entry_is_nearly_tied() {
    let form = form_with_four_coefficients(["1.00000000000075", "1", "2", ""]);
    let ranked = form.ranking();
    assert_eq!(ranked[0].index, 0);
    assert!(ranked[0].coefficient > ranked[1].coefficient);
    assert_eq!(ranked[2].saving_text, "最便宜的比它便宜约 50.0%");
    assert_eq!(form.summary().detail, "便宜约 50.0% · 以较贵模型为基准");
}

#[test]
fn custom_names_are_displayed_trimmed_with_position_specific_empty_fallbacks() {
    let mut form = form_with_four_coefficients(["4", "3", "2", "1"]);
    for index in 0..MODEL_COUNT {
        form.rename(index, "　 \t".into());
        assert_eq!(form.model_name(index), format!("模型{}", index + 1));
    }
    form.rename(3, "  自定义最便宜  ".into());
    assert_eq!(form.models[3].name, "  自定义最便宜  ");
    assert_eq!(form.model_name(3), "自定义最便宜");
    assert_eq!(form.summary().headline, "自定义最便宜 最便宜");
    assert_eq!(form.summary().winner, 4);
}

#[test]
fn rename_limits_unicode_characters_without_changing_values_or_other_models() {
    let mut form = form_with_four_coefficients(["4", "3", "2", "1"]);
    let before = form.clone();
    form.rename(2, "😀中".repeat(20));
    assert_eq!(form.models[2].name, "😀中".repeat(15));
    assert_eq!(form.models[2].name.chars().count(), 30);
    assert_eq!(form.models[2].values, before.models[2].values);
    for other in [0, 1, 3] {
        assert_eq!(form.models[other], before.models[other]);
    }
    form.models[2].name = "长".repeat(50);
    assert_eq!(form.model_name(2).chars().count(), 30);
}

#[test]
fn duplicate_names_do_not_merge_models_or_change_winner_indexes() {
    let mut form = form_with_four_coefficients(["4", "3", "2", "1"]);
    for index in 0..MODEL_COUNT {
        form.rename(index, "同名".into());
    }
    assert_eq!(form.summary().winner, 4);
    assert_eq!(form.ranking().len(), 4);
}

#[test]
fn serialization_roundtrip_preserves_names_raw_text_and_invalid_values() {
    let mut form = form_with_four_coefficients([" ８０％ ", "invalid", "1e-3", "0.30"]);
    form.rename(2, "自定义😀".into());
    form.models[0].values[0] = " \t ".into();
    let json = serde_json::to_string(&form).unwrap();
    assert_eq!(serde_json::from_str::<FormState>(&json).unwrap(), form);
    let model = &form.models[0];
    assert_eq!(
        serde_json::from_str::<ModelInput>(&serde_json::to_string(model).unwrap()).unwrap(),
        *model
    );
}

#[test]
fn deserialization_missing_fields_uses_position_specific_defaults() {
    assert_eq!(
        serde_json::from_str::<FormState>("{}").unwrap(),
        FormState::default()
    );
    assert_eq!(
        serde_json::from_str::<ModelInput>("{}").unwrap(),
        ModelInput::default()
    );
    let form: FormState = serde_json::from_str(r#"{"models":[{}, {"values":["7"]}]}"#).unwrap();
    assert_eq!(form.models[0].name, "模型1");
    assert_eq!(form.models[1].name, "模型2");
    assert_eq!(
        form.models[1].values,
        ["7", "", "", "", ""].map(String::from)
    );
    assert_eq!(form.models[2], FormState::default().models[2]);
    assert_eq!(form.models[3], FormState::default().models[3]);
}

#[test]
fn deserialization_pads_short_arrays_and_truncates_long_arrays() {
    let form: FormState = serde_json::from_str(r#"{"models":[{"name":"A","values":["1","2","3","4","5","6"]},{},{},{},{"name":"ignored"}]}"#).unwrap();
    assert_eq!(form.models[0].name, "A");
    assert_eq!(
        form.models[0].values,
        ["1", "2", "3", "4", "5"].map(String::from)
    );
    assert_eq!(form.models[3].name, "模型4");
    assert_eq!(
        serde_json::from_str::<FormState>(r#"{"models":[]}"#).unwrap(),
        FormState::default()
    );
    let model: ModelInput = serde_json::from_str(r#"{"values":["a","b"]}"#).unwrap();
    assert_eq!(model.values, ["a", "b", "", "", ""].map(String::from));
}

#[test]
fn valid_equal_models_keep_equal_summary_even_if_other_models_are_invalid() {
    let form = form_with_four_coefficients(["bad", "2", "2", "NaN"]);
    assert_eq!(form.summary().winner, 0);
    assert_eq!(form.summary().headline, "参与比较的模型一样贵");
    assert!(form
        .summary()
        .detail
        .contains("有 2 个模型数值有误，未参与比较"));
    assert_eq!(form.ranking().len(), 2);
    assert!(form.ranking().iter().all(|entry| entry.is_best));
}

#[test]
fn ranking_does_not_treat_tiny_positive_values_as_zero_or_equal() {
    let form = form_with_four_coefficients(["2e-300", "0", "1e-300", ""]);
    let ranked = form.ranking();
    assert_eq!(
        ranked.iter().map(|entry| entry.index).collect::<Vec<_>>(),
        [1, 2, 0]
    );
    assert_eq!(
        ranked.iter().map(|entry| entry.place).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(form.summary().winner, 2);
    assert_eq!(form.coefficient_text(0), "2.000e-300");
    assert_eq!(form.coefficient_text(1), "0");
    assert_eq!(ranked[1].saving_text, "最便宜的比它便宜约 100.0%");
}

#[test]
fn deserialization_preserves_explicit_empty_names_and_rejects_wrong_types() {
    let form: FormState =
        serde_json::from_str(r#"{"models":[{"name":""},{"name":"  "}]}"#).unwrap();
    assert_eq!(form.models[0].name, "");
    assert_eq!(form.models[1].name, "  ");
    assert_eq!(form.model_name(0), "模型1");
    assert_eq!(form.model_name(1), "模型2");
    for invalid in [
        r#"{"models":false}"#,
        r#"{"models":[{"values":false}]}"#,
        r#"{"models":[{"values":[3]}]}"#,
        r#"{"models":[{"name":3}]}"#,
    ] {
        assert!(
            serde_json::from_str::<FormState>(invalid).is_err(),
            "{invalid}"
        );
    }
}
