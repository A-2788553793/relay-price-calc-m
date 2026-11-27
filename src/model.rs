//! 表单与展示状态。无 UI 依赖，便于自动测试。
use crate::calc::{format_coefficient, is_equal, parse_number, rank, saving_percent, Prices};
use serde::{Deserialize, Deserializer, Serialize};

pub const MODEL_COUNT: usize = 2;
pub const FIELDS: [&str; 5] = ["输入", "输出", "缓存写入", "缓存读取", "平均缓存率"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelInput {
    pub name: String,
    #[serde(deserialize_with = "deserialize_values")]
    pub values: [String; 5],
}

fn deserialize_values<'de, D>(deserializer: D) -> Result<[String; 5], D::Error>
where
    D: Deserializer<'de>,
{
    let mut values = Vec::<String>::deserialize(deserializer)?.into_iter();
    Ok(std::array::from_fn(|_| values.next().unwrap_or_default()))
}

fn deserialize_models<'de, D>(deserializer: D) -> Result<[ModelInput; MODEL_COUNT], D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Default, Deserialize)]
    #[serde(default)]
    struct ModelFields {
        name: Option<String>,
        #[serde(deserialize_with = "deserialize_values")]
        values: [String; 5],
    }

    let mut models = Vec::<ModelFields>::deserialize(deserializer)?.into_iter();
    Ok(std::array::from_fn(|index| match models.next() {
        Some(model) => ModelInput {
            name: model.name.unwrap_or_else(|| format!("模型{}", index + 1)),
            values: model.values,
        },
        None => ModelInput {
            name: format!("模型{}", index + 1),
            ..ModelInput::default()
        },
    }))
}

impl ModelInput {
    pub fn is_empty(&self) -> bool {
        self.values.iter().all(|value| value.trim().is_empty())
    }

    pub fn field_error(&self, index: usize) -> String {
        let Some(text) = self.values.get(index) else {
            return "未知字段".into();
        };
        match parse_number(text) {
            Err(_) => "请输入有效数字".into(),
            Ok(value) if value < 0.0 => "不能是负数".into(),
            Ok(value) if index == 4 && value > 100.0 => "不能超过 100%".into(),
            Ok(_) => String::new(),
        }
    }

