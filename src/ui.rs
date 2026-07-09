//! UI 渲染

use eframe::egui;
use egui::*;

use crate::display::DisplayConfig;
use crate::models::*;
use crate::storage;
use crate::util::*;

// 政务配色
pub const GOV_RED: Color32 = Color32::from_rgb(0xC8, 0x10, 0x2E);
pub const GOV_GOLD: Color32 = Color32::from_rgb(0xD4, 0xA8, 0x43);
pub const GOV_DARK_RED: Color32 = Color32::from_rgb(0x8B, 0x00, 0x00);
pub const BG_LIGHT: Color32 = Color32::from_rgb(0xFA, 0xF7, 0xF0);
pub const ROW_EVEN: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const ROW_ODD: Color32 = Color32::from_rgb(0xF5, 0xF0, 0xE8);

pub struct VotingApp {
    pub data: SaveData,
    pub current_choices: Vec<Option<VoteChoice>>,
    pub new_name_buf: String,
    pub status_msg: String,
    pub font_loaded: bool,
    pub editing: bool,
    pub tally_tex: [Option<egui::TextureHandle>; 5],
    pub display: DisplayConfig,
    /// 系统自动检测的 DPI 缩放率
    pub system_scale: f32,
    /// 用户手动调节的缩放倍率（基于 system_scale 的增量）
    pub user_scale: f32,
}

impl VotingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data = storage::load_save_data();
        let len = data.candidates.len();
        let display = crate::display::load_display_config();
        Self {
            data,
            current_choices: vec![None; len],
            new_name_buf: String::new(),
            status_msg: String::new(),
            font_loaded: false,
            editing: false,
            tally_tex: [None, None, None, None, None],
            display,
            system_scale: 1.0,
            user_scale: 1.0,
        }
    }

    fn save(&self) {
        storage::save_save_data(&self.data);
    }

    /// 最终使用的缩放率 = 系统 DPI × 用户调节
    pub fn effective_scale(&self) -> f32 {
        (self.system_scale * self.user_scale).clamp(0.75, 2.5)
    }

    pub fn submit_round(&mut self) {
        for (i, ch) in self.current_choices.iter().enumerate() {
            if let Some(c) = ch {
                if i < self.data.candidates.len() {
                    match c {
                        VoteChoice::Approve => self.data.candidates[i].approve += 1,
                        VoteChoice::Oppose => self.data.candidates[i].oppose += 1,
                        VoteChoice::Abstain => self.data.candidates[i].abstain += 1,
                    }
                }
            }
        }
        self.data.round += 1;
        self.current_choices = vec![None; self.data.candidates.len()];
        self.save();
        self.status_msg = format!("✓ 第 {} 轮投票已提交并保存", self.data.round);
    }

    pub fn add_candidate(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() { return; }
        let max_id = self.data.candidates.iter().map(|c| c.id).max().unwrap_or(0);
        self.data.candidates.push(Candidate::new(max_id + 1, name));
        self.current_choices.push(None);
        storage::save_candidates(&self.data.candidates);
        self.save();
        self.status_msg = format!("✓ 已添加「{}」", name);
    }

    pub fn remove_candidate(&mut self, id: u32) {
        self.data.candidates.retain(|c| c.id != id);
        self.current_choices = vec![None; self.data.candidates.len()];
        storage::save_candidates(&self.data.candidates);
        self.save();
        self.status_msg = "✓ 已删除候选人".to_string();
    }

    pub fn reset_votes(&mut self) {
        for c in &mut self.data.candidates {
            c.approve = 0;
            c.oppose = 0;
            c.abstain = 0;
        }
        self.data.round = 0;
        self.current_choices = vec![None; self.data.candidates.len()];
        self.save();
        self.status_msg = "✓ 已重置投票数据".to_string();
    }

    pub fn voted_count(&self) -> usize {
        self.current_choices.iter().filter(|c| c.is_some()).count()
    }
}

