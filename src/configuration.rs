use serde_json;

use std::fs::OpenOptions;
use std::io::BufReader;

static CONGIGURATION_FILE: &'static str = "configuration.json";
static BOT_TOKEN: &'static str = "305992740:AAFLC-zkocg7inSmaaIIydeFW6Gs6aBu2Go";
static DATA_FILE: &'static str = "data.json";

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub bot_token: String,
    pub data_file: String
}

impl Configuration {
    pub fn load() -> Configuration {
        let file = OpenOptions::new()
            .read(true)
            .open(CONGIGURATION_FILE);
        match file {
            Ok(opened_file) => {
                let reader = BufReader::new(opened_file);
                let configuration: Result<Configuration, _> = serde_json::from_reader(reader);
                match configuration {
                    Ok(conf) => conf,
                    Err(err) => { 
                        error!("Couldn't deserealize config: {:?}", err);
                        Default::default() }
                }
            }
            Err(err) => {
                warn!("Coludn't open configuration file {:?}", err);
                Default::default() 
            }
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {bot_token: BOT_TOKEN.to_string(), data_file: DATA_FILE.to_string()}
    }
}