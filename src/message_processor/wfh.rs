use message_processor::dialog_processing::{YES_NO_MENU, DialogAction, ReplyMessage,
                                           DialogInitializationResult, DynamicSerializable,
                                           StaticNameGetter, Dialog, Event, ChannelMessage};
use basic_structures::{WfhSingleDay, Menu};
use user_data::UserInfo;

use chrono;
use time;

use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InitialState {}

const TODAY : &str = "today";
const TOMORROW : &str = "tomorrow";
const UNTILL_NOW : &str = "untill now";
const FROM_NOW : &str = "from now";
const OTHER : &str = "other";

lazy_static! {  
    pub static ref WHEN_MENU : Menu = vec!(vec!(TODAY.into(), TOMORROW.into()),
                                           vec!(UNTILL_NOW.into(),(FROM_NOW.into())),
                                           vec!(OTHER.into()));
}

impl InitialState {
    fn new() -> Self {
        Self {}
    }

    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> (WfhState, DialogAction) {
        if text.starts_with("/wfh") {
            match user_info.get_calendar_name() {
                Some(_) => (WfhState::ChooseMode(ChooseModeStateState::new()), 
                            DialogAction::ProcessAndContinue(Some(ReplyMessage::new("When?", Some(WHEN_MENU.clone()))), None)),
                None => {
                    (WfhState::Initial(InitialState::new()),
                     DialogAction::ProcessAndStop(Some(ReplyMessage::new("Please specify your calendar name using /setmyname",
                                                                         None)),
                                                  None,
                                                  None))
                }
            }
        } else {
            (WfhState::Initial(InitialState::new()), DialogAction::Stop)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChooseModeStateState {}

impl ChooseModeStateState {
    fn new() -> Self {
        Self {}
    }

    fn try_process(&mut self, text: &str, _: &mut UserInfo) -> (WfhState, DialogAction) {
        match text {
            TODAY => (WfhState::Confirmation(ConfirmationState::Today),
             DialogAction::ProcessAndContinue(Some(ReplyMessage::new("Confirm event wfh for today?",
                                                                     Some(YES_NO_MENU.clone()))),
                                              None)),
            TOMORROW => (WfhState::Confirmation(ConfirmationState::Tomorrow),
             DialogAction::ProcessAndContinue(Some(ReplyMessage::new("Confirm event wfh for tomorrow?",
                                                                     Some(YES_NO_MENU.clone()))),
                                              None)),

            _ => (WfhState::Initial(InitialState::new()), DialogAction::Stop)                                
        }
        
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ConfirmationState {
    Today,
    Tomorrow,
    // TodayBeforeNow,
    // TodayFromNow
}

impl ConfirmationState {
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> (WfhState, DialogAction) {
        if text == "yes" {
            match self {
                &mut ConfirmationState::Today =>  (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::WfhSingleDay(WfhSingleDay::new(user_info.get_calendar_name().unwrap(),
                           &chrono::Local::today()))),
                           Some(ChannelMessage::new("wfh today")))),
                &mut ConfirmationState::Tomorrow => (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::WfhSingleDay(WfhSingleDay::new(user_info.get_calendar_name().unwrap(),
                           &(chrono::Local::today() + time::Duration::days(1))))),
                           Some(ChannelMessage::new("wfh tommorow"))))
            }
        } else {
            (WfhState::Initial(InitialState::new()),
             DialogAction::ProcessAndStop(Some(ReplyMessage::new("Canceled!", None)), None, None))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum WfhState {
    Initial(InitialState),
    ChooseMode(ChooseModeStateState),
    Confirmation(ConfirmationState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WfhDialog {
    state: WfhState,
}

impl Dialog for WfhDialog {
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> DialogAction {
        let (state, result) = match self.state {
            WfhState::Initial(ref mut state) => state.try_process(text, user_info),
            WfhState::ChooseMode(ref mut state) => state.try_process(text, user_info),
            WfhState::Confirmation(ref mut state) => state.try_process(text, user_info),
        };

        self.state = state;

        result
    }

    fn make(initial_message: &str, user_info: &mut UserInfo) -> DialogInitializationResult {
        let mut dialog = Self { state: WfhState::Initial(InitialState::new()) };
        let result = dialog.try_process(initial_message, user_info);
        match result {
            DialogAction::ProcessAndContinue(reply, event) => {
                DialogInitializationResult::StartedProcessing(reply, event, Box::new(dialog))
            }
            DialogAction::ProcessAndStop(reply, event, _) => {
                DialogInitializationResult::Finished(reply, event)
            }
            DialogAction::Stop => DialogInitializationResult::NotProcessed,
        }
    }
}

impl DynamicSerializable for WfhDialog {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap() // FIXME
    }

    fn from_string(string: &str) -> Self {
        serde_json::from_str::<Self>(string).unwrap() // FIXME
    }
}

impl StaticNameGetter for WfhDialog {
    fn get_name() -> &'static str {
        return "wfh-dialog";
    }
}