impl eframe::App for VotingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 加载字体
        if !self.font_loaded {
            self.font_loaded = true;
            load_misans_font(ctx);
        }

        // ★ 从 Windows 系统读取 DPI 缩放率
        let native_ppp = ctx.input(|i| i.viewport().native_pixels_per_point);
        self.system_scale = native_ppp.unwrap_or(1.0);

        // 应用缩放系数（系统 DPI × 用户调节）
        ctx.set_pixels_per_point(self.effective_scale());

        // 加载画正字图片纹理
        for idx in 0..5 {
            if self.tally_tex[idx].is_none() {
                let n = (idx + 1) as u32;
                let key = format!("tally_{}", n);
                if let Some(bytes) = tally_image_bytes(n) {
                    if let Ok(img) = image::load_from_memory(bytes) {
                        let size = [img.width() as _, img.height() as _];
                        let rgba = img.to_rgba8();
                        let pixels = rgba.as_flat_samples();
                        let texture = ctx.load_texture(
                            key,
                            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                            egui::TextureOptions::default(),
                        );
                        self.tally_tex[idx] = Some(texture);
                    }
                }
            }
        }

        // ── 标题栏 ──
        egui::TopBottomPanel::top("header")
            .exact_height(68.0)
            .frame(egui::Frame::default().fill(GOV_RED))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        ui.heading(
                            RichText::new(&self.display.title)
                                .size(22.0)
                                .color(GOV_GOLD)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!(
                                "已完成 {} 轮 ｜ 共 {} 位候选人",
                                self.data.round,
                                self.data.candidates.len()
                            ))
                            .size(13.0)
                            .color(Color32::WHITE),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(Button::new(
                                RichText::new(&self.display.btn_fullscreen).color(Color32::WHITE),
                            ).fill(GOV_DARK_RED).min_size(vec2(72.0, 32.0)))
                            .clicked()
                        {
                            let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
                        }
                        ui.add_space(6.0);
                        if ui
                            .add(Button::new(
                                RichText::new(&self.display.btn_close).color(Color32::WHITE),
                            ).fill(Color32::from_rgb(0x33, 0x33, 0x33)).min_size(vec2(72.0, 32.0)))
                            .clicked()
                        {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        ui.add_space(16.0);
                    });
                });
            });

        // ── 反馈栏 ──
        if !self.status_msg.is_empty() {
            egui::TopBottomPanel::bottom("status")
                .exact_height(30.0)
                .frame(egui::Frame::default().fill(Color32::from_rgb(0xE8, 0xE0, 0xD0)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(&self.status_msg)
                                .size(13.0)
                                .color(GOV_DARK_RED)
                                .strong(),
                        );
                    });
                });
        }

        // ── 主面板 ──
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BG_LIGHT))
            .show(ctx, |ui| {
                if self.editing {
                    render_edit(ui, self);
                } else {
                    render_voting(ui, self);
                }
            });

        // F11 全屏
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
        }
    }
}

// ──────────────────────────────────────────────
// 字体/图片工具
// ──────────────────────────────────────────────

fn misans_bytes() -> &'static [u8] {
    include_bytes!("../resources/font/MiSans-Normal.ttf")
}

fn tally_image_bytes(n: u32) -> Option<&'static [u8]> {
    match n {
        1 => Some(include_bytes!("../resources/img/1.png")),
        2 => Some(include_bytes!("../resources/img/2.png")),
        3 => Some(include_bytes!("../resources/img/3.png")),
        4 => Some(include_bytes!("../resources/img/4.png")),
        5 => Some(include_bytes!("../resources/img/5.png")),
        _ => None,
    }
}

fn load_misans_font(ctx: &egui::Context) {
    let font_data = FontData::from_static(misans_bytes());
    let mut defs = FontDefinitions::default();
    defs.font_data.insert("MiSans".into(), font_data);
    defs.families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "MiSans".into());
    defs.families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "MiSans".into());
    ctx.set_fonts(defs);
}

// ──────────────────────────────────────────────
// 投票页面
// ──────────────────────────────────────────────

