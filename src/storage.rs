//! 持久化存储：候选人CSV + 数据JSON

use crate::models::{Candidate, SaveData};
use std::fs;
use std::path::PathBuf;

/// 获取存储根目录（exe同级 /data）
pub fn data_dir() -> PathBuf {
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    base.join("data")
}

/// 候选人 CSV 路径
pub fn csv_path() -> PathBuf {
    data_dir().join("candidates.csv")
}

/// 投票数据 JSON 路径
pub fn json_path() -> PathBuf {
    data_dir().join("votes.json")
}

/// 默认候选人名单
const DEFAULT_NAMES: &[&str] = &[
    "马蓉", "马麟", "马万强", "马国全", "王成举",
    "牛慧杰", "祁鹏", "李婧若", "唐龙", "潘富强",
];

/// 读取候选人列表（CSV），若不存在则创建默认文件
pub fn load_candidates() -> Vec<Candidate> {
    let path = csv_path();
    if !path.exists() {
        return create_default_csv();
    }
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return create_default_csv(),
    };
    let mut candidates = Vec::new();
    let mut id = 1u32;
    for line in content.lines() {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }
        candidates.push(Candidate::new(id, name));
        id += 1;
    }
    if candidates.is_empty() {
        return create_default_csv();
    }
    candidates
}

/// 写回候选人列表到 CSV
pub fn save_candidates(candidates: &[Candidate]) {
    let _ = fs::create_dir_all(data_dir());
    let content: String = candidates
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(csv_path(), content);
}

/// 创建默认 CSV 并返回相应候选人列表
fn create_default_csv() -> Vec<Candidate> {
    let _ = fs::create_dir_all(data_dir());
    let content = DEFAULT_NAMES.join("\n");
    let _ = fs::write(csv_path(), &content);
    DEFAULT_NAMES
        .iter()
        .enumerate()
        .map(|(i, &n)| Candidate::new((i + 1) as u32, n))
        .collect()
}

/// 加载完整保存数据（候选人 + 投票 + 轮次）
pub fn load_save_data() -> SaveData {
    let path = json_path();
    if let Ok(json) = fs::read_to_string(&path) {
        if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
            return data;
        }
    }
    SaveData::new(load_candidates())
}

/// 保存完整数据
pub fn save_save_data(data: &SaveData) {
    let _ = fs::create_dir_all(data_dir());
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(json_path(), json);
    }
}

/// 导出最终结果到 txt
/// 格式：居中标题「柳树乡党委委员候选人」
///       表头：序号 姓名 同意 反对 弃权 总票数(同意)
pub fn export_results(data: &SaveData) -> Result<PathBuf, std::io::Error> {
    let _ = fs::create_dir_all(data_dir());
    let path = data_dir().join("投票结果.txt");

    let mut lines: Vec<String> = Vec::new();

    // 居中标题（基于 80 字符宽度居中）
    lines.push(center_text("柳树乡党委委员候选人", 80));
    lines.push(center_text(
        &format!("公开投票选举结果  第 {} 轮", data.round),
        80,
    ));
    lines.push(center_text(&format!("导出时间：{}", crate::util::timestamp()), 80));
    lines.push("================================================================================".to_string());

    // 表头
    lines.push(format!(
        "{:<4} {:<8} {:<14} {:<14} {:<14} {:<10}",
        "序号", "姓名", "同意", "反对", "弃权", "总票数(同意)"
    ));
    lines.push("--------------------------------------------------------------------------------".to_string());

    // 数据行
    for (i, c) in data.candidates.iter().enumerate() {
        lines.push(format!(
            "{:<4} {:<8} {:<14} {:<14} {:<14} {:<10}",
            i + 1,
            c.name,
            crate::util::tally_text(c.approve),
            crate::util::tally_text(c.oppose),
            crate::util::tally_text(c.abstain),
            c.approve,
        ));
    }

    lines.push("================================================================================".to_string());

    let _ = fs::write(&path, lines.join("\n"))?;
    Ok(path)
}

/// 将文本居中到指定宽度
fn center_text(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let pad = (width - len) / 2;
    format!("{}{}", " ".repeat(pad), s)
}
