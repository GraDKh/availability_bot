pub type ChatID = super::telegram_bot::Integer;

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