use config_rs::{Config, File, FileFormat};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct IcpConfig{
    pub identity_pem_path: String,
    pub icp_domain: String,
    pub agent_canister_id: String,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig{
    pub debug: bool,
    pub log_dir_file_name: String,
    pub icp_config: IcpConfig,
    pub database_url: String,
}

// constructer
impl AppConfig {
    pub fn new() -> AppConfig {
        let app_config = Config::builder()
            .add_source(File::new("application", FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<AppConfig>()
            .unwrap();
        // dbg!(&app_config);
        app_config
    }
}
