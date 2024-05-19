use slint_build::CompilerConfiguration;

fn build_slint() -> Result<(), Box<dyn std::error::Error>> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "software-renderer")] {
            use slint_build::EmbedResourcesKind;
            let slint_cfg = CompilerConfiguration::new().embed_resources(EmbedResourcesKind::EmbedForSoftwareRenderer);
            slint_build::compile_with_config("ui/app.slint", slint_cfg)?;
        } else {
            let slint_cfg = CompilerConfiguration::new();
            slint_build::compile_with_config("ui/app.slint", slint_cfg)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_slint()?;
    Ok(())
}
