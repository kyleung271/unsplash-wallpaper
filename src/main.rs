use env_logger::try_init_from_env;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error::Error;
use wallpaper::set_from_url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Landscape,
    Portrait,
    Squarish,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct QueryParams<'a> {
    collections: Option<u32>,
    query: Option<Cow<'a, str>>,
    featured: bool,
    orientation: Orientation,
}

impl<'a> Default for QueryParams<'a> {
    fn default() -> Self {
        Self {
            collections: None,
            query: Some(("wallpaper").into()),
            featured: true,
            orientation: Orientation::Landscape,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct DownloadParams<'a> {
    #[serde(alias = "width")]
    w: u32,
    #[serde(alias = "height")]
    h: u32,
    #[serde(alias = "format")]
    fm: Cow<'a, str>,
    fit: Cow<'a, str>,
    crop: Option<Cow<'a, str>>,
    dpr: f64,
}

impl<'a> Default for DownloadParams<'a> {
    fn default() -> Self {
        Self {
            w: 1920,
            h: 1080,
            fm: ("jpg").into(),
            fit: ("crop").into(),
            crop: Some(("entropy").into()),
            dpr: 1.,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    urls: Urls,
}

#[derive(Debug, Deserialize)]
struct Urls {
    raw: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Config<'a> {
    key: String,
    query: QueryParams<'a>,
    download: DownloadParams<'a>,
}

impl<'a> Config<'a> {
    pub fn try_init() -> Result<Self, Box<dyn Error>> {
        let mut config = config::Config::new();
        config.merge(config::File::with_name("unsplash").required(false))?;
        config.merge(config::Environment::with_prefix("UNSPLASH").separator("_"))?;

        let config: Config = config.try_into()?;
        Ok(config)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    try_init_from_env("UNSPLASH_LOG")?;
    let config = Config::try_init()?;

    info!("{:?}", config);

    let client = Client::new();

    let req = client
        .get("https://api.unsplash.com/photos/random")
        .header("Authorization", format!("Client-ID {}", config.key))
        .header("Accept-Version", "v1")
        .query(&config.query);

    info!("{:?}", req);

    let res = req.send()?;

    info!("{:?}", res);

    let res: Response = res.error_for_status()?.json()?;

    let req = client
        .get(&res.urls.raw)
        .query(&config.download)
        .build()?;
    let url = req.url().as_str();

    info!("Download from: {:?}", url);

    set_from_url(url)?;

    Ok(())
}
