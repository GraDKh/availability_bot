pub type ChatID = super::telegram_bot::ChatId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserName {
    first_name: String,
    last_name: String,
    calendar_name: Option<String>,
}

impl UserName {
    fn new(first_name: String, last_name: String) -> Self {
        let mut result = Self {
            first_name: first_name,
            last_name: last_name,
            calendar_name: None,
        };
        result.calendar_name = UserName::calculate_calendar_name(&result.first_name,
                                                                 &result.last_name);
        result
    }

    fn calculate_calendar_name(first_name: &String, last_name: &String) -> Option<String> {
        if first_name.len() == 0 || last_name.len() == 0 {
            None
        } else {
            Some(format!("{}.{}", &first_name[0..1], last_name))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub chat_id: ChatID,
    pub name: UserName,
}

impl UserInfo {
    pub fn new(chat_id: ChatID, first_name: String, last_name: String) -> Self {
        Self {
            chat_id,
            name: UserName::new(first_name, last_name),
        }
    }

    pub fn get_calendar_name(&self) -> Option<&String> {
        self.name.calendar_name.as_ref()
    }

    pub fn get_first_name(&self) -> &String {
        &self.name.first_name
    }

    pub fn get_last_name(&self) -> &String {
        &self.name.last_name
    }

    pub fn set_calendar_name(&mut self, calendar_name: String) {
        self.name.calendar_name = Some(calendar_name)
    }
}
