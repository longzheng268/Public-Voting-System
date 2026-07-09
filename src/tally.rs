//! 画正字：用 SVG 矢量路径实时渲染每一笔

use egui;

/// 正字 5 笔的 SVG path 数据（来自书法 SVG 拆分的每一笔）
const STROKE_1: &str = r##"<path d="M505,667C561,678 621,688 683,699C723,707 747,715 754,720C760,726 762,732 759,738C754,747 741,756 719,761C697,767 649,754 574,734C472,709 387,699 321,693C293,691 287,681 308,668C338,649 379,646 431,655C439,657 449,659 460,659C490,664 490,664 505,667Z" fill="#333" />"##;

const STROKE_2: &str = r##"<path d="M534,154C536,250 537,339 539,420C540,445 540,445 540,457C542,575 546,605 544,631C542,650 517,661 505,667C491,673 454,673 460,659C475,623 485,606 485,591C485,569 484,204 484,149C484,132 534,137 534,154Z" fill="#333" />"##;

const STROKE_3: &str = r##"<path d="M539,420C565,414 590,417 616,422C709,444 760,453 767,457C774,463 775,470 772,475C766,484 752,491 732,495C710,500 688,496 667,487C646,480 625,474 603,468C583,464 562,461 540,457C528,455 527,422 539,420Z" fill="#333" />"##;

const STROKE_4: &str = r##"<path d="M346,135C330,275 324,364 327,402C328,419 323,433 312,441C294,454 274,463 254,468C243,471 234,471 228,465C222,461 222,452 228,441C243,413 258,385 266,354C273,325 284,250 300,131C303,116 349,120 346,135Z" fill="#333" />"##;

const STROKE_5: &str = r##"<path d="M300,131C235,127 168,120 100,114C83,113 80,106 93,91C103,79 116,71 131,66C147,61 162,61 176,64C370,109 610,123 884,110C895,110 914,113 918,119C923,129 919,141 906,151C861,187 821,198 784,190C719,178 636,168 534,154C501,151 501,151 484,149C439,145 393,141 346,135C315,132 315,132 300,131Z" fill="#333" />"##;

const FULL_SVG_TEMPLATE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 800" width="100" height="80">{strokes}</svg>"##;

/// 生成显示「画正字」count 笔的 SVG 字符串
pub fn tally_svg(count: u8) -> String {
    let strokes = match count {
        0 => String::new(),
        1 => STROKE_1.to_string(),
        2 => format!("{}{}", STROKE_1, STROKE_2),
        3 => format!("{}{}{}", STROKE_1, STROKE_2, STROKE_3),
        4 => format!("{}{}{}{}", STROKE_1, STROKE_2, STROKE_3, STROKE_4),
        _ => format!(
            "{}{}{}{}{}",
            STROKE_1, STROKE_2, STROKE_3, STROKE_4, STROKE_5
        ),
    };
    FULL_SVG_TEMPLATE.replace("{strokes}", &strokes)
}

/// 渲染画正字为 egui ColorImage（每一票画一笔，到 5 笔为一个完整的正字）
pub fn tally_color_image(count: u8) -> egui::ColorImage {
    let n = if count == 0 { 0 } else { ((count - 1) % 5) + 1 };
    let svg_str = tally_svg(n);

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(
        &svg_str,
        &resvg::usvg::Options::default(),
        &fontdb,
    )
    .expect("SVG parse failed");

    let size = tree.size().to_int_size();
    let w = size.width().max(1);
    let h = size.height().max(1);

    let transform = resvg::tiny_skia::Transform::identity();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).unwrap();
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia 的像素是 top-to-bottom，egui ColorImage 需要 bottom-to-top → 翻转 Y
    let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
    for y in (0..h).rev() {
        for x in 0..w {
            let p = pixmap.pixel(x, y).unwrap();
            rgba.push(p.red());
            rgba.push(p.green());
            rgba.push(p.blue());
            rgba.push(p.alpha());
        }
    }

    egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba)
}
