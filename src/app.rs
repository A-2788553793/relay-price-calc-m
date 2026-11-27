use crate::{
    calc::format_coefficient,
    model::{FormState, FIELDS, MODEL_COUNT},
    storage::{AppConfig, ConfigStore, MAX_TEXT_SIZE, MIN_TEXT_SIZE},
};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::include_modules!();
pub const PROJECT_URL: &str = "https://github.com/A-2788553793/relay-price-calc-m";

struct UiModels {
    cards: Rc<VecModel<ModelCard>>,
    fields: [Rc<VecModel<PriceField>>; MODEL_COUNT],
    ranking: Rc<VecModel<RankingRow>>,
}

impl UiModels {
    fn new() -> Self {
        let fields = std::array::from_fn(|_| {
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
        });
        let cards = Rc::new(VecModel::from(
            fields
                .iter()
                .map(|fields| ModelCard {
                    fields: ModelRc::from(fields.clone()),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        ));
        Self {
            cards,
            fields,
            ranking: Rc::new(VecModel::default()),
        }
    }
}

fn refresh(window: &MainWindow, state: &FormState, models: &UiModels) {
    let ranking = state.ranking();
    for (model_index, model) in models.fields.iter().enumerate() {
        for field_index in 0..FIELDS.len() {
            let mut row = model.row_data(field_index).expect("固定五个字段");
            let value = state.models[model_index].values[field_index].as_str();
            let error = state.models[model_index].field_error(field_index);
            if row.value.as_str() != value || row.error.as_str() != error {
                row.value = value.into();
                row.error = error.into();
                model.set_row_data(field_index, row);
            }
        }
        let mut card = models.cards.row_data(model_index).expect("固定四个模型");
        // Preserve nested model identity and avoid resetting focused editors on every keystroke.
        let name = state.models[model_index].name.as_str();
        let coefficient = state.coefficient_text(model_index);
        let error = state.models[model_index]
            .coefficient()
            .err()
            .unwrap_or_default();
        let best = ranking
            .iter()
            .any(|row| row.index == model_index && row.is_best)
            && ranking.len() >= 2;
        if card.name.as_str() != name
            || card.coefficient.as_str() != coefficient
            || card.error.as_str() != error
            || card.best != best
        {
            card.name = name.into();
            card.coefficient = coefficient.into();
            card.error = error.into();
            card.best = best;
            models.cards.set_row_data(model_index, card);
        }
    }
    let rows = ranking
        .iter()
        .map(|row| RankingRow {
            name: state.model_name(row.index).into(),
            place: row.place.to_string().into(),
            coefficient: format_coefficient(row.coefficient).into(),
            saving: row.saving_text.as_str().into(),
            best: row.is_best,
        })
        .collect::<Vec<_>>();
    models.ranking.set_vec(rows);
    let summary = state.summary();
    window.set_headline(summary.headline.into());
    window.set_detail(summary.detail.into());
    window.set_winner(summary.winner);
}

fn persist(window: &MainWindow, config: &AppConfig, store: &ConfigStore) {
    match store.save(config) {
        Ok(()) if store.is_persistent() => {
            window.set_storage_status("已自动保存到本机；卸载将清除数据".into())
        }
        Ok(()) => {}
        Err(error) => window.set_storage_status(error.into()),
    }
}

/// 测试和桌面预览专用；不会读写真实配置。
pub fn create_window() -> Result<MainWindow, slint::PlatformError> {
    create_window_with_store(ConfigStore::memory())
}

/// 可注入私有目录/临时目录，便于验证从编辑到重开恢复的完整路径。
pub fn create_window_with_store(
    mut store: ConfigStore,
) -> Result<MainWindow, slint::PlatformError> {
    let loaded = store.load();
    let window = MainWindow::new()?;
    window.set_dark_mode(loaded.config.dark_mode);
    window.set_text_size(loaded.config.text_size);
    window.set_storage_status(loaded.notice.into());
    let state = Rc::new(RefCell::new(loaded.config));
    let store = Rc::new(store);
    let models = Rc::new(UiModels::new());
    window.set_models(ModelRc::from(models.cards.clone()));
    window.set_ranking(ModelRc::from(models.ranking.clone()));
    refresh(&window, &state.borrow().form, &models);

    let weak = window.as_weak();
    let edit_state = state.clone();
    let edit_models = models.clone();
    let edit_store = store.clone();
    window.on_field_edited(move |model, field, text| {
        if model < 0 || model as usize >= MODEL_COUNT || field < 0 || field as usize >= FIELDS.len()
        {
            return;
        }
        let mut config = edit_state.borrow_mut();
        config.form.set(model as usize, field as usize, text.into());
        if let Some(window) = weak.upgrade() {
            refresh(&window, &config.form, &edit_models);
            persist(&window, &config, &edit_store);
        }
    });
    let weak = window.as_weak();
    let edit_state = state.clone();
    let edit_models = models.clone();
    let edit_store = store.clone();
    window.on_name_edited(move |model, text| {
        if model < 0 || model as usize >= MODEL_COUNT {
            return;
        }
        let mut config = edit_state.borrow_mut();
        config.form.rename(model as usize, text.into());
        if let Some(window) = weak.upgrade() {
            refresh(&window, &config.form, &edit_models);
            persist(&window, &config, &edit_store);
        }
    });
    let weak = window.as_weak();
    let edit_state = state.clone();
    let edit_models = models.clone();
    let edit_store = store.clone();
    window.on_clear_model_requested(move |model| {
        if model < 0 || model as usize >= MODEL_COUNT {
            return;
        }
        let mut config = edit_state.borrow_mut();
        config.form.clear_model(model as usize);
        if let Some(window) = weak.upgrade() {
            refresh(&window, &config.form, &edit_models);
            persist(&window, &config, &edit_store);
        }
    });
    let weak = window.as_weak();
    let edit_state = state.clone();
    let edit_models = models.clone();
    let edit_store = store.clone();
    window.on_clear_requested(move || {
        let mut config = edit_state.borrow_mut();
        config.form.clear();
        if let Some(window) = weak.upgrade() {
            refresh(&window, &config.form, &edit_models);
            persist(&window, &config, &edit_store);
        }
    });
    let weak = window.as_weak();
    window.on_settings_edited(move |dark, size| {
        let mut config = state.borrow_mut();
        config.dark_mode = dark;
        config.text_size = size.clamp(MIN_TEXT_SIZE, MAX_TEXT_SIZE);
        if let Some(window) = weak.upgrade() {
            window.set_dark_mode(config.dark_mode);
            window.set_text_size(config.text_size);
            persist(&window, &config, &store);
        }
    });
    window.on_open_project_requested(|| {});
    Ok(window)
}

pub fn run() -> Result<(), slint::PlatformError> {
    create_window()?.run()
}

#[cfg(target_os = "android")]
pub fn run_android(app: slint::android::AndroidApp) -> Result<(), slint::PlatformError> {
    let store = match app.internal_data_path() {
        Some(path) if path.is_absolute() => ConfigStore::in_directory(path),
        _ => ConfigStore::unavailable("无法取得应用私有目录；自动保存不可用"),
    };
    let window = create_window_with_store(store)?;
    let weak = window.as_weak();
    window.on_open_project_requested(move || {
        if let Err(error) = crate::android::open_project(&app) {
            if let Some(window) = weak.upgrade() {
                window.set_storage_status(error.into());
            }
        }
    });
    window.run()
}
