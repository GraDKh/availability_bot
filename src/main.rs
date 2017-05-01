#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate telegram_bot;
#[macro_use]
extern crate mime;
extern crate hyper;
extern crate hyper_rustls;
extern crate url;

mod user_data;
mod save_load_state;
mod post_to_form;

use user_data::ChatID;
use user_data::BotState;
use user_data::UserInfo;
use save_load_state::UserSerializationInfo;
use save_load_state::UserCollectionSerializationData;
use telegram_bot::{Api, MessageType, ListeningMethod, ListeningAction, Chat, ReplyMarkup};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::Box;
use std::ops::Deref;
use std::ops::DerefMut;

static BOT_TOKEN: &'static str = "305992740:AAFLC-zkocg7inSmaaIIydeFW6Gs6aBu2Go";

trait StateProcessor<State> {
    fn process(&self, state: &mut State, answer: &str);
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

    fn process(&mut self, answer: &str) {
        let state_holder = &mut self.state_holder.deref().borrow_mut();
        self.processor
            .deref()
            .process(state_holder.get_state(), answer);
    }
}

struct BotStateProcessor {
    api: Rc<Api>,
    chat_id: ChatID,
}

impl<'a> BotStateProcessor {
    fn new(api: Rc<Api>, chat_id: ChatID) -> BotStateProcessor {
        BotStateProcessor {
            api: api,
            chat_id: chat_id,
        }
    }

    fn send_text(&self, text: String) {
        self.api
            .send_message(self.chat_id, text, None, None, None, None)
            .unwrap();
    }

    fn send_menu(&self, menu: Vec<Vec<String>>) {
        let reply_markup = ReplyMarkup::Keyboard(telegram_bot::ReplyKeyboardMarkup {
                                                     keyboard: menu,
                                                     one_time_keyboard: Some(true),
                                                     selective: Some(true),
                                                     ..Default::default()
                                                 });
        self.api
            .send_message(self.chat_id,
                          "choose:".to_string(),
                          None,
                          None,
                          None,
                          Some(reply_markup))
            .unwrap();
    }
}

impl StateProcessor<UserInfo> for BotStateProcessor {
    fn process(&self, user_info: &mut UserInfo, answer: &str) {
        match user_info.state {
            BotState::Initial => {
                if answer.starts_with("/help") {
                    self.send_text("No help".to_string());
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/whoami") {
                    let message =
                        format!("{} {}\nIn calendar will be \"{}\"",
                                user_info.first_name,
                                user_info.last_name,
                                user_info.get_calendar_name().unwrap_or("-".to_string()));
                    self.send_text(message);
                    user_info.state = BotState::Initial;
                } else if answer.starts_with("/wfh") {
                    self.send_menu(vec![vec!["yes".to_string(), "no".to_string()]]);
                    user_info.state = BotState::WfhStart;
                }
            }
            BotState::WfhStart => {
                if answer == "yes" {
                    self.send_text("Applied!".to_string());
                    user_info.state = BotState::Initial;
                    post_to_form::post_wfh(user_info.get_calendar_name().unwrap().as_str());
                } else if answer == "no" {
                    self.send_text("Canceled!".to_string());
                    user_info.state = BotState::Initial;
                }
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
    fn new(info: UserInfo, api: Rc<Api>) -> Self {
        let chat_id = info.chat_id;
        let info = Rc::new(RefCell::from(info));
        Self {
            info: info.clone(),
            state_machine: StateMachine::new(info, Box::new(BotStateProcessor::new(api, chat_id))),
        }
    }
}

struct UserCollection {
    api: Rc<Api>,
    users: HashMap<ChatID, User>,
    last_message_id: Option<i64>,
}

impl UserCollection {
    fn new(api: Rc<Api>) -> Self {
        Self {
            api: api,
            users: HashMap::<ChatID, User>::new(),
            last_message_id: None,
        }
    }

    fn get_or_create_user<H>(&mut self, chat_id: ChatID, name_getter: &H) -> &mut User
        where H: Fn() -> (String, String)
    {
        let api = self.api.clone();
        self.users
            .entry(chat_id)
            .or_insert_with(|| {
                                let (first_name, last_name) = name_getter();
                                User::new(UserInfo::new(chat_id,
                                                        BotState::Initial,
                                                        first_name,
                                                        last_name),
                                          api)
                            })
    }

    fn on_new_message(&mut self, message_id: i64) -> bool {
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

    fn save(&self, saver: &save_load_state::DataSaver) {
        let users: Vec<_> = self.users
            .iter()
            .map(|(id, user)| {
                     UserSerializationInfo::new(id.clone(), user.info.deref().borrow().clone())
                 })
            .collect();
        let serialization_data =
            UserCollectionSerializationData::new(self.last_message_id.unwrap(), users);
        saver.save_data(serialization_data);
    }

    fn load(&mut self, loader: &save_load_state::DataSaver) {
        loader
            .load_data()
            .map(|user_data| {
                     self.last_message_id = Some(user_data.last_id);
                     for user_info in user_data.users {
                         self.users
                             .insert(user_info.chat_id,
                                     User::new(user_info.info, self.api.clone()));
                     }
                 });
    }
}

const DATA_FILE: &'static str = "data.json";

fn main() {
    let api = Rc::new(Api::from_token(BOT_TOKEN).unwrap());

    println!("getMe: {:?}", api.get_me());

    let users = RefCell::new(UserCollection::new(api.clone()));
    let data_saver = save_load_state::creat_data_saver(DATA_FILE);
    users.borrow_mut().deref_mut().load(&*data_saver);
    let mut listener = api.deref().listener(ListeningMethod::LongPoll(None));

    listener
        .listen(|update| {
            println!("Got message: {:?}", update);

            update
                .message
                .as_ref()
                .map(|ref message| {
                    match message.msg {
                        MessageType::Text(ref text) => {

                            let mut users_mut = users.borrow_mut();
                            let mut users_ref = users_mut.deref_mut();
                            if users_ref.on_new_message(update.update_id) {
                                let chat_id = match message.chat {
                                    Chat::Private { id, .. } => id,
                                    Chat::Group { id, .. } => id,
                                    Chat::Channel { id, .. } => id,
                                };

                                let get_name = || {
                                    (message.from.first_name.clone(),
                                     message
                                         .from
                                         .last_name
                                         .clone()
                                         .unwrap_or("".to_string()))
                                };

                                {
                                    let mut user = users_ref.get_or_create_user(chat_id, &get_name);
                                    user.state_machine.process(text);
                                }

                                users_ref.save(&*data_saver);
                            }
                        }
                        _ => {}
                    };
                });

            Result::Ok(ListeningAction::Continue)
        })
        .unwrap();
}
