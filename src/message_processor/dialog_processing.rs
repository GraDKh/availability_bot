use basic_structures::WfhSingleDay;
use user_data::UserInfo;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use serde::de::{Deserialize, Deserializer, Visitor, Error, SeqAccess};

use std::fmt::{Debug, Formatter};
use std::fmt;
use std::sync::Mutex;
use std::collections::hash_map::HashMap;
use std::ops::Deref;

pub type Menu = Vec<Vec<String>>;

pub enum Event {
    WfhSingleDay(WfhSingleDay),
}

pub struct ReplyMessage {
    pub text: String,
    pub menu: Option<Menu>,
}

impl ReplyMessage {
    pub fn new<S>(text: S, menu: Option<Menu>) -> Self
        where S: Into<String>
    {
        Self {
            text: text.into(),
            menu,
        }
    }
}

pub enum DialogAction {
    ProcessAndContinue(Option<ReplyMessage>, Option<Event>),
    ProcessAndStop(Option<ReplyMessage>, Option<Event>),
    Stop,
}

pub enum DialogInitializationResult {
    NotProcessed,
    Finished(Option<ReplyMessage>, Option<Event>),
    StartedProcessing(Option<ReplyMessage>, Option<Event>, Box<Dialog>),
}

pub trait DynamicNameGetter {
    fn get_name_inst(&self) -> &'static str;
}

pub trait StaticNameGetter {
    fn get_name() -> &'static str where Self: Sized;
}

impl<T> DynamicNameGetter for T
    where T: StaticNameGetter + Sized
{
    fn get_name_inst(&self) -> &'static str {
        Self::get_name()
    }
}

pub trait DynamicSerializable: DynamicNameGetter + StaticNameGetter {
    fn to_string(&self) -> String;
    fn from_string(string: &str) -> Self where Self: Sized;
}

pub trait ClonableToDialog {
    fn clone_to_dialog(&self) -> Box<Dialog>;
}

pub trait Dialog: DynamicSerializable + ClonableToDialog {
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> DialogAction;
    fn make(initial_message: &str, user_info: &mut UserInfo) -> DialogInitializationResult
        where Self: Sized;
}

impl<T> ClonableToDialog for T where T: 'static + Dialog + Clone {
    fn clone_to_dialog(&self) -> Box<Dialog> {
        Box::new(self.clone())
    }
}

lazy_static! {  
    pub static ref YES_NO_MENU : Menu = vec!(vec!("yes".into(), "no".into()));
}

type DeserializersMap = HashMap<&'static str, Box<Fn(&str) -> Box<Dialog> + Send>>;
type DeserializersMapProtected = Mutex<DeserializersMap>;

lazy_static! {
    static ref DESERIALIZERS: DeserializersMapProtected = DeserializersMapProtected::new(DeserializersMap::new());
}

pub fn register_dialog<T>()
    where T: 'static + Dialog
{
    let mut deserializers = DESERIALIZERS.lock().unwrap();
    deserializers.insert(T::get_name(),
                         Box::new(|string: &str| Box::new(T::from_string(string))));
}

impl Serialize for Box<Dialog> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq_serializer = serializer.serialize_seq(Some(2))?;
        seq_serializer
            .serialize_element(self.deref().get_name_inst())?;
        seq_serializer
            .serialize_element(self.deref().to_string().as_str())?;
        seq_serializer.end()
    }
}

impl<'de> Deserialize<'de> for Box<Dialog> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct SequenceVisitor {}

        impl<'de> Visitor<'de> for SequenceVisitor {
            type Value = (String, String);

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("sequence of two strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>
            {
                //FIXME
                let name = seq.next_element::<String>()?.unwrap();
                let data = seq.next_element::<String>()?.unwrap();
                Ok((name, data))
            }
        }

        let (type_name, data) = deserializer.deserialize_seq(SequenceVisitor {})?;
        let deserializers = DESERIALIZERS.lock().unwrap();
        let dialog_deserializer = deserializers.get(type_name.as_str()).unwrap(); //FIXME
        Ok(dialog_deserializer(data.as_str()))
    }
}

impl Clone for Box<Dialog> {
    fn clone(&self) -> Box<Dialog> {
        self.deref().clone_to_dialog()
    }
}