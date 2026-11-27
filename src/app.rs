use crate::model::{FormState, FIELDS};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::include_modules!();

fn field_model() -> Rc<VecModel<PriceField>> {
    Rc::new(VecModel::from(
        FIELDS
            .iter()
            .enumerate()
            .map(|(index, label)| PriceField {
                label: (*label).into(),
                value: "".into(),
                error: "".into(),
                unit: if index == 4 { "%".into() } else { "".into() },
            })
            .collect::<Vec<_>>(),
    ))
}

fn refresh(window: &MainWindow, state: &FormState, models: &[Rc<VecModel<PriceField>>; 2]) {
    for (model_index, model) in models.iter().enumerate() {
        for field_index in 0..5 {
            let mut row = model.row_data(field_index).expect("固定五个字段");
            let value = state.models[model_index].values[field_index].as_str();
            let error = state.models[model_index].field_error(field_index);
            if row.value.as_str() != value || row.error.as_str() != error {
                row.value = value.into();
                row.error = error.into();
                model.set_row_data(field_index, row);
            }
        }
    }
    window.set_coefficient_one(state.coefficient_text(0).into());
    window.set_coefficient_two(state.coefficient_text(1).into());
    window.set_error_one(
        state.models[0]
            .coefficient()
            .err()
            .unwrap_or_default()
            .into(),
    );
    window.set_error_two(
        state.models[1]
            .coefficient()
            .err()
            .unwrap_or_default()
            .into(),
    );
    let summary = state.summary();
    window.set_headline(summary.headline.into());
    window.set_detail(summary.detail.into());
    window.set_winner(summary.winner);
}

/// 构建窗口和回调；用于 Android、桌面预览和无屏自动测试。
pub fn create_window() -> Result<MainWindow, slint::PlatformError> {
    let window = MainWindow::new()?;
    let models = [field_model(), field_model()];
    window.set_model_one(ModelRc::from(models[0].clone()));
    window.set_model_two(ModelRc::from(models[1].clone()));
    let state = Rc::new(RefCell::new(FormState::default()));
    refresh(&window, &state.borrow(), &models);

    let weak = window.as_weak();
    let edit_state = state.clone();
    let edit_models = models.clone();
    window.on_field_edited(move |model, field, text| {
        if model < 0 || field < 0 {
            return;
        }
        edit_state
            .borrow_mut()
            .set(model as usize, field as usize, text.into());
        if let Some(window) = weak.upgrade() {
            refresh(&window, &edit_state.borrow(), &edit_models);
        }
    });

    let weak = window.as_weak();
    window.on_clear_requested(move || {
        state.borrow_mut().clear();
        if let Some(window) = weak.upgrade() {
            refresh(&window, &state.borrow(), &models);
        }
    });
    Ok(window)
}

pub fn run() -> Result<(), slint::PlatformError> {
    create_window()?.run()
}