fn render_voting(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    let total = app.data.candidates.len();
    let voted = app.voted_count();
    let pending = total - voted;

    // ── 工具栏 + 缩放滑块 ──
    ui.horizontal(|ui| {
        if ui
            .add(Button::new(RichText::new(&d.btn_manage).strong()).fill(GOV_RED).min_size(vec2(120.0, 34.0)))
            .clicked()
        {
            app.editing = true;
        }
        if ui
            .add(Button::new(RichText::new(&d.btn_export).strong()).fill(GOV_DARK_RED).min_size(vec2(120.0, 34.0)))
            .clicked()
        {
            match storage::export_results(&app.data) {
                Ok(p) => app.status_msg = format!("✓ 已导出 {}", p.display()),
                Err(e) => app.status_msg = format!("✗ {}", e),
            }
        }
        if ui
            .add(Button::new(RichText::new(&d.btn_reset).strong()).fill(Color32::from_rgb(0x66, 0x66, 0x66)).min_size(vec2(80.0, 34.0)))
            .clicked()
        {
            app.reset_votes();
        }

        ui.separator();

        // ★ 字体大小滑块
        ui.label(RichText::new("🔤").size(16.0));
        ui.label(RichText::new("大小").size(13.0));
        let slider = Slider::new(&mut app.user_scale, 0.75..=2.0)
            .text("")
            .clamp_to_range(true);
        ui.add(slider);
        ui.label(
            RichText::new(format!("{:.0}%", app.effective_scale() * 100.0))
                .size(12.0)
                .color(Color32::GRAY),
        );
        if ui.button(RichText::new("↺").size(16.0)).clicked() {
            app.user_scale = 1.0;
        }
    });

    ui.add_space(12.0);

    // ★ ★ ★  进度提示卡片  ★ ★ ★
    egui::Frame::default()
        .fill(if pending == 0 {
            Color32::from_rgb(0xE8, 0xF5, 0xE9)
        } else {
            Color32::from_rgb(0xFF, 0xF3, 0xE0)
        })
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("第 {} 轮投票", app.data.round + 1))
                        .size(20.0)
                        .strong()
                        .color(GOV_DARK_RED),
                );
                ui.add_space(24.0);
                ui.label(
                    RichText::new(format!("已投票：{} / {} 人", voted, total))
                        .size(18.0)
                        .color(Color32::from_rgb(0xE6, 0x51, 0x00)),
                );
                ui.add_space(20.0);
                if pending > 0 {
                    ui.label(
                        RichText::new(format!("⚠ 还有 {} 人未选择", pending))
                            .size(15.0)
                            .color(Color32::from_rgb(0xE6, 0x51, 0x00))
                            .strong(),
                    );
                } else {
                    ui.label(
                        RichText::new("✅ 全部已选择，请提交")
                            .size(15.0)
                            .color(Color32::from_rgb(0x1B, 0x7A, 0x2E))
                            .strong(),
                    );
                }
            });
        });

    ui.add_space(14.0);

    // ★ ★ ★  候选人投票卡片区  ★ ★ ★
    let snap: Vec<(Candidate, Option<VoteChoice>)> = app
        .data
        .candidates
        .iter()
        .zip(app.current_choices.iter())
        .map(|(c, ch)| (c.clone(), *ch))
        .collect();

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 70.0)
        .show(ui, |ui| {
            for (i, (cand, ch)) in snap.iter().enumerate() {
                let voted_flag = ch.is_some();
                let card_bg = if !voted_flag {
                    Color32::from_rgb(0xFF, 0xF8, 0xE1)
                } else if i % 2 == 0 {
                    ROW_EVEN
                } else {
                    ROW_ODD
                };

                egui::Frame::default()
                    .fill(card_bg)
                    .inner_margin(Margin::symmetric(10.0, 14.0))
                    .outer_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 4.0 })
                    .rounding(Rounding::same(6.0))
                    .show(ui, |ui| {
                        // 行 1：序号 + 姓名
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:02}", i + 1))
                                    .size(20.0)
                                    .strong()
                                    .color(GOV_RED),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(&cand.name)
                                    .size(20.0)
                                    .strong()
                                    .color(GOV_DARK_RED),
                            );
                            ui.add_space(16.0);
                            if !voted_flag {
                                ui.label(
                                    RichText::new("⚡ 请投票").size(14.0).color(Color32::from_rgb(0xE6, 0x51, 0x00)).strong(),
                                );
                            } else {
                                let (txt, col) = match ch.unwrap() {
                                    VoteChoice::Approve => ("已选：赞成", Color32::from_rgb(0x1B, 0x7A, 0x2E)),
                                    VoteChoice::Oppose => ("已选：不赞成", Color32::from_rgb(0xC6, 0x28, 0x28)),
                                    VoteChoice::Abstain => ("已选：弃权", Color32::from_rgb(0x55, 0x55, 0x55)),
                                };
                                ui.label(RichText::new(txt).size(14.0).color(col).strong());
                            }
                        });

                        ui.add_space(8.0);

                        // 行 2：三个大按钮
                        ui.horizontal(|ui| {
                            // 赞成
                            let sel_approve = *ch == Some(VoteChoice::Approve);
                            let btn_approve = egui::Button::new(
                                RichText::new(format!("👍 {}", d.col_approve)).size(15.0).strong().color(Color32::WHITE),
                            )
                            .fill(if sel_approve { Color32::from_rgb(0x1B, 0x7A, 0x2E) } else { Color32::from_rgb(0x81, 0xC7, 0x84) })
                            .rounding(Rounding::same(6.0))
                            .min_size(vec2(130.0, 44.0));
                            if ui.add(btn_approve).clicked() {
                                app.current_choices[i] = if sel_approve { None } else { Some(VoteChoice::Approve) };
                            }

                            ui.add_space(10.0);

                            // 不赞成
                            let sel_oppose = *ch == Some(VoteChoice::Oppose);
                            let btn_oppose = egui::Button::new(
                                RichText::new(format!("👎 {}", d.col_oppose)).size(15.0).strong().color(Color32::WHITE),
                            )
                            .fill(if sel_oppose { Color32::from_rgb(0xC6, 0x28, 0x28) } else { Color32::from_rgb(0xE5, 0x73, 0x73) })
                            .rounding(Rounding::same(6.0))
                            .min_size(vec2(130.0, 44.0));
                            if ui.add(btn_oppose).clicked() {
                                app.current_choices[i] = if sel_oppose { None } else { Some(VoteChoice::Oppose) };
                            }

                            ui.add_space(10.0);

                            // 弃权
                            let sel_abstain = *ch == Some(VoteChoice::Abstain);
                            let btn_abstain = egui::Button::new(
                                RichText::new(format!("○ {}", d.col_abstain)).size(15.0).strong().color(Color32::WHITE),
                            )
                            .fill(if sel_abstain { Color32::from_rgb(0x55, 0x55, 0x55) } else { Color32::from_rgb(0xBB, 0xBB, 0xBB) })
                            .rounding(Rounding::same(6.0))
                            .min_size(vec2(130.0, 44.0));
                            if ui.add(btn_abstain).clicked() {
                                app.current_choices[i] = if sel_abstain { None } else { Some(VoteChoice::Abstain) };
                            }
                        });
                    });
            }
        });

    ui.add_space(12.0);

    // ── 提交按钮 ──
    ui.vertical_centered(|ui| {
        let ready = pending == 0;
        let btn = egui::Button::new(
            RichText::new(format!("📨 提交第 {} 轮", app.data.round + 1))
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(if ready { GOV_RED } else { Color32::from_rgb(0xAA, 0xAA, 0xAA) })
        .rounding(Rounding::same(8.0))
        .min_size(vec2(300.0, 50.0));
        if ui.add(btn).clicked() {
            app.submit_round();
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new("提交后自动保存 data/votes.json ｜ 重启可继续")
                .size(12.0)
                .color(Color32::GRAY),
        );
    });
}

