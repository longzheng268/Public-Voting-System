//! 柳树乡党委委员候选人公开投票选举软件

mod display;
mod models;
mod storage;
mod tally;
mod ui;
mod util;

use eframe::egui;
use ui::VotingApp;
use std::process::Command;

/// 读取 Windows 系统真实 DPI 缩放率（通过注册表）
fn windows_dpi_scale() -> f32 {
    let output = Command::new("reg")
        .args(&[
            "query",
            r"HKCU\Control Panel\Desktop\WindowMetrics",
            "/v",
            "AppliedDPI",
        ])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if line.contains("AppliedDPI") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let Ok(dpi) = parts.last().unwrap().parse::<f32>() {
                            return dpi / 96.0;
                        }
                    }
                }
            }
        }
    }
    1.0
}

fn main() -> eframe::Result<()> {
    let dpi_scale = windows_dpi_scale();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(900.0 * dpi_scale, 700.0 * dpi_scale))
            .with_min_inner_size(egui::vec2(700.0 * dpi_scale, 500.0 * dpi_scale))
            .with_title("柳树乡党委委员候选人公开投票选举"),
        ..Default::default()
    };
    let dpi = dpi_scale;
    eframe::run_native(
        "柳树乡党委委员候选人公开投票选举",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_pixels_per_point(dpi);
            Ok(Box::new(VotingApp::new(cc)))
        }),
    )
}
