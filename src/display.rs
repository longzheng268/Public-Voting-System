//! 软件界面显示内容配置（从 data/display.toml 读取）

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DisplayConfig {
    pub title: String,
    pub col_no: String,
    pub col_name: String,
    pub col_approve: String,
    pub col_oppose: String,
    pub col_abstain: String,
    pub col_total_approve: String,
    pub btn_manage: String,
    pub btn_export: String,
    pub btn_reset: String,
    pub btn_submit: String,
    pub btn_back: String,
    pub btn_add: String,
    pub btn_fullscreen: String,
    pub btn_close: String,
    pub summary_title: String,
    pub edit_title: String,
    pub edit_add_label: String,
    pub edit_list_label: String,
    pub edit_hint: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            title: "柳树乡党委委员候选人公开投票选举".into(),
            col_no: "序号".into(),
            col_name: "姓名".into(),
            col_approve: "赞成".into(),
            col_oppose: "不赞成".into(),
            col_abstain: "弃权".into(),
            col_total_approve: "总票数(同意)".into(),
            btn_manage: "⚙ 管理候选人".into(),
            btn_export: "📄 导出结果".into(),
            btn_reset: "🔄 重置".into(),
            btn_submit: "提交第 {round} 轮投票（自动保存）".into(),
            btn_back: "← 返回投票".into(),
            btn_add: "添加".into(),
            btn_fullscreen: "⛶ 全屏".into(),
            btn_close: "✕ 退出".into(),
            summary_title: "累计得票（画正字）".into(),
            edit_title: "候选人管理".into(),
            edit_add_label: "新增姓名：".into(),
            edit_list_label: "现有候选人（× 删除）：".into(),
            edit_hint: "候选人数据存储在 data/candidates.csv，可直接用记事本编辑。".into(),
        }
    }
}

pub fn load_display_config() -> DisplayConfig {
    let base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let path: PathBuf = base.join("data").join("display.toml");

    if !path.exists() {
        return DisplayConfig::default();
    }
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return DisplayConfig::default(),
    };
    toml::from_str(&content).unwrap_or_else(|_| DisplayConfig::default())
}
