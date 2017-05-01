use super::user_data;

use serde_json;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSerializationInfo {
    pub chat_id: user_data::ChatID,
    pub info: user_data::UserInfo
}

impl UserSerializationInfo {
    pub fn new(chat_id: user_data::ChatID, info: user_data::UserInfo) -> Self {
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

