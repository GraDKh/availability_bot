use telegram_bot;
use serde_json;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;

pub type ChatID = telegram_bot::Integer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BotState {
    Initial,
    WfhStart,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub chat_id: ChatID,
    pub state: BotState,
    pub first_name: String,
    pub last_name: String
}

impl UserInfo {
    pub fn new(chat_id: ChatID, state: BotState, first_name: String, last_name: String) -> UserInfo {
        UserInfo {
            chat_id: chat_id,
            state: state,
            first_name: first_name,
            last_name: last_name
        }
    }

    pub fn get_calendar_name(&self) -> Option<String> {
        if self.first_name.len() == 0 || self.last_name.len() == 0 {
            None
        } else {
            Some(format!("{}.{}", &self.first_name[0..1], self.last_name))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSerializationInfo {
    pub chat_id: ChatID,
    pub info: UserInfo
}

impl UserSerializationInfo {
    pub fn new(chat_id: ChatID, info: UserInfo) -> Self {
        Self {chat_id: chat_id, info: info}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCollectionSerializationData {
    pub last_id: i64,
    pub users: Vec<UserSerializationInfo>
}

impl UserCollectionSerializationData {
    pub fn new(last_id: i64, user_infos: Vec<UserSerializationInfo>) -> Self {
        Self {last_id: last_id, users: user_infos}
    }
}

pub trait DataSaver {
    fn save_data(&self, data: UserCollectionSerializationData);
    fn load_data(&self) -> Option<UserCollectionSerializationData>;
}

struct FileDataSaver {
    path: String,
}

impl FileDataSaver {
    fn new(path: &str) -> Self {
        Self {path: path.to_string()}
    }
}

impl DataSaver for FileDataSaver {
    fn save_data(&self, data: UserCollectionSerializationData) {
        let file = OpenOptions::new()
            .write(true)
            .create(true).open(self.path.as_str()).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &data).unwrap();
    }

    fn load_data(&self) -> Option<UserCollectionSerializationData> {
        let file = OpenOptions::new()
            .read(true)
            .open(self.path.as_str());
        match file {
            Ok(opened_file) => {
                let reader = BufReader::new(opened_file);
                let user_data: Result<UserCollectionSerializationData, _> = serde_json::from_reader(reader);
                match user_data {
                    Ok(data) => Some(data),
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }
}

pub fn creat_data_saver(path: &str) -> Box<DataSaver> {
    Box::new(FileDataSaver::new(path))
}

