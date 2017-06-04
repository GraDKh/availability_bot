use ::message_processor::dialog_processing::{YES_NO_MENU, DialogAction, ReplyMessage,
    DialogInitializationResult, DynamicSerializable, StaticNameGetter, Dialog, Event};
use ::basic_structures::{WfhSingleDay};
use ::user_data::{UserInfo};

use chrono;

use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InitialState {
}

impl InitialState {
    fn new() -> Self {
        Self {}
    }

    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> (WfhState, DialogAction) {
        if text.starts_with("/wfh") {
            match user_info.get_calendar_name() {
                Some(_) => (WfhState::SetDate(SetDateState::new()), 
                            DialogAction::ProcessAndContinue(Some(ReplyMessage::new("Today?", Some(YES_NO_MENU.clone()))), None)),
                None => (WfhState::Initial(InitialState::new()),
                         DialogAction::ProcessAndStop(Some(ReplyMessage::new("Please specify your calendar name using /setmyname",
                            None)), None))
            }
        }
        else {
            (WfhState::Initial(InitialState::new()), DialogAction::Stop)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SetDateState {
}

impl SetDateState {
    fn new() -> Self {
        Self {}
    }

    fn try_process(&mut self, text: &str, _: &mut UserInfo) -> (WfhState, DialogAction) {
        if text == "yes" {
            (WfhState::Confirmation(ConfirmationState::new()), 
            DialogAction::ProcessAndContinue(Some(ReplyMessage::new("Confirm event wfh for today?", Some(YES_NO_MENU.clone()))), None))
        }
        else {
            (WfhState::Initial(InitialState::new()), DialogAction::Stop)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfirmationState {
}

impl ConfirmationState {
    fn new() -> Self {
        Self {}
    }

    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> (WfhState, DialogAction) {
        if text == "yes" {
            (WfhState::Initial(InitialState::new()), 
            DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", Some(YES_NO_MENU.clone()))), 
                Some(Event::WfhSingleDay(WfhSingleDay::new(user_info.get_calendar_name().unwrap(),
                &chrono::Local::today())))))
        }
        else {
            (WfhState::Initial(InitialState::new()), DialogAction::ProcessAndStop(Some(ReplyMessage::new("Canceled!", None)), None))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum WfhState {
    Initial(InitialState),
    SetDate(SetDateState),
    Confirmation(ConfirmationState)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WfhDialog {
    state: WfhState
}

impl Dialog for WfhDialog {
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> DialogAction {
        let (state, result) = match self.state {
            WfhState::Initial(ref mut state) => state.try_process(text, user_info),
            WfhState::SetDate(ref mut state) => state.try_process(text, user_info),
            WfhState::Confirmation(ref mut state) => state.try_process(text, user_info)
        };

        self.state = state;

        result
    }

    fn make(initial_message:&str, user_info:&mut UserInfo) -> DialogInitializationResult {
        let mut dialog = Self {state: WfhState::Initial(InitialState::new())};
        let result = dialog.try_process(initial_message, user_info);
        match result {
            DialogAction::ProcessAndContinue(reply, event) => DialogInitializationResult::StartedProcessing(reply, event, Box::new(dialog)),
            DialogAction::ProcessAndStop(reply, event) => DialogInitializationResult::Finished(reply, event),
            DialogAction::Stop => DialogInitializationResult::NotProcessed,
        }
    }
}

impl DynamicSerializable for WfhDialog {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()// FIXME
    }

    fn from_string(string: &str) -> Self {
        serde_json::from_str::<Self>(string).unwrap() // FIXME
    }
}

impl StaticNameGetter for WfhDialog {
    fn get_name() -> &'static str{
        return "wfh-dialog";
    }
}