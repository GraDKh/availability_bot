extern crate telegram_bot;
extern crate hyper;
extern crate hyper_rustls;

use telegram_bot::{Api, MessageType, ListeningMethod, ListeningAction, Chat, ReplyMarkup};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::Box;
use std::ops::Deref;
use std::ops::DerefMut;

static BOT_TOKEN: &'static str = "305992740:AAFLC-zkocg7inSmaaIIydeFW6Gs6aBu2Go";

type ChatID = telegram_bot::Integer;

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
        let state = self.processor
            .deref()
            .process(state_holder.get_state(), answer);
    }
}

#[derive(Clone)]
enum BotState {
    Initial,
    WfhStart,
}

struct BotStateProcessor {
    api: Rc<Api>,
    chat_id: ChatID,
}

struct UserInfo {
    chat_id: ChatID,
    state: BotState,
    first_name: String,
    last_name: String
}

impl UserInfo {
    fn new(chat_id: ChatID, state: BotState, first_name: String, last_name: String) -> UserInfo {
        UserInfo {
            chat_id: chat_id,
            state: state,
            first_name: first_name,
            last_name: last_name
        }
    }


fn get_calendar_name(&self) -> Option<String> {
    if self.first_name.len() == 0 || self.last_name.len() == 0 {
        None
    } else {
        Some(format!("{}.{}", &self.first_name[0..1], self.last_name))
    }
}
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
    fn process(&self, userInfo: &mut UserInfo, answer: &str) {
        match userInfo.state {
            BotState::Initial => {
                if answer.starts_with("/help") {
                    self.send_text("No help".to_string());
                    userInfo.state = BotState::Initial;
                } else if answer.starts_with("/whoami") {
                    let message = format!("{} {}\nIn calendar will be \"{}\"",
                                          userInfo.first_name, userInfo.last_name,
                                          userInfo.get_calendar_name().unwrap_or("-".to_string()));
                    self.send_text(message);
                    userInfo.state = BotState::Initial;
                } else if answer.starts_with("/wfh") {
                    self.send_menu(vec![vec!["yes".to_string(), "no".to_string()]]);
                    userInfo.state = BotState::WfhStart;
                }
            }
            BotState::WfhStart => {
                if answer == "yes" {
                    self.send_text("Applied!".to_string());
                    userInfo.state = BotState::Initial;
                } else if answer == "no" {
                    self.send_text("Canceled!".to_string());
                    userInfo.state = BotState::Initial;
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
    fn new(info: UserInfo, api: Rc<Api>) -> User {
        let chat_id = info.chat_id;
        let info = Rc::new(RefCell::from(info));
        User {
            info: info.clone(),
            state_machine: StateMachine::new(info, Box::new(BotStateProcessor::new(api, chat_id))),
        }
    }
}

struct UserCollection {
    api: Rc<Api>,
    users: HashMap<ChatID, User>,
}

impl UserCollection {
    fn new(api: Rc<Api>) -> UserCollection {
        UserCollection {
            api: api,
            users: HashMap::<ChatID, User>::new(),
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
                                User::new(UserInfo::new(chat_id, BotState::Initial, first_name, last_name),
                                          api)
                            })
    }
}

fn main() {
    let api = Rc::new(Api::from_token(BOT_TOKEN).unwrap());

    println!("getMe: {:?}", api.get_me());

    let users = RefCell::new(UserCollection::new(api.clone()));
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
                            let chat_id = match message.chat {
                                Chat::Private { id, .. } => id,
                                Chat::Group { id, .. } => id,
                                Chat::Channel { id, .. } => id,
                            };

                            let get_name = || {
                                (message.from.first_name.clone(), message.from.last_name.clone().unwrap_or("".to_string()))
                            };

                            let mut users_mut = users.borrow_mut();
                            let mut user = users_mut
                                .deref_mut()
                                .get_or_create_user(chat_id, &get_name);
                            user.state_machine.process(text);
                        }
                        _ => {}
                    };
                });

            Result::Ok(ListeningAction::Continue)
        })
        .unwrap();
}
