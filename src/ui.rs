//! UI 渲染

use eframe::egui;
use egui::*;

use crate::display::DisplayConfig;
use crate::models::*;
use crate::storage;
use crate::util::*;

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
    pub system_scale: f32,
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

    pub fn effective_scale(&self) -> f32 {
        (self.system_scale * self.user_scale).clamp(0.75, 2.5)
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

    pub fn voted_count(&self) -> usize {
        self.current_choices.iter().filter(|c| c.is_some()).count()
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
        self.status_msg = "✓ 已删除候选人".to_string();
    }

    pub fn reset_votes(&mut self) {
        for c in &mut self.data.candidates {
            c.approve = 0;
            c.oppose = 0;
            c.abstain = 0;
        }
        self.save();
        self.status_msg = "✓ 已重置投票数据".to_string();
    }
}

impl eframe::App for VotingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.font_loaded {
            self.font_loaded = true;
            load_misans_font(ctx);
        }

        let native_ppp = ctx.input(|i| i.viewport().native_pixels_per_point);
        self.system_scale = native_ppp.unwrap_or(1.0);
        ctx.set_pixels_per_point(self.effective_scale());

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

        egui::TopBottomPanel::top("header")
            .exact_height(64.0)
            .frame(egui::Frame::default().fill(GOV_RED))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        ui.heading(RichText::new(&self.display.title).size(22.0).color(GOV_GOLD).strong());
                        ui.label(RichText::new(format!("共 {} 位候选人 ｜ 显示缩放：{:.0}%", self.data.candidates.len(), self.effective_scale() * 100.0)).size(12.0).color(Color32::WHITE));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(Button::new(RichText::new(&self.display.btn_fullscreen).color(Color32::WHITE)).fill(GOV_DARK_RED).min_size(vec2(72.0, 32.0))).clicked() {
                            let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
                        }
                        ui.add_space(6.0);
                        if ui.add(Button::new(RichText::new(&self.display.btn_close).color(Color32::WHITE)).fill(Color32::from_rgb(0x33, 0x33, 0x33)).min_size(vec2(72.0, 32.0))).clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        ui.add_space(16.0);
                    });
                });
            });

        if !self.status_msg.is_empty() {
            egui::TopBottomPanel::bottom("status")
                .exact_height(30.0)
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
    defs.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "MiSans".into());
    defs.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "MiSans".into());
    ctx.set_fonts(defs);
}

// ─── 投票页面 ─────────────────────────────────────

