use serde::Deserialize;

pub(crate) const CONFIG_PATH: &str = "./.saptest.toml";
pub(crate) const DEFAULT_CONFIG: LibConfig = LibConfig {
    database: DatabaseConfig {
        pets_version: None,
        foods_version: None,
        tokens_version: None,
        names_version: None,
        filename: None,
        update_on_startup: true,
    },
    general: GeneralConfig { build_graph: true },
};

#[derive(Deserialize)]
pub struct LibConfig {
    pub database: DatabaseConfig,
    pub general: GeneralConfig,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub pets_version: Option<u16>,
    pub foods_version: Option<u16>,
    pub tokens_version: Option<u16>,
    pub names_version: Option<u16>,
    pub filename: Option<String>,
    pub update_on_startup: bool,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    /// Storing battle data and building digraphs allows visualization of battle logic but causes a performance hit.
    /// ~ 225% increase in benchmarking times. (860 ns -> 2.7 us)
    pub build_graph: bool,
}
