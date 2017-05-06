use super::user_data::ChatID;

use chrono;

pub trait MessageProcessor {
    fn is_new_message(&mut self, message_id: i64) -> bool;

    fn process_message(&mut self,
                       chat_id: ChatID,
                       first_name: &str,
                       last_name: Option<&str>,
                       message: &str);
}

pub trait MessageSender {
    fn send_text(&mut self, chat_id: ChatID, text: String);
    fn send_menu(&mut self,
                 chat_id: ChatID,
                 text: String,
                 menu: Vec<Vec<String>>);
}

pub trait EventsSender {
    fn post_wfh(&mut self,
                name: &str,
                date: &chrono::Date<chrono::Local>);
}