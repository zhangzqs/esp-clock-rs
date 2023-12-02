fn main() -> anyhow::Result<()> {
    if !std::path::Path::new("cfg.toml").exists() {
        anyhow::bail!("You need to create a `cfg.toml` file with your Wi-Fi credentials! Use `config.example.toml` as a template.");
    }

    embuild::espidf::sysenv::output();
    Ok(())
}
