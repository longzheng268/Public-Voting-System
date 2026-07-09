//! 工具函数：画正字调度、时间戳

/// 画正字节点：描述如何把 N 票渲染为若干张图片
#[derive(Clone, Debug)]
pub struct TallyNode {
    /// 完整"正"字图片的数量（每张 = 5票）
    pub full: u32,
    /// 余数（0..=4），表示还需要显示哪张余图（1.png .. 4.png）
    /// 0 表示没有余数
    pub remainder: u32,
}

impl TallyNode {
    pub fn new(count: u32) -> Self {
        Self {
            full: count / 5,
            remainder: count % 5,
        }
    }

    /// 是否无任何票
    pub fn is_zero(&self) -> bool {
        self.full == 0 && self.remainder == 0
    }

    /// 总票数
    pub fn total(&self) -> u32 {
        self.full * 5 + self.remainder
    }
}

/// 简洁数字（导出用）
pub fn tally_number(count: u32) -> String {
    count.to_string()
}

/// 导出用：画正字文字形式（正字 + 余数笔画 + 数字标注）
/// 例如：12票 → "正正（12）"，7票 → "正〇七" 或 "正"+数字
/// 正式场合采用：完整正字 + 括注数字
pub fn tally_text(count: u32) -> String {
    if count == 0 {
        return "0".to_string();
    }
    let full = (count / 5) as usize;
    let remainder = (count % 5) as usize;
    let mut s = String::new();
    for _ in 0..full {
        s.push('正');
    }
    if remainder > 0 {
        // 余数的笔画片段用正字逐笔：一 十 干 王
        let strokes = ['一', '十', '干', '王'];
        s.push(strokes[remainder - 1]);
    }
    s.push_str(&format!("({})", count));
    s
}

/// 生成北京时间时间戳
pub fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs = secs + 8 * 3600; // UTC+8
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    let hour = (secs % 86400) / 3600;
    let min = (secs % 3600) / 60;
    let sec = secs % 60;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, min, sec
    )
}
