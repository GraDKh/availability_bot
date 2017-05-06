use super::basic_structures::*;
use super::user_data::*;
use super::save_load_state::*;

use chrono;

use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

trait StateProcessor<State> {
    fn process(&self, state: 
               &mut State, 
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
            .process(state_holder.get_state(), answer, message_sender, events_sender);
    }
}

struct BotStateProcessor {
}

impl BotStateProcessor {
    fn new() -> Self {
        Self {}
    }
}

impl StateProcessor<UserInfo> for BotStateProcessor {
    fn process(&self, user_info: &mut UserInfo, answer: &str,
               message_sender: &mut MessageSender,
               events_sender: &mut EventsSender) {
        match user_info.state {
            BotState::Initial => {
                if answer.starts_with("/help") {
                    message_sender.send_text(user_info.chat_id, "https://www.youtube.com/watch?v=yWP6Qki8mWc".to_string());
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/whoami") {
                    {
                        let calendar_name = user_info.get_calendar_name().as_ref().map(|name| name.as_str())
                            .unwrap_or("<not specified>");
                        let message =
                            format!("{} {}\nIn calendar will be \"{}\"",
                                    user_info.get_first_name(),
                                    user_info.get_last_name(),
                                    calendar_name);
                        message_sender.send_text(user_info.chat_id, message);
                    }
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/setmyname") {
                    message_sender.send_text(user_info.chat_id, "Enter the name to be used in calendar".to_string());
                    user_info.state = BotState::SetName
                } else if answer.starts_with("/wfh") {
                    if user_info.get_calendar_name().is_some() {
                        message_sender.send_menu(user_info.chat_id, "Send work from home for today?".to_string(),
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
                            Some(name) => events_sender.post_wfh(name.as_str(),
                                                                 &chrono::Local::today()),
                            None => error!("WFH: User name for {:?} not specified", user_info)
                        }
                    }
                } else if answer == "no" {
                    message_sender.send_text(user_info.chat_id, "Canceled!".to_string());
                    user_info.state = BotState::Initial;
                }
            }
            BotState::SetName => {
                user_info.set_calendar_name(answer.to_string());
                message_sender.send_text(user_info.chat_id, format!("Your name will be \"{}\"", answer));
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
           data_saver: &'a mut DataSaver,) -> Self {
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
            UserCollectionSerializationData::new(self.last_message_id.expect("last_message_id not specified but save is invoked"), users);
        match self.data_saver.save_data(serialization_data) {
            Ok(_) => {},
            Err(error) => error!("Couldn't save bot state: {}", error.description())
        };
    }

    fn load(&mut self) {
        match self.data_saver.load_data() {
            Ok(user_data) => {
                     self.last_message_id = Some(user_data.last_id);
                     for user_info in user_data.users {
                         self.users
                             .insert(user_info.chat_id,
                                     User::new(user_info.info));
                     }
                 }
            Err(error) => warn!("Couldn't load bot state: {:?}", error.description())
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
                 None => "".to_string()
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
            user.state_machine.process(message, self.message_sender, self.events_sender);
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