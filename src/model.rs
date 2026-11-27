//! 表单与展示状态。无 UI 依赖，便于自动测试。
use crate::calc::{compare, format_coefficient, parse_number, Prices, Verdict};

pub const FIELDS: [&str; 5] = ["输入", "输出", "缓存写入", "缓存读取", "平均缓存率"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelInput {
    pub values: [String; 5],
}

impl ModelInput {
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormState {
    pub models: [ModelInput; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub headline: String,
    pub detail: String,
    /// -1=输入无效，0=相同，1=模型1更便宜，2=模型2更便宜。
    pub winner: i32,
}

impl FormState {
    pub fn set(&mut self, model: usize, field: usize, text: String) {
        if let Some(value) = self
            .models
            .get_mut(model)
            .and_then(|m| m.values.get_mut(field))
        {
            *value = text;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn summary(&self) -> Summary {
        let (Ok(c1), Ok(c2)) = (self.models[0].coefficient(), self.models[1].coefficient()) else {
            return Summary {
                headline: "请修正标红的数值".into(),
                detail: "输入有效后会立即重新比较".into(),
                winner: -1,
            };
        };
        let (winner, cheap, dear) = match compare(c1, c2) {
            Verdict::Equal => {
                return Summary {
                    headline: "两个模型一样贵".into(),
                    detail: "价格系数相同".into(),
                    winner: 0,
                }
            }
            Verdict::Model1Cheaper => (1, c1, c2),
            Verdict::Model2Cheaper => (2, c2, c1),
        };
        let saving = (dear - cheap) / dear * 100.0;
        let saving_text = if saving < 0.05 {
            "不足 0.1%".into()
        } else {
            format!("约 {saving:.1}%")
        };
        Summary {
            headline: format!("模型{winner}更便宜"),
            detail: format!("便宜{saving_text} · 以较贵模型为基准"),
            winner,
        }
    }

    pub fn coefficient_text(&self, model: usize) -> String {
        self.models[model]
            .coefficient()
            .map(format_coefficient)
            .unwrap_or_else(|_| "—".into())
    }
}
