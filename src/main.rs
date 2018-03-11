#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate telegram_bot;
extern crate tokio_core;
extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate time;
extern crate yup_oauth2;
#[macro_use]
extern crate lazy_static;

mod basic_structures;
mod message_processor;
mod configuration;
mod user_data;
mod save_load_state;
mod events_sender;
mod telegram_api;

fn main() {
    env_logger::try_init().unwrap_or_else(|error| {
                                          let message = format!("Couldn't initialize logging {:?}",
                                                                error);
                                          println!("{}", message);
                                          panic!("{}", message);
                                      });
    message_processor::init_dialog_types();

    let configuration = configuration::Configuration::load();
    
    let mut message_sender = telegram_api::TelegramApi::new(&configuration.bot_token);
    let mut events_sender = events_sender::CalendarEventsSender::new();
    let mut data_saver = save_load_state::FileDataSaver::new(&configuration.data_file);
    let mut message_processor = message_processor::UserCollection::new(&mut events_sender,
                                                                       &mut data_saver);
    message_sender.process_messages(&mut message_processor);
}
