use std::process::Command;

fn build_slint() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "embedded")]
    let slint_cfg = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    #[cfg(not(feature = "embedded"))]
    let slint_cfg = slint_build::CompilerConfiguration::new();
    slint_build::compile_with_config("ui/appwindow.slint", slint_cfg)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_slint()?;
    Ok(())
}
