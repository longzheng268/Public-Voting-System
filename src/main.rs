//! 柳树乡党委委员候选人公开投票选举软件
//! 画正字 · 政务风格 · 数据持久化 · 模块化设计
//!
//! 目录结构：
//!   data/candidates.csv —— 候选人名单（可编辑）
//!   data/votes.json     —— 投票数据（自动保存）
//!   data/投票结果.txt    —— 导出结果

mod display;
mod models;
mod storage;
mod tally;
mod ui;
mod util;

use eframe::egui;
use ui::VotingApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(900.0, 750.0))
            .with_min_inner_size(egui::vec2(800.0, 600.0))
            .with_title("柳树乡党委委员候选人公开投票选举"),
        ..Default::default()
    };
    eframe::run_native(
        "柳树乡党委委员候选人公开投票选举",
        options,
        Box::new(|cc| Ok(Box::new(VotingApp::new(cc)))),
    )
}