// ──────────────────────────────────────────────
// 画正字图片工具
// ──────────────────────────────────────────────

pub fn tally_images_ui(
    ui: &mut egui::Ui,
    count: u32,
    tex: &[Option<egui::TextureHandle>; 5],
    max_height: f32,
) {
    if count == 0 {
        ui.label(RichText::new("—").size(14.0).color(Color32::GRAY));
        return;
    }
    let node = TallyNode::new(count);
    ui.horizontal(|ui| {
        for _ in 0..node.full {
            if let Some(t) = &tex[4] {
                ui.add(egui::Image::from_texture(t).max_height(max_height));
            }
        }
        if node.remainder >= 1 && node.remainder <= 5 {
            let idx = (node.remainder - 1) as usize;
            if let Some(t) = &tex[idx] {
                ui.add(egui::Image::from_texture(t).max_height(max_height));
            }
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("({})", count))
                .size(11.0)
                .color(Color32::GRAY),
        );
    });
}

// ──────────────────────────────────────────────
// 编辑候选人页面
// ──────────────────────────────────────────────

fn render_edit(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    ui.horizontal(|ui| {
        if ui.button(RichText::new(&d.btn_back).strong().size(15.0)).clicked() {
            app.editing = false;
        }
    });
    ui.add_space(12.0);

    ui.heading(RichText::new(&d.edit_title).size(22.0).color(GOV_DARK_RED).strong());
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&d.edit_add_label).size(15.0));
        ui.add(TextEdit::singleline(&mut app.new_name_buf).desired_width(200.0).font(FontId::proportional(16.0)));
        if ui
            .add(Button::new(RichText::new(&d.btn_add).strong().size(14.0))
                .fill(Color32::from_rgb(0x22, 0x8B, 0x22)).min_size(vec2(80.0, 36.0)))
            .clicked()
        {
            app.add_candidate(&app.new_name_buf.clone());
            app.new_name_buf.clear();
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);

    ui.label(RichText::new(&d.edit_list_label).size(15.0).color(GOV_DARK_RED));
    ui.add_space(8.0);

    let mut to_remove: Option<u32> = None;
    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
        for (i, c) in app.data.candidates.iter().enumerate() {
            let bg = if i % 2 == 0 { ROW_EVEN } else { ROW_ODD };
            egui::Frame::default()
                .fill(bg)
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}", i + 1)).size(14.0).strong().color(GOV_DARK_RED));
                        ui.add_space(12.0);
                        ui.label(RichText::new(&c.name).size(15.0).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(Button::new(RichText::new("×").size(16.0).color(Color32::WHITE).strong())
                                    .fill(Color32::from_rgb(0xDC, 0x14, 0x3C)))
                                .clicked()
                            {
                                to_remove = Some(c.id);
                            }
                        });
                    });
                });
            ui.add_space(2.0);
        }
    });

    if let Some(id) = to_remove {
        app.remove_candidate(id);
    }

    ui.add_space(20.0);
    ui.separator();
    ui.label(
        RichText::new(&d.edit_hint)
            .size(12.0)
            .color(Color32::GRAY),
    );
}

