use std::io::Read;

pub fn load_settings() -> Settings {
    let mut settings_file =
        std::fs::File::open("./settings.toml").expect("failed to open settings.toml");
    let mut settings_toml = String::new();
    settings_file
        .read_to_string(&mut settings_toml)
        .expect("failed to load settings.toml");
    toml::from_str(&settings_toml).unwrap()
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub music: bool,
    pub voice: bool,
    pub sounds: bool,
    pub enemies: bool,
    pub dozer_drive: bool,
}
