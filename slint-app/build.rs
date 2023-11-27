fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slint_cfg = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/appwindow.slint", slint_cfg).unwrap();
    // slint_build::compile("ui/appwindow.slint").unwrap();
    Ok(())
}
