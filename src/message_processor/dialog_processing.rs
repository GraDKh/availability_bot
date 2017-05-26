use ::basic_structures::{WfhSingleDay};
use ::user_data::UserInfo;
use serde::{Serialize};
use serde::de::{DeserializeOwned};

use std::fmt::{Debug};

pub type Menu = Vec<Vec<String>>;

pub enum Event {
    WfhSingleDay(WfhSingleDay),
}

pub struct ReplyMessage {
    pub text: String,
    pub menu: Option<Menu>
}

impl ReplyMessage {
    pub fn new<S>(text: S, menu: Option<Menu>) -> Self 
        where S : Into<String> {
        Self {text: text.into(), menu}
    }
}

pub enum DialogAction {
    ProcessAndContinue(Option<ReplyMessage>, Option<Event>),
    ProcessAndStop(Option<ReplyMessage>, Option<Event>),
    Stop
}

pub enum DialogInitializationResult {
    NotProcessed,
    Finished(Option<ReplyMessage>, Option<Event>),
    StartedProcessing(Option<ReplyMessage>, Option<Event>, Box<Dialog>)
}

pub trait Dialog : DeserializeOwned + Serialize + Clone + Debug{
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> DialogAction;
    fn make(initial_message: &str, user_info: &mut UserInfo) -> DialogInitializationResult where Self : Sized;
}

lazy_static! {  
    pub static ref YES_NO_MENU : Menu = vec!(vec!("yes".into(), "no".into()));
}