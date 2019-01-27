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

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub music: bool,
    pub voice: bool,
    pub sounds: bool,
    pub enemies: bool,

    pub round1_dozers: u32,
    pub round1_swat: u32,
    pub round1_crates: u32,
    pub round1_rocks: u32,

    pub round2_dozers: u32,
    pub round2_swat: u32,
    pub round2_crates: u32,
    pub round2_rocks: u32,

    pub round3_dozers: u32,
    pub round3_swat: u32,
    pub round3_crates: u32,
    pub round3_rocks: u32,

    pub round4_dozers: u32,
    pub round4_swat: u32,
    pub round4_crates: u32,
    pub round4_rocks: u32,

    pub round5_dozers: u32,
    pub round5_swat: u32,
    pub round5_crates: u32,
    pub round5_rocks: u32,
}
