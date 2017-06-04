use super::basic_structures::*;
use super::user_data::*;
use super::save_load_state::*;

use chrono;

use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::collections::HashMap;
use std::mem;

use self::dialog_processing::{ReplyMessage, Dialog, DialogAction, Event, DialogInitializationResult};
use self::wfh::WfhDialog;
use self::simple_dialogs::{HelpDialog, WhoAmIDialog};

mod dialog_processing;
mod wfh;
mod simple_dialogs;

#[derive(Serialize, Deserialize)]
struct DialogsProcessor {
    active_dialog: Option<Box<Dialog>>
}

macro_rules! try_find_dialog {
    ($message:expr, $user_info:expr, $process_message:ident, $DialogType:ident $(, $DialogTypes:ident)*) => {{
        let dialog_init_result = $DialogType::make($message, $user_info);
        match dialog_init_result {
            DialogInitializationResult::NotProcessed => try_find_dialog!($message, $user_info, $process_message $(,$DialogTypes)*),
            DialogInitializationResult::Finished(reply, event) => { 
                $process_message(reply, event);
                None
            }
            DialogInitializationResult::StartedProcessing(reply, event, dialog) => {
                $process_message(reply, event);
                Some(dialog)
            }
        }
    }};

    ($message:expr, $user_info:expr, $process_message:ident) => {None};
}

impl DialogsProcessor {
    fn new() -> Self {
        Self {active_dialog: None}
    }

