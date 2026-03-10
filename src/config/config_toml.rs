use std::{env, fs};

use opensubs::Language;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ConfigToml {
    pub yts: Option<Yts>,
    pub opensubs: Option<Opensubs>,
    pub transmission: Option<Transmission>,
}

#[derive(Deserialize, Debug)]
struct Transmission {
    pub host: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Yts {
    pub host: Option<String>,
    pub download_dir: Option<String>,
    pub order: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Opensubs {
    pub languages: Vec<String>,
    pub order: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub yts_host: String,
    pub yts_download_dir: String,
    pub yts_order: yts::OrderBy,
    pub opensubs_langs: Vec<Language>,
    pub opensubs_order: opensubs::OrderBy,
    pub transmission_host: String,
    pub transmission_username: Option<String>,
    pub transmission_password: Option<String>,
}

impl From<ConfigToml> for Config {
    fn from(value: ConfigToml) -> Self {
        let mut config = Config::default();

        if let Some(yts) = value.yts {
            if let Some(host) = yts.host {
                config.yts_host = host;
            }
            if let Some(download_dir) = yts.download_dir {
                config.yts_download_dir = download_dir;
            }
            if let Some(order) = yts.order {
                config.yts_order = yts::OrderBy::try_from(order.as_str())
                    .unwrap_or_else(|_| panic!("Failed to convert '{order}' to YTS Order"));
            }
        }

        if let Some(opensubs) = value.opensubs {
            config.opensubs_langs = opensubs
                .languages
                .iter()
                .map(|lang| {
                    opensubs::Language::try_from(lang.as_str()).unwrap_or_else(|_| {
                        panic!("Failed to convert '{lang}' to Opensubs Language")
                    })
                })
                .collect();

            if let Some(order) = opensubs.order {
                config.opensubs_order = match order.to_lowercase().as_str() {
                    "downloads" => opensubs::OrderBy::Downloads,
                    "rating" => opensubs::OrderBy::Rating,
                    "uploaded" => opensubs::OrderBy::Uploaded,
                    _ => panic!("Failed to convert '{order}' to Opensubs Order"),
                }
            }
        }

        if let Some(transmission) = value.transmission {
            if let Some(host) = transmission.host {
                config.transmission_host = host;
            }
            config.transmission_username = transmission.username;
            config.transmission_password = transmission.password;
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            yts_host: "https://yts.bz".to_string(),
            yts_download_dir: format!(
                "{}/Downloads",
                env::var_os("HOME")
                    .expect("No HOME variable set.")
                    .to_str()
                    .expect("Error converting HOME var to string")
            ),
            yts_order: yts::OrderBy::Rating,
            opensubs_langs: vec![Language::Spanish],
            opensubs_order: opensubs::OrderBy::Downloads,
            transmission_host: "http://127.0.0.1:9091/transmission/rpc".to_string(),
            transmission_username: None,
            transmission_password: None,
        }
    }
}

pub fn configuration() -> anyhow::Result<Config> {
    let home_path = env::var_os("HOME").expect("No HOME variable set.");

    let config_path = format!(
        "{}{}",
        home_path.to_string_lossy(),
        "/.config/yts-cli/config.toml"
    );

    if let Ok(toml) = fs::read_to_string(config_path) {
        let config_toml: ConfigToml = toml::from_str(&toml)?;
        Ok(config_toml.into())
    } else {
        Ok(Config::default())
    }
}
