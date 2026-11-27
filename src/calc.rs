//! 计价规则来自 A-2788553793/relay-price-calc（MIT）。
//! 仅表示原项目定义的价格系数，不假定实际请求的输入输出 token 数相等。

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prices {
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
    /// 百分比，80 代表 80%。
    pub cache_rate_percent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Model1Cheaper,
    Model2Cheaper,
    Equal,
}

const RELATIVE_EPSILON: f64 = 1e-12;

impl Prices {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("输入", self.input),
            ("输出", self.output),
            ("缓存写入", self.cache_write),
            ("缓存读取", self.cache_read),
            ("平均缓存率", self.cache_rate_percent),
        ] {
            if !value.is_finite() {
                return Err(format!("“{name}”不是有效数字"));
            }
            if value < 0.0 {
                return Err(format!("“{name}”不能是负数"));
            }
        }
        if self.cache_rate_percent > 100.0 {
            return Err("“平均缓存率”不能超过 100%".into());
        }
        Ok(())
    }

    /// 与上游同步：非缓存价格之和乘未命中率，缓存价格之和乘命中率。
    pub fn coefficient(&self) -> Result<f64, String> {
        self.validate()?;
        let hit = self.cache_rate_percent / 100.0;
        let miss = 1.0 - hit;
        let value = (self.input + self.output) * miss + (self.cache_read + self.cache_write) * hit;
        if !value.is_finite() {
            return Err("数值过大，无法计算".into());
        }
        Ok(value)
    }
}

pub fn compare(c1: f64, c2: f64) -> Verdict {
    let scale = c1.abs().max(c2.abs());
    if (c1 - c2).abs() <= scale * RELATIVE_EPSILON {
        Verdict::Equal
    } else if c1 < c2 {
        Verdict::Model1Cheaper
    } else {
        Verdict::Model2Cheaper
    }
}

/// 与原项目一致：空白按 0；支持全角数字、句号和末尾的百分号。
pub fn parse_number(text: &str) -> Result<f64, String> {
    let normalized: String = text
        .trim()
        .chars()
        .map(|c| match c {
            '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap(),
            '．' | '。' => '.',
            '％' => '%',
            _ => c,
        })
        .collect();
    let s = normalized.trim_end_matches('%').trim();
    if s.is_empty() {
        return Ok(0.0);
    }
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("“{}”不是有效数字", text.trim()))
}

pub fn format_coefficient(value: f64) -> String {
    if value != 0.0 && value.abs() < 1e-4 {
        return format!("{value:.3e}");
    }
    let text = format!("{value:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".into()
    } else {
        text.into()
    }
}
