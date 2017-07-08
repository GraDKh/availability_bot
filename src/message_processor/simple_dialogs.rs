use message_processor::dialog_processing::{DialogAction, ReplyMessage, DialogInitializationResult,
                                           DynamicSerializable, StaticNameGetter, Dialog, Event};
use user_data::UserInfo;

use serde_json;

pub trait SimpleDialog {
    fn process_message(message: &str,
                       user_info: &mut UserInfo)
                       -> Option<(Option<ReplyMessage>, Option<Event>)>
        where Self: Sized;
}

impl<T> Dialog for T
    where T: 'static + SimpleDialog + DynamicSerializable + Clone + Sized
{
    fn try_process(&mut self, _: &str, _: &mut UserInfo) -> DialogAction {
        panic!("Simple dialog should contain single action");
    }

    fn make(initial_message: &str, user_info: &mut UserInfo) -> DialogInitializationResult
        where Self: Sized
    {
        match Self::process_message(initial_message, user_info) {
            Some((reply, event)) => DialogInitializationResult::Finished(reply, event),
            None => DialogInitializationResult::NotProcessed,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelpDialog {}

impl SimpleDialog for HelpDialog {
    fn process_message(message: &str,
                       _: &mut UserInfo)
                       -> Option<(Option<ReplyMessage>, Option<Event>)> {
        if message.starts_with("/help") {
            Some((Some(ReplyMessage::new("https://www.youtube.com/watch?v=yWP6Qki8mWc", None)),
                  None))
        } else {
            None
        }
    }
}

impl DynamicSerializable for HelpDialog {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap() // FIXME
    }

    fn from_string(string: &str) -> Self {
        serde_json::from_str::<Self>(string).unwrap() // FIXME
    }
}

impl StaticNameGetter for HelpDialog {
    fn get_name() -> &'static str {
        return "help-dialog";
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WhoAmIDialog {}

impl SimpleDialog for WhoAmIDialog {
    fn process_message(message: &str,
                       user_info: &mut UserInfo)
                       -> Option<(Option<ReplyMessage>, Option<Event>)> {
        if message.starts_with("/whoami") {
            let calendar_name = user_info
                .get_calendar_name()
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or("<not specified>");
            let message = format!("{} {}\nIn calendar will be \"{}\"",
                                  user_info.get_first_name(),
                                  user_info.get_last_name(),
                                  calendar_name);
            Some((Some(ReplyMessage::new(message, None)), None))
        } else {
            None
        }
    }
}

impl DynamicSerializable for WhoAmIDialog {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap() // FIXME
    }

    fn from_string(string: &str) -> Self {
        serde_json::from_str::<Self>(string).unwrap() // FIXME
    }
}

impl StaticNameGetter for WhoAmIDialog {
    fn get_name() -> &'static str {
        return "whoami-dialog";
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetMyNameDialog {}

static SET_MY_NAME_PREFIX: &str = "/setmyname";

fn parse_name<'a>(message: &'a str) -> Option<&'a str> {
    let name = (&message[SET_MY_NAME_PREFIX.len()..]).trim();
    // TODO: use regexp
    if name.len() > 0 { Some(name) } else { None }
}

impl SimpleDialog for SetMyNameDialog {
    fn process_message(message: &str,
                       user_info: &mut UserInfo)
                       -> Option<(Option<ReplyMessage>, Option<Event>)> {
        if message.starts_with(SET_MY_NAME_PREFIX) {
            let reply_message =
                match parse_name(message) {
                    Some(name) => {
                        user_info.set_calendar_name(name.to_string());
                        format!("Your calendar name will be \"{}\"", name)
                    }
                    None => "No valid name is specified. Please specify name in format \"/setmyname J.Doe\"".to_string(),
                };

            Some((Some(ReplyMessage::new(reply_message, None)), None))
        } else {
            None
        }
    }
}

impl DynamicSerializable for SetMyNameDialog {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap() // FIXME
    }

    fn from_string(string: &str) -> Self {
        serde_json::from_str::<Self>(string).unwrap() // FIXME
    }
}

impl StaticNameGetter for SetMyNameDialog {
    fn get_name() -> &'static str {
        return "setmyname-dialog";
    }
}
