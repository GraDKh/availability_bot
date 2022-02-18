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
pub struct WholeDayEvent {
    summary: String,
    start: CalendarDate,
    end: CalendarDate,
}

impl WholeDayEvent {
    pub fn new(text: String, start_date: &LocalDate, end_date: &LocalDate) -> Self {
        fn format_date(date: &LocalDate) -> String {
            date.format("%Y-%m-%d").to_string()
        }

        let start = CalendarDate { date: format_date(start_date) };
        let end = CalendarDate { date: format_date(end_date) };

        Self {
            summary: text,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalendarDateTime {
    #[serde(rename = "dateTime")]
    date_time: String,
}

pub type LocalDateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialDayEvent {
    summary: String,
    start: CalendarDateTime,
    end: CalendarDateTime,
}

impl PartialDayEvent {
    pub fn new(text: String, start_date: &LocalDateTime, end_date: &LocalDateTime) -> Self {
        fn format_date(date: &LocalDateTime) -> String {
            date.to_rfc3339()
        }

        let start = CalendarDateTime { date_time: format_date(start_date) };
        let end = CalendarDateTime { date_time: format_date(end_date) };

        Self {
            summary: text,
            start,
            end,
        }
    }
}

pub trait EventsSender {
    fn post_whole_day(&mut self, event: WholeDayEvent);
    fn post_partial_day(&mut self, event: PartialDayEvent);
}