    pub fn coefficient(&self) -> Result<f64, String> {
        let mut numbers = [0.0; 5];
        for (index, text) in self.values.iter().enumerate() {
            let error = self.field_error(index);
            if !error.is_empty() {
                return Err(format!("{}：{error}", FIELDS[index]));
            }
            numbers[index] = parse_number(text)?;
        }
        Prices {
            input: numbers[0],
            output: numbers[1],
            cache_write: numbers[2],
            cache_read: numbers[3],
            cache_rate_percent: numbers[4],
        }
        .coefficient()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FormState {
    #[serde(deserialize_with = "deserialize_models")]
    pub models: [ModelInput; MODEL_COUNT],
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            models: std::array::from_fn(|index| ModelInput {
                name: format!("模型{}", index + 1),
                ..ModelInput::default()
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub headline: String,
    pub detail: String,
    /// -1=有效模型不足且有错误，0=未填/单条/并列，1..=2=唯一最便宜的模型。
    pub winner: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankingEntry {
    pub index: usize,
    pub place: usize,
    pub coefficient: f64,
    pub is_best: bool,
    pub saving_text: String,
}

/// 保留一位小数，但绝不把真实价差表述为约 0.0%。
fn saving_text(saving: f64) -> String {
    if saving < 0.05 {
        "不到0.1%".into()
    } else {
        format!("约 {saving:.1}%")
    }
}

impl FormState {
    pub fn set(&mut self, model: usize, field: usize, text: String) {
        if let Some(value) = self
            .models
            .get_mut(model)
            .and_then(|model| model.values.get_mut(field))
        {
            *value = text;
        }
    }

    pub fn clear(&mut self) {
        for model in &mut self.models {
            model.values = Default::default();
        }
    }

    pub fn clear_model(&mut self, index: usize) {
        if let Some(model) = self.models.get_mut(index) {
            model.values = Default::default();
        }
    }

    pub fn rename(&mut self, index: usize, text: String) {
        if let Some(model) = self.models.get_mut(index) {
            model.name = text.chars().take(30).collect();
        }
    }

    pub fn model_name(&self, index: usize) -> String {
        let name = self.models.get(index).map_or("", |model| model.name.trim());
        if name.is_empty() {
            format!("模型{}", index.saturating_add(1))
        } else {
            name.chars().take(30).collect()
        }
    }

    pub fn coefficient_text(&self, index: usize) -> String {
        self.models
            .get(index)
            .filter(|model| !model.is_empty())
            .and_then(|model| model.coefficient().ok())
            .map(format_coefficient)
            .unwrap_or_else(|| "—".into())
    }

    pub fn ranking(&self) -> Vec<RankingEntry> {
        let coefficients = self.models.each_ref().map(|model| {
            if model.is_empty() {
                None
            } else {
                model.coefficient().ok()
            }
        });
        let ranked = rank(&coefficients);
        let Some(best) = ranked
            .iter()
            .map(|entry| entry.coefficient)
            .reduce(f64::min)
        else {
            return Vec::new();
        };
        ranked
            .iter()
            .map(|entry| {
                let is_best = is_equal(entry.coefficient, best);
                let place = if is_best {
                    1
                } else {
                    1 + ranked
                        .iter()
                        .filter(|other| {
                            other.coefficient < entry.coefficient
                                && !is_equal(other.coefficient, entry.coefficient)
                        })
                        .count()
                };
                RankingEntry {
                    index: entry.index,
                    place,
                    coefficient: entry.coefficient,
                    is_best,
                    saving_text: if is_best {
                        "—".into()
                    } else {
                        format!(
                            "最便宜的比它便宜{}",
                            saving_text(saving_percent(best, entry.coefficient))
                        )
                    },
                }
            })
            .collect()
    }

    pub fn summary(&self) -> Summary {
        let ranked = self.ranking();
        let errors = self
            .models
            .iter()
            .filter(|model| !model.is_empty() && model.coefficient().is_err())
            .count();
        let mut summary = if ranked.len() < 2 {
            Summary {
                headline: if errors > 0 {
                    "请先修正标红的数值".into()
                } else {
                    "至少填写两个模型才能比较".into()
                },
                detail: ranked.first().map_or_else(
                    || "填写数值后实时计算，空白字段按 0".into(),
                    |entry| {
                        format!(
                            "已填写{}，再填写一个模型即可比较",
                            self.model_name(entry.index)
                        )
                    },
                ),
                winner: if errors > 0 { -1 } else { 0 },
            }
        } else {
            let cheapest: Vec<_> = ranked.iter().filter(|entry| entry.is_best).collect();
            if cheapest.len() == ranked.len() {
                Summary {
                    headline: "参与比较的模型一样贵".into(),
                    detail: "价格系数相同".into(),
                    winner: 0,
                }
            } else {
                let names: Vec<_> = cheapest
                    .iter()
                    .map(|entry| self.model_name(entry.index))
                    .collect();
                let best = ranked
                    .iter()
                    .map(|entry| entry.coefficient)
                    .reduce(f64::min)
                    .unwrap();
                let dear = ranked
                    .iter()
                    .map(|entry| entry.coefficient)
                    .reduce(f64::max)
                    .unwrap();
                Summary {
                    headline: format!("{} 最便宜", names.join("、")),
                    detail: format!(
                        "便宜{} · 以较贵模型为基准",
                        saving_text(saving_percent(best, dear))
                    ),
                    winner: if cheapest.len() == 1 {
                        (cheapest[0].index + 1) as i32
                    } else {
                        0
                    },
                }
            }
        };
        if errors > 0 {
            summary
                .detail
                .push_str(&format!(" · 有 {errors} 个模型数值有误，未参与比较"));
        }
        summary
    }
}