fn render_voting(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    let total = app.data.candidates.len();
    let voted = app.voted_count();
    let pending = total - voted;

    // 工具栏
    ui.horizontal(|ui| {
        if ui.add(Button::new(RichText::new(&d.btn_manage).strong()).fill(GOV_RED).min_size(vec2(120.0, 34.0))).clicked() { app.editing = true; }
        if ui.add(Button::new(RichText::new(&d.btn_export).strong()).fill(GOV_DARK_RED).min_size(vec2(120.0, 34.0))).clicked() {
            match storage::export_results(&app.data) {
                Ok(p) => app.status_msg = format!("✓ 已导出 {}", p.display()),
                Err(e) => app.status_msg = format!("✗ {}", e),
            }
        }
        if ui.add(Button::new(RichText::new(&d.btn_reset).strong()).fill(Color32::from_rgb(0x66, 0x66, 0x66)).min_size(vec2(80.0, 34.0))).clicked() { app.reset_votes(); }
        ui.separator();
        ui.label(RichText::new("🔤").size(16.0));
        ui.add(Slider::new(&mut app.user_scale, 0.75..=2.0).text(""));
        ui.label(RichText::new(format!("{:.0}%", app.effective_scale() * 100.0)).size(12.0).color(Color32::GRAY));
        if ui.button(RichText::new("↺").size(16.0)).clicked() { app.user_scale = 1.0; }
    });

    ui.add_space(10.0);

    // ★ 进度卡片
    egui::Frame::default()
        .fill(if pending == 0 { Color32::from_rgb(0xE8, 0xF5, 0xE9) } else { Color32::from_rgb(0xFF, 0xF3, 0xE0) })
        .inner_margin(Margin::symmetric(14.0, 10.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("请选择").size(18.0).strong().color(GOV_DARK_RED));
                ui.add_space(16.0);
                ui.label(RichText::new(format!("已选 {} / {} 人", voted, total)).size(16.0).color(if pending == 0 { Color32::from_rgb(0x1B, 0x7A, 0x2E) } else { Color32::from_rgb(0xE6, 0x51, 0x00) }));
                if pending > 0 {
                    ui.add_space(12.0);
                    ui.label(RichText::new(format!("⚠ 还有 {} 人未选", pending)).size(14.0).color(Color32::from_rgb(0xE6, 0x51, 0x00)).strong());
                }
            });
        });

    ui.add_space(10.0);

    // 表头
    egui::Frame::default().fill(GOV_RED).inner_margin(Margin::symmetric(6.0, 6.0)).show(ui, |ui| {
        ui.columns(8, |cols| {
            let labels = ["序号", "姓名", "赞成", "不赞成", "弃权", "赞成✓", "不赞成✓", "弃权✓"];
            for (i, label) in labels.iter().enumerate() {
                cols[i].centered_and_justified(|ui| {
                    ui.label(RichText::new(*label).color(GOV_GOLD).strong().size(13.0));
                });
            }
        });
    });

    ui.add_space(4.0);

    // 候选人行（点击 = 暂存选择，提交才生效 → 更新正字）
    let snap: Vec<(Candidate, Option<VoteChoice>)> = app.data.candidates.iter()
        .zip(app.current_choices.iter())
        .map(|(c, ch)| (c.clone(), *ch)).collect();

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 70.0)
        .show(ui, |ui| {
            for (i, (cand, ch)) in snap.iter().enumerate() {
                let voted_flag = ch.is_some();
                let bg = if !voted_flag { Color32::from_rgb(0xFF, 0xF8, 0xE1) } else if i % 2 == 0 { ROW_EVEN } else { ROW_ODD };

                egui::Frame::default().fill(bg).inner_margin(Margin::symmetric(6.0, 8.0)).show(ui, |ui| {
                    ui.columns(8, |cols| {
                        // 序号
                        cols[0].centered_and_justified(|ui| {
                            ui.label(RichText::new(format!("{:02}", i + 1)).size(16.0).strong().color(GOV_RED));
                        });
                        // 姓名
                        cols[1].centered_and_justified(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&cand.name).size(16.0).strong().color(GOV_DARK_RED));
                                if !voted_flag {
                                    ui.label(RichText::new("●").size(10.0).color(Color32::from_rgb(0xE6, 0x51, 0x00)));
                                }
                            });
                        });
                        // 赞成按钮（暂存状态 → 深色 = 已选）
                        cols[2].centered_and_justified(|ui| {
                            let sel = *ch == Some(VoteChoice::Approve);
                            let btn = egui::Button::new(RichText::new(&d.col_approve).size(14.0).strong().color(Color32::WHITE))
                                .fill(if sel { Color32::from_rgb(0x1B, 0x7A, 0x2E) } else { Color32::from_rgb(0x81, 0xC7, 0x84) })
                                .rounding(Rounding::same(6.0)).min_size(vec2(0.0, 40.0));
                            if ui.add(btn).clicked() { app.select(i, VoteChoice::Approve); }
                        });
                        // 不赞成按钮
                        cols[3].centered_and_justified(|ui| {
                            let sel = *ch == Some(VoteChoice::Oppose);
                            let btn = egui::Button::new(RichText::new(&d.col_oppose).size(14.0).strong().color(Color32::WHITE))
                                .fill(if sel { Color32::from_rgb(0xC6, 0x28, 0x28) } else { Color32::from_rgb(0xE5, 0x73, 0x73) })
                                .rounding(Rounding::same(6.0)).min_size(vec2(0.0, 40.0));
                            if ui.add(btn).clicked() { app.select(i, VoteChoice::Oppose); }
                        });
                        // 弃权按钮
                        cols[4].centered_and_justified(|ui| {
                            let sel = *ch == Some(VoteChoice::Abstain);
                            let btn = egui::Button::new(RichText::new(&d.col_abstain).size(14.0).strong().color(Color32::WHITE))
                                .fill(if sel { Color32::from_rgb(0x55, 0x55, 0x55) } else { Color32::from_rgb(0xBB, 0xBB, 0xBB) })
                                .rounding(Rounding::same(6.0)).min_size(vec2(0.0, 40.0));
                            if ui.add(btn).clicked() { app.select(i, VoteChoice::Abstain); }
                        });
                        // 赞成正字（提交后才更新）
                        cols[5].centered_and_justified(|ui| {
                            tally_images_ui(ui, cand.approve, &app.tally_tex, 24.0);
                        });
                        // 不赞成正字
                        cols[6].centered_and_justified(|ui| {
                            tally_images_ui(ui, cand.oppose, &app.tally_tex, 24.0);
                        });
                        // 弃权正字
                        cols[7].centered_and_justified(|ui| {
                            tally_images_ui(ui, cand.abstain, &app.tally_tex, 24.0);
                        });
                    });
                });
                ui.add_space(2.0);
            }
        });

    ui.add_space(10.0);

    // 提交确认按钮
    ui.vertical_centered(|ui| {
        let btn = egui::Button::new(RichText::new("📨 确认最终结果").size(18.0).strong().color(Color32::WHITE))
            .fill(GOV_RED).rounding(Rounding::same(8.0)).min_size(vec2(280.0, 48.0));
        if ui.add(btn).clicked() { app.submit(); }
        ui.add_space(4.0);
        ui.label(RichText::new("点击按钮暂存选择 → 提交后正字实时更新并保存").size(11.0).color(Color32::GRAY));
    });
}

