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
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate yup_oauth2;

mod basic_structures;
mod message_processor;
mod configuration;
mod user_data;
mod save_load_state;
mod events_sender;
mod telegram_api;

use telegram_bot::Api;

fn main() {
    env_logger::init().unwrap_or_else(|error| {
                                          let message = format!("Couldn't initialize logging {:?}",
                                                                error);
                                          println!("{}", message);
                                          panic!("{}", message);
                                      });

    let configuration = configuration::Configuration::load();

    let bot_api =
        Api::from_token(&configuration.bot_token).expect("Couldn't create API object for bot");
    info!("getMe: {:?}", bot_api.get_me());

    let mut message_sender = telegram_api::TelegramMessageSender::new(&bot_api);
    let mut events_sender = events_sender::CalendarEventsSender::new();
    let mut data_saver = save_load_state::FileDataSaver::new(&configuration.data_file);
    let mut message_processor = message_processor::UserCollection::new(&mut message_sender,
                                                                       &mut events_sender,
                                                                       &mut data_saver);
    telegram_api::process_messages(&bot_api, &mut message_processor);
}
