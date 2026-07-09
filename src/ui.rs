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
    /// 缓存 5 笔正字纹理（索引 0=1画，4=5画）
    pub tally_tex: [Option<egui::TextureHandle>; 5],
    pub display: DisplayConfig,
    pub system_scale: f32,
    pub user_scale: f32,
    /// 选举标题（从 title.csv 读取，可编辑）
    pub title: String,
    pub title_edit_buf: String,
}

impl VotingApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let data = storage::load_save_data();
        let len = data.candidates.len();
        let display = crate::display::load_display_config();
        // 用 Windows 真实 DPI 作为 system_scale
        let system_scale = cc.egui_ctx.pixels_per_point().max(1.0);
        let title = storage::load_title();
        let title_edit_buf = title.clone();
        Self {
            data,
            current_choices: vec![None; len],
            new_name_buf: String::new(),
            status_msg: String::new(),
            font_loaded: false,
            editing: false,
            tally_tex: [None, None, None, None, None],
            display,
            system_scale,
            user_scale: 1.0,
            title,
            title_edit_buf,
        }
    }

    fn save(&self) { storage::save_save_data(&self.data); }

    pub fn effective_scale(&self) -> f32 {
        // 限制最大缩放不超过 160%，避免内容溢出
        (self.system_scale * self.user_scale).clamp(0.75, 1.6)
    }

    pub fn select(&mut self, idx: usize, choice: VoteChoice) {
        if idx >= self.current_choices.len() { return; }
        self.current_choices[idx] = if self.current_choices[idx] == Some(choice) { None } else { Some(choice) };
    }

    pub fn submit(&mut self) {
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
        self.current_choices = vec![None; self.data.candidates.len()];
        self.save();
        self.status_msg = "✓ 投票结果已确认并保存".to_string();
    }

    pub fn add_candidate(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() { return; }
        let max_id = self.data.candidates.iter().map(|c| c.id).max().unwrap_or(0);
        self.data.candidates.push(Candidate::new(max_id + 1, name));
        storage::save_candidates(&self.data.candidates);
        self.save();
        self.status_msg = format!("✓ 已添加「{}」", name);
    }

    pub fn remove_candidate(&mut self, id: u32) {
        self.data.candidates.retain(|c| c.id != id);
        storage::save_candidates(&self.data.candidates);
        self.save();
        self.status_msg = "✓ 已删除".to_string();
    }

    pub fn reset_votes(&mut self) {
        for c in &mut self.data.candidates { c.approve = 0; c.oppose = 0; c.abstain = 0; }
        self.save();
        self.status_msg = "✓ 已重置".to_string();
    }

    pub fn voted_count(&self) -> usize {
        self.current_choices.iter().filter(|c| c.is_some()).count()
    }
}

impl eframe::App for VotingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.font_loaded {
            self.font_loaded = true;
            load_misans_font(ctx);
        }

        // 应用最终缩放率（系统 DPI × 用户微调）
        ctx.set_pixels_per_point(self.effective_scale());

        // 加载画正字纹理（SVG 矢量渲染）
        for idx in 0..5 {
            if self.tally_tex[idx].is_none() {
                let n = (idx + 1) as u8;
                let key = format!("tally_{}", n);
                let color_img = crate::tally::tally_color_image(n);
                let texture = ctx.load_texture(key, color_img, egui::TextureOptions::default());
                self.tally_tex[idx] = Some(texture);
            }
        }

        egui::TopBottomPanel::top("header")
            .exact_height(56.0)
            .frame(egui::Frame::default().fill(GOV_RED))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        // 标题（可点击编辑）
                        let title_resp = ui.add(egui::Label::new(
                            RichText::new(&self.title).size(20.0).color(GOV_GOLD).strong().underline(),
                        ));
                        title_resp.context_menu(|ui| {
                            ui.text_edit_singleline(&mut self.title_edit_buf);
                            if ui.button("保存").clicked() {
                                let new_title = self.title_edit_buf.trim().to_string();
                                if !new_title.is_empty() {
                                    self.title = new_title.clone();
                                    storage::save_title(&new_title);
                                    self.status_msg = "标题已保存".to_string();
                                }
                                ui.close_menu();
                            }
                        });
                        title_resp.on_hover_text("右键点击编辑标题");

                        ui.label(RichText::new(format!(
                            "共 {} 位 | {:.0}%",
                            self.data.candidates.len(),
                            self.effective_scale() * 100.0
                        )).size(12.0).color(Color32::WHITE));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(Button::new(RichText::new(&self.display.btn_fullscreen).color(Color32::WHITE))
                            .fill(GOV_DARK_RED).min_size(vec2(72.0, 30.0))).clicked() {
                            let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
                        }
                        ui.add_space(6.0);
                        if ui.add(Button::new(RichText::new(&self.display.btn_close).color(Color32::WHITE))
                            .fill(Color32::from_rgb(0x33, 0x33, 0x33)).min_size(vec2(72.0, 30.0))).clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        ui.add_space(16.0);
                    });
                });
            });

        if !self.status_msg.is_empty() {
            egui::TopBottomPanel::bottom("status")
                .exact_height(28.0)
                .frame(egui::Frame::default().fill(Color32::from_rgb(0xE8, 0xE0, 0xD0)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(&self.status_msg).size(13.0).color(GOV_DARK_RED).strong());
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(BG_LIGHT))
            .show(ctx, |ui| {
                if self.editing { render_edit(ui, self); } else { render_voting(ui, self); }
            });

        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
        }
    }
}