    fn process(&mut self,
               message: &str,
               user_info: &mut UserInfo,
               message_sender: &mut MessageSender,
               events_sender: &mut EventsSender) {
        let user_info = RefCell::from(user_info);
        let mut process_action = |reply: Option<ReplyMessage>, event: Option<Event>| {
            if let Some(reply) = reply {
                match reply.menu {
                            Some(menu) => message_sender.send_menu(user_info.borrow().chat_id, reply.text, menu),
                            None => message_sender.send_text(user_info.borrow().chat_id, reply.text)
                };
            };

            if let Some(event) = event {
                match event {
                    Event::WfhSingleDay(event) => events_sender.post_wfh(event)
                }
            }
        };
        
        let mut active_dialog : Option<Box<Dialog>> = None;
        mem::swap(&mut self.active_dialog, &mut active_dialog);
        self.active_dialog = match active_dialog {
            Some(mut dialog) => {
                let result = dialog.try_process(message, user_info.borrow_mut().deref_mut());
                match result {
                    DialogAction::ProcessAndContinue(reply, event) => {
                        process_action(reply, event);
                        Some(dialog)
                    }
                    DialogAction::ProcessAndStop(reply, event) => {
                        process_action(reply, event);
                        None
                    }
                    DialogAction::Stop => None
                }
            }
            None => {
                try_find_dialog![message, user_info.borrow_mut().deref_mut(), process_action, 
                                 WfhDialog, HelpDialog, WhoAmIDialog]
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct UserState {
    user_info: UserInfo,
    dialogs_processor: DialogsProcessor
}

impl UserState {
    fn new() {
        
    }
}

trait StateProcessor<State> {
    fn process(&self,
               state: &mut State,
               answer: &str,
               message_sender: &mut MessageSender,
               events_sender: &mut EventsSender);
}

trait StateHolder<State> {
    fn get_state(&mut self) -> &mut State;
}

struct DialogStateMachine<State, Processor: StateProcessor<State>> {
    state_holder: Rc<RefCell<StateHolder<State>>>,
    processor: Box<Processor>,
}

impl<State, Processor: StateProcessor<State>> DialogStateMachine<State, Processor> {
    fn new(state_holder: Rc<RefCell<StateHolder<State>>>,
           processor: Box<Processor>)
           -> DialogStateMachine<State, Processor> {
        DialogStateMachine::<State, Processor> {
            state_holder: state_holder,
            processor: processor,
        }
    }

    fn process(&mut self,
               answer: &str,
               message_sender: &mut MessageSender,
               events_sender: &mut EventsSender) {
        let state_holder = &mut self.state_holder.deref().borrow_mut();
        self.processor
            .deref()
            .process(state_holder.get_state(),
                     answer,
                     message_sender,
                     events_sender);
    }
}

struct BotStateProcessor {}

impl BotStateProcessor {
    fn new() -> Self {
        Self {}
    }
}

impl StateProcessor<UserInfo> for BotStateProcessor {
    fn process(&self,
               user_info: &mut UserInfo,
               answer: &str,
               message_sender: &mut MessageSender,
               events_sender: &mut EventsSender) {
        match user_info.state {
            BotState::Initial => {
                if answer.starts_with("/help") {
                    message_sender.send_text(user_info.chat_id,
                                             "https://www.youtube.com/watch?v=yWP6Qki8mWc"
                                                 .to_string());
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/whoami") {
                    {
                        let calendar_name = user_info
                            .get_calendar_name()
                            .as_ref()
                            .map(|name| name.as_str())
                            .unwrap_or("<not specified>");
                        let message = format!("{} {}\nIn calendar will be \"{}\"",
                                              user_info.get_first_name(),
                                              user_info.get_last_name(),
                                              calendar_name);
                        message_sender.send_text(user_info.chat_id, message);
                    }
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/setmyname") {
                    message_sender.send_text(user_info.chat_id,
                                             "Enter the name to be used in calendar".to_string());
                    user_info.state = BotState::SetName
                } else if answer.starts_with("/wfh") {
                    if user_info.get_calendar_name().is_some() {
                        message_sender.send_menu(user_info.chat_id,
                                                 "Send work from home for today?".to_string(),
                                                 vec![vec!["yes".to_string(), "no".to_string()]]);
                        user_info.state = BotState::WfhConfirmation;
                    }
                }
            }
            BotState::WfhConfirmation => {
                if answer == "yes" {
                    message_sender.send_text(user_info.chat_id, "Applied!".to_string());
                    user_info.state = BotState::Initial;
                    {
                        match user_info.get_calendar_name() {
                            Some(name) => {
                                events_sender.post_wfh(WfhSingleDay::new(name.as_str(), &chrono::Local::today()))
                            }
                            None => error!("WFH: User name for {:?} not specified", user_info),
                        }
                    }
                } else if answer == "no" {
                    message_sender.send_text(user_info.chat_id, "Canceled!".to_string());
                    user_info.state = BotState::Initial;
                }
            }
            BotState::SetName => {
                user_info.set_calendar_name(answer.to_string());
                message_sender.send_text(user_info.chat_id,
                                         format!("Your name will be \"{}\"", answer));
                user_info.state = BotState::Initial;
            }
        }
    }
}

type StateMachine = DialogStateMachine<UserInfo, BotStateProcessor>;

impl StateHolder<UserInfo> for UserInfo {
    fn get_state(self: &mut Self) -> &mut UserInfo {
        self
    }
}

struct User {
    info: Rc<RefCell<UserInfo>>,
    state_machine: StateMachine,
}

impl User {
    fn new(info: UserInfo) -> Self {
        let info = Rc::new(RefCell::from(info));
        Self {
            info: info.clone(),
            state_machine: StateMachine::new(info, Box::new(BotStateProcessor::new())),
        }
    }
}

pub struct UserCollection<'a> {
    message_sender: &'a mut MessageSender,
    events_sender: &'a mut EventsSender,
    data_saver: &'a mut DataSaver,
    users: HashMap<ChatID, User>,
    last_message_id: Option<i64>,
}

impl<'a> UserCollection<'a> {
    pub fn new(message_sender: &'a mut MessageSender,
               events_sender: &'a mut EventsSender,
               data_saver: &'a mut DataSaver)
               -> Self {
        let mut result = Self {
            message_sender,
            events_sender,
            data_saver,
            users: HashMap::<ChatID, User>::new(),
            last_message_id: None,
        };

        result.load();

        result
    }

    fn save(&self) {
        let users: Vec<_> = self.users
            .iter()
            .map(|(id, user)| {
                     UserSerializationInfo::new(id.clone(), user.info.deref().borrow().clone())
                 })
            .collect();
        let serialization_data =
            UserCollectionSerializationData::new(
                self.last_message_id.expect("last_message_id not specified but save is invoked"), users);
        match self.data_saver.save_data(serialization_data) {
            Ok(_) => {}
            Err(error) => error!("Couldn't save bot state: {}", error.description()),
        };
    }

    fn load(&mut self) {
        match self.data_saver.load_data() {
            Ok(user_data) => {
                self.last_message_id = Some(user_data.last_id);
                for user_info in user_data.users {
                    self.users
                        .insert(user_info.chat_id, User::new(user_info.info));
                }
            }
            Err(error) => warn!("Couldn't load bot state: {:?}", error.description()),
        }
    }
}

impl<'a> MessageProcessor for UserCollection<'a> {
    fn process_message(&mut self,
                       chat_id: ChatID,
                       first_name: &str,
                       last_name: Option<&str>,
                       message: &str) {
        let get_name = || {
            (first_name.to_string(),
             match last_name {
                 Some(name) => name.to_string(),
                 None => "".to_string(),
             })
        };

        {
            let mut user = self.users
                .entry(chat_id)
                .or_insert_with(|| {
                                    let (first_name, last_name) = get_name();
                                    User::new(UserInfo::new(chat_id,
                                                            BotState::Initial,
                                                            first_name,
                                                            last_name))
                                });
            user.state_machine
                .process(message, self.message_sender, self.events_sender);
        }

        self.save();
    }

    fn is_new_message(&mut self, message_id: i64) -> bool {
        return match self.last_message_id {
                   Some(id) => {
                       if message_id <= id {
                           false
                       } else {
                           self.last_message_id = Some(message_id);
                           true
                       }
                   }
                   None => {
            self.last_message_id = Some(message_id);
            true
        }
               };
    }
}

#[cfg(test)]
mod tests {

use ::basic_structures::*;
use ::user_data::*;
use ::message_processor::*;
use ::save_load_state::{DataSaver, SaveResult, LoadResult, UserCollectionSerializationData, UserSerializationInfo};

use std::cell::Cell;
use std::ops::Deref;

enum Event {
    WfhSingleDay{name: String, date: LocalDate},
}

struct MockEventsSender {
    events: Vec<Event>
}

impl MockEventsSender {
    fn new() -> Self {
        Self {events: Vec::<Event>::new()}
    }
}

impl EventsSender for MockEventsSender {
    fn post_wfh(&mut self, name: &str, date: &LocalDate) {
        self.events.push(Event::WfhSingleDay{name: name.to_string(), date: date.clone()});
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Message {
    chat_id: ChatID,
    text: String,
    menu: Option<Menu>
}

impl Message {
    fn new<S>(chat_id: ChatID, text: S, menu: Option<Menu>) -> Self
        where S: Into<String> {
        Self {chat_id, text: text.into(), menu}
    }
}

struct MockMessageSender {
    messages: Vec<Message>
}

impl MockMessageSender {
    fn new() -> Self {
        Self {messages: Vec::<Message>::new()}
    }
}

impl MessageSender for MockMessageSender {
    fn send_text(&mut self, chat_id: ChatID, text: String) {
        self.messages.push(Message::new(chat_id, text, None));
    }

    fn send_menu(&mut self, chat_id: ChatID, text: String, menu: Menu) {
        self.messages.push(Message::new(chat_id, text, Some(menu)));
    }
}

struct MockDataSaver {
    save_count: Cell<i32>,
    load_count: Cell<i32>
}

impl MockDataSaver {
    fn new() -> Self {
        Self{save_count: Cell::new(0), load_count: Cell::new(0)}
    }
}

impl DataSaver for MockDataSaver {
    fn save_data(&self, data: UserCollectionSerializationData) -> SaveResult {
        self.save_count.set(self.save_count.get() + 1);
        Ok(())
    }

    fn load_data(&self) -> LoadResult {
        self.load_count.set(self.load_count.get() + 1);
        Ok(UserCollectionSerializationData::new(42, Vec::<UserSerializationInfo>::new()))
    }
}

fn simple_reply_test(first_name: &str, last_name: Option<&str>, text: &str, expected_reply: &str) {
    let mut message_sender = MockMessageSender::new();
    let mut events_sender = MockEventsSender::new();
    let mut data_saver = MockDataSaver::new();
    {
        let mut message_processor = UserCollection::new(&mut message_sender,
                                                    &mut events_sender,
                                                    &mut data_saver);
        
        message_processor.process_message(42, first_name, last_name, text);
    }
    assert_eq!(events_sender.events.len(), 0);
    assert_eq!(data_saver.save_count, Cell::new(1));
    assert_eq!(message_sender.messages, vec![Message::new(42, expected_reply, None)]);
}

macro_rules! define_send_reply_test {
    ($first_name:expr, $last_name:expr, $([$request:expr, $reply:expr]),+) => (
        let mut message_sender = MockMessageSender::new();
        let mut events_sender = MockEventsSender::new();
        let mut data_saver = MockDataSaver::new();
        {
            let mut message_processor = super::UserCollection::new(
                &mut message_sender,
                &mut events_sender,
                &mut data_saver);
            
            $(message_processor.process_message(42, $first_name, $last_name, $request);)*
        }

        assert_eq!(events_sender.events.len(), 0);

        let messages = vec![$(Message::new(42, $reply, None), )*];
        assert_eq!(data_saver.save_count, Cell::new(messages.len() as i32));
        assert_eq!(message_sender.messages, messages);
    )
}

#[test]
fn test_help() {
    define_send_reply_test!("Vasiliy", None, ["/help", "https://www.youtube.com/watch?v=yWP6Qki8mWc"]);
}

#[test]
fn test_whoami() {
    define_send_reply_test!("Vasiliy", None, ["/whoami", "Vasiliy \nIn calendar will be \"<not specified>\""]);
    define_send_reply_test!("Vasiliy", Some("Pupkin"), ["/whoami", "Vasiliy Pupkin\nIn calendar will be \"V.Pupkin\""]);
}

#[test]
fn test_setmyname() {
    define_send_reply_test!("Vasiliy", None, 
        ["/whoami", "Vasiliy \nIn calendar will be \"<not specified>\""],
        ["/setmyname", "Enter the name to be used in calendar"],
        ["A.Crowley", "Your name will be \"A.Crowley\""],
        ["/whoami", "Vasiliy \nIn calendar will be \"A.Crowley\""]);
}

}
