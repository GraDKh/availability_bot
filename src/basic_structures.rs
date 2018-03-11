use super::user_data::ChatID;

use chrono;

pub trait MessageProcessor {
    fn is_new_message(&mut self, message_id: i64) -> bool;

    fn process_message(&mut self,
                       message_sender: &mut MessageSender,
                       chat_id: ChatID,
                       first_name: &str,
                       last_name: Option<&str>,
                       message: &str);
}

pub type Menu = Vec<Vec<String>>;

pub trait MessageSender {
    fn send_text(&mut self, chat_id: ChatID, text: String);
    fn send_menu(&mut self, chat_id: ChatID, text: String, menu: Menu);
    fn send_status_to_channel(&mut self, text: String);
}

pub type LocalDate = chrono::Date<chrono::Local>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalendarDate {
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WfhSingleDay {
    summary: String,
    start: CalendarDate,
    end: CalendarDate,
}

impl WfhSingleDay {
    pub fn new(name: &str, date: &LocalDate) -> Self {
        let date = date.format("%Y-%m-%d").to_string();
        let start = CalendarDate { date };
        let end = start.clone();

        Self {
            summary: format!("WFH {}", name),
            start,
            end,
        }
    }
}

pub trait EventsSender {
    fn post_wfh(&mut self, event: WfhSingleDay);
}