fn misans_bytes() -> &'static [u8] { include_bytes!("../resources/font/MiSans-Normal.ttf") }

fn load_misans_font(ctx: &egui::Context) {
    let font_data = FontData::from_static(misans_bytes());
    let mut defs = FontDefinitions::default();
    defs.font_data.insert("MiSans".into(), font_data);
    defs.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "MiSans".into());
    defs.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "MiSans".into());
    ctx.set_fonts(defs);
}

// ─── 投票页面 ─────────────────────────────────────

fn render_voting(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    let total = app.data.candidates.len();
    let voted = app.voted_count();

    // 工具栏（使用换行布局）
    ui.horizontal_wrapped(|ui| {
        if ui.add(Button::new(RichText::new("  管理候选人  ").strong()).fill(GOV_RED).min_size(vec2(110.0, 34.0))).clicked() { app.editing = true; }
        ui.add_space(4.0);
        if ui.add(Button::new(RichText::new("  导出结果  ").strong()).fill(GOV_DARK_RED).min_size(vec2(100.0, 34.0))).clicked() {
            match storage::export_results(&app.data) {
                Ok(p) => app.status_msg = format!("已导出 {}", p.display()),
                Err(e) => app.status_msg = format!("导出失败: {}", e),
            }
        }
        ui.add_space(4.0);
        if ui.add(Button::new(RichText::new("  重置  ").strong()).fill(Color32::from_rgb(0x66, 0x66, 0x66)).min_size(vec2(72.0, 34.0))).clicked() { app.reset_votes(); }
    });

    ui.add_space(4.0);

    // 缩放控制（单独一行）
    ui.horizontal(|ui| {
        ui.label(RichText::new("文字大小:").size(12.0));
        ui.add(Slider::new(&mut app.user_scale, 0.75..=1.5).text(""));
        ui.label(RichText::new(format!("{:.0}%", app.effective_scale() * 100.0)).size(12.0).color(Color32::GRAY));
        if ui.button(RichText::new("恢复").size(12.0)).clicked() { app.user_scale = 1.0; }
    });

    ui.add_space(6.0);

    // ★ 主表格区：使用 Grid（最稳定的行列布局）
    let snap: Vec<(Candidate, Option<VoteChoice>)> = app.data.candidates.iter()
        .zip(app.current_choices.iter())
        .map(|(c, ch)| (c.clone(), *ch)).collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(ui.available_height() - 50.0)
        .show(ui, |ui| {
            egui::Grid::new("voting_grid")
                .striped(true)
                .min_col_width(30.0)
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("序号").color(GOV_GOLD).strong().size(13.0));
                    ui.label(RichText::new("姓名").color(GOV_GOLD).strong().size(13.0));
                    ui.label(RichText::new(&d.col_approve).color(GOV_GOLD).strong().size(13.0));
                    ui.label(RichText::new(&d.col_oppose).color(GOV_GOLD).strong().size(13.0));
                    ui.label(RichText::new(&d.col_abstain).color(GOV_GOLD).strong().size(13.0));
                    ui.label(RichText::new("赞成").color(GOV_GOLD).strong().size(12.0));
                    ui.label(RichText::new("不赞成").color(GOV_GOLD).strong().size(12.0));
                    ui.label(RichText::new("弃权").color(GOV_GOLD).strong().size(12.0));
                    ui.end_row();

                    // 数据行
                    for (i, (cand, ch)) in snap.iter().enumerate() {
                        // 序号 + 姓名
                        ui.label(RichText::new(format!("{:02}", i + 1)).size(14.0).strong().color(GOV_RED));
                        ui.label(RichText::new(&cand.name).size(14.0).strong().color(GOV_DARK_RED));

                        // 三个投票按钮
                        let sel = *ch == Some(VoteChoice::Approve);
                        let btn = egui::Button::new(
                            RichText::new(&d.col_approve).size(13.0).strong().color(Color32::WHITE)
                        )
                        .fill(if sel { Color32::from_rgb(0x1B, 0x7A, 0x2E) } else { Color32::from_rgb(0x81, 0xC7, 0x84) })
                        .rounding(Rounding::same(4.0));
                        if ui.add(btn).clicked() { app.select(i, VoteChoice::Approve); }

                        let sel = *ch == Some(VoteChoice::Oppose);
                        let btn = egui::Button::new(
                            RichText::new(&d.col_oppose).size(13.0).strong().color(Color32::WHITE)
                        )
                        .fill(if sel { Color32::from_rgb(0xC6, 0x28, 0x28) } else { Color32::from_rgb(0xE5, 0x73, 0x73) })
                        .rounding(Rounding::same(4.0));
                        if ui.add(btn).clicked() { app.select(i, VoteChoice::Oppose); }

                        let sel = *ch == Some(VoteChoice::Abstain);
                        let btn = egui::Button::new(
                            RichText::new(&d.col_abstain).size(13.0).strong().color(Color32::WHITE)
                        )
                        .fill(if sel { Color32::from_rgb(0x55, 0x55, 0x55) } else { Color32::from_rgb(0xBB, 0xBB, 0xBB) })
                        .rounding(Rounding::same(4.0));
                        if ui.add(btn).clicked() { app.select(i, VoteChoice::Abstain); }

                        // 三个画正字显示
                        tally_images_ui(ui, cand.approve, &app.tally_tex, 22.0);
                        tally_images_ui(ui, cand.oppose, &app.tally_tex, 22.0);
                        tally_images_ui(ui, cand.abstain, &app.tally_tex, 22.0);

                        ui.end_row();
                    }
                });
        });

    ui.add_space(8.0);

    // 提交按钮
    ui.vertical_centered(|ui| {
        let btn = egui::Button::new(RichText::new("  确认最终结果  ").size(16.0).strong().color(Color32::WHITE))
            .fill(GOV_RED).rounding(Rounding::same(8.0)).min_size(vec2(220.0, 44.0));
        if ui.add(btn).clicked() { app.submit(); }
        ui.add_space(2.0);
        ui.label(RichText::new("点击暂存选择，提交后更新正字").size(10.0).color(Color32::GRAY));
    });
}

