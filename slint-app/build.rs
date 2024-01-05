fn build_slint_embedded() -> Result<(), Box<dyn std::error::Error>> {
    let slint_cfg = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/appwindow.slint", slint_cfg)?;
    Ok(())
}

fn build_slint_normal() -> Result<(), Box<dyn std::error::Error>> {
    let slint_cfg = slint_build::CompilerConfiguration::new();
    slint_build::compile_with_config("ui/appwindow.slint", slint_cfg)?;
    Ok(())
}

fn build_slint() -> Result<(), Box<dyn std::error::Error>> {
    // 先按照软件渲染器编译
    if let Err(e) = build_slint_embedded() {
        println!("build slint with embedded failed: {}", e);
        // 如果失败了，再按照普通的编译
        build_slint_normal()?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_slint()?;
    Ok(())
}
