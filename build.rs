// 构建脚本：字体通过 include_bytes! 直接嵌入二进制，不需要额外操作

fn main() {
    // 监控字体文件变化
    println!("cargo:rerun-if-changed=resources/font/MiSans-Normal.ttf");

    // 监控 5 张正字图片变化
    for n in 1..=5 {
        println!("cargo:rerun-if-changed=resources/img/{}.png", n);
    }
    println!("cargo:rerun-if-changed=resources/img/");

    // 嵌入 Windows ico 图标资源
    #[cfg(windows)]
    {
        let _ = embed_resource::compile("resources/icons/embed_icon.rc", std::iter::empty::<&str>());
    }
}