pub fn tally_images_ui(ui: &mut egui::Ui, count: u32, tex: &[Option<egui::TextureHandle>; 5], max_height: f32) {
    if count == 0 {
        ui.label(RichText::new("-").size(12.0).color(Color32::GRAY));
        return;
    }
    let full = (count / 5) as usize;
    let rem = (count % 5) as usize;
    ui.horizontal(|ui| {
        for _ in 0..full {
            if let Some(t) = &tex[4] { ui.add(egui::Image::from_texture(t).max_height(max_height)); }
        }
        if rem >= 1 && rem <= 5 {
            if let Some(t) = &tex[rem - 1] { ui.add(egui::Image::from_texture(t).max_height(max_height)); }
        }
        ui.label(RichText::new(format!("({})", count)).size(10.0).color(Color32::GRAY));
    });
}

fn render_edit(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    ui.horizontal(|ui| {
        if ui.button(RichText::new(&d.btn_back).strong().size(14.0)).clicked() { app.editing = false; }
    });
    ui.add_space(12.0);
    ui.heading(RichText::new(&d.edit_title).size(20.0).color(GOV_DARK_RED).strong());
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&d.edit_add_label).size(14.0));
        ui.add(TextEdit::singleline(&mut app.new_name_buf).desired_width(180.0).font(FontId::proportional(15.0)));
        if ui.add(Button::new(RichText::new(&d.btn_add).strong()).fill(Color32::from_rgb(0x22, 0x8B, 0x22)).min_size(vec2(72.0, 32.0))).clicked() {
            app.add_candidate(&app.new_name_buf.clone());
            app.new_name_buf.clear();
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);
    ui.label(RichText::new(&d.edit_list_label).size(14.0).color(GOV_DARK_RED));
    ui.add_space(8.0);

    let mut to_remove: Option<u32> = None;
    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
        for (i, c) in app.data.candidates.iter().enumerate() {
            let bg = if i % 2 == 0 { ROW_EVEN } else { ROW_ODD };
            egui::Frame::default().fill(bg).inner_margin(Margin::symmetric(8.0, 6.0)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}", i + 1)).size(14.0).strong().color(GOV_DARK_RED));
                    ui.add_space(12.0);
                    ui.label(RichText::new(&c.name).size(15.0).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(Button::new(RichText::new("删除").size(12.0).color(Color32::WHITE).strong())
                            .fill(Color32::from_rgb(0xDC, 0x14, 0x3C))).clicked() {
                            to_remove = Some(c.id);
                        }
                    });
                });
            });
            ui.add_space(2.0);
        }
    });

    if let Some(id) = to_remove { app.remove_candidate(id); }

    ui.add_space(20.0);
    ui.separator();
    ui.label(RichText::new(&d.edit_hint).size(11.0).color(Color32::GRAY));
}
