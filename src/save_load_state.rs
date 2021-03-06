use super::user_data::{ChatID, UserInfo};
use super::message_processor::DialogsProcessor;

use serde_json;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct UserSerializationInfo {
    pub chat_id: ChatID,
    pub info: UserInfo,
    pub processor: DialogsProcessor
}

impl UserSerializationInfo {
    pub fn new(chat_id: ChatID, info: UserInfo, processor: DialogsProcessor) -> Self {
        Self {chat_id, info, processor}
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserCollectionSerializationData {
    pub last_id: i64,
    pub users: Vec<UserSerializationInfo>,
}

impl UserCollectionSerializationData {
    pub fn new(last_id: i64, user_infos: Vec<UserSerializationInfo>) -> Self {
        Self {
            last_id: last_id,
            users: user_infos,
        }
    }
}

pub type SaveResult = Result<(), Box<Error>>;
pub type LoadResult = Result<UserCollectionSerializationData, Box<Error>>;

pub trait DataSaver {
    fn save_data(&self, data: UserCollectionSerializationData) -> SaveResult;
    fn load_data(&self) -> LoadResult;
}

pub struct FileDataSaver {
    path: String,
}

impl FileDataSaver {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }
}

impl DataSaver for FileDataSaver {
    fn save_data(&self, data: UserCollectionSerializationData) -> SaveResult {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.path.as_str())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &data)?;
        Ok(())
    }

    fn load_data(&self) -> LoadResult {
        let file = OpenOptions::new().read(true).open(self.path.as_str())?;
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(data) => Ok(data),
            Err(error) => Err(Box::new(error)),
        }
    }
}
