use message_processor::dialog_processing::{YES_NO_MENU, DialogAction, ReplyMessage,
                                           DialogInitializationResult, DynamicSerializable,
                                           StaticNameGetter, Dialog, Event, ChannelMessage};
use basic_structures::{LocalDate, LocalDateTime, WholeDayEvent, PartialDayEvent, Menu};
use user_data::UserInfo;

use chrono;
use chrono::{Timelike};
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
            UNTILL_NOW => (WfhState::Confirmation(ConfirmationState::TodayBeforeNow),
             DialogAction::ProcessAndContinue(Some(ReplyMessage::new("Confirm event wfh for today before now?",
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
    TodayBeforeNow,
    TodayFromNow
}

impl ConfirmationState {
    fn try_process(&mut self, text: &str, user_info: &mut UserInfo) -> (WfhState, DialogAction) {
        if text == "yes" {
            match self {
                &mut ConfirmationState::Today =>  (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::WholeDay(make_wfh_for_today(user_info.get_calendar_name().unwrap()))),
                           Some(ChannelMessage::new("wfh today")))),
                &mut ConfirmationState::Tomorrow => (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::WholeDay(make_wfh_for_tomorrow(user_info.get_calendar_name().unwrap()))),
                           Some(ChannelMessage::new("wfh tommorow")))),
                &mut ConfirmationState::TodayBeforeNow => (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::PartialDay(make_wfh_before_now(user_info.get_calendar_name().unwrap()))),
                           Some(ChannelMessage::new("wfh today untill now")))),
                &mut ConfirmationState::TodayFromNow => (WfhState::Initial(InitialState::new()), 
                           DialogAction::ProcessAndStop(Some(ReplyMessage::new("Applied!", None)), 
                           Some(Event::PartialDay(make_wfh_from_now(user_info.get_calendar_name().unwrap()))),
                           Some(ChannelMessage::new("wfh today from now"))))
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

fn make_wfh_whole_day_event(name: &str,  start_date: &LocalDate, end_date: &LocalDate) -> WholeDayEvent {
    WholeDayEvent::new(format!("WFH: {}", name), start_date, end_date)
}

fn make_whf_single_day(name: &str,  date: &LocalDate) -> WholeDayEvent {
    make_wfh_whole_day_event(name, date, date)
}

fn make_wfh_for_today(name: &str) -> WholeDayEvent {
    make_whf_single_day(name, &chrono::Local::today())
}

fn make_wfh_for_tomorrow(name: &str) -> WholeDayEvent {
    make_whf_single_day(name, &(chrono::Local::today() + time::Duration::days(1)))
}

fn make_wfh_partial_day_event(name: &str, start_time: &LocalDateTime, end_time: &LocalDateTime) -> PartialDayEvent {
    PartialDayEvent::new(format!("WFH: {}", name), start_time, end_time)
}

fn make_same_with_hours(time: &LocalDateTime, hours: u32) -> LocalDateTime {
    time.with_hour(hours).unwrap().with_minute(0).
        unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
}

fn make_wfh_before_now(name: &str) -> PartialDayEvent {
    let now = chrono::Local::now();
    let day_start = make_same_with_hours(&now, 9);

    make_wfh_partial_day_event(name, &day_start, &now)
}

fn make_wfh_from_now(name: &str) -> PartialDayEvent {
    let now = chrono::Local::now();
    let day_end = make_same_with_hours(&now, 20);

    make_wfh_partial_day_event(name, &now, &day_end)
}