// ─── 画正字工具 ─────────────────────────────────────

pub fn tally_images_ui(ui: &mut egui::Ui, count: u32, tex: &[Option<egui::TextureHandle>; 5], max_height: f32) {
    if count == 0 {
        ui.label(RichText::new("—").size(14.0).color(Color32::GRAY));
        return;
    }
    let full = (count / 5) as usize;
    let rem = (count % 5) as usize;
    ui.horizontal(|ui| {
        for _ in 0..full {
            if let Some(t) = &tex[4] { ui.add(egui::Image::from_texture(t).max_height(max_height)); }
        }
        if rem >= 1 && rem <= 5 {
            let idx = rem - 1;
            if let Some(t) = &tex[idx] { ui.add(egui::Image::from_texture(t).max_height(max_height)); }
        }
        ui.add_space(3.0);
        ui.label(RichText::new(format!("({})", count)).size(11.0).color(Color32::GRAY));
    });
}

// ─── 编辑候选人页面 ─────────────────────────────────

fn render_edit(ui: &mut egui::Ui, app: &mut VotingApp) {
    let d = app.display.clone();
    ui.horizontal(|ui| {
        if ui.button(RichText::new(&d.btn_back).strong().size(15.0)).clicked() { app.editing = false; }
    });
    ui.add_space(12.0);
    ui.heading(RichText::new(&d.edit_title).size(22.0).color(GOV_DARK_RED).strong());
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&d.edit_add_label).size(15.0));
        ui.add(TextEdit::singleline(&mut app.new_name_buf).desired_width(200.0).font(FontId::proportional(16.0)));
        if ui.add(Button::new(RichText::new(&d.btn_add).strong().size(14.0)).fill(Color32::from_rgb(0x22, 0x8B, 0x22)).min_size(vec2(80.0, 36.0))).clicked() {
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
            egui::Frame::default().fill(bg).inner_margin(Margin::symmetric(8.0, 6.0)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}", i + 1)).size(14.0).strong().color(GOV_DARK_RED));
                    ui.add_space(12.0);
                    ui.label(RichText::new(&c.name).size(15.0).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(Button::new(RichText::new("×").size(16.0).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xDC, 0x14, 0x3C))).clicked() {
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
    ui.label(RichText::new(&d.edit_hint).size(12.0).color(Color32::GRAY));
}
