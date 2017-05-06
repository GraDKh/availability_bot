use super::basic_structures::{MessageSender, MessageProcessor};
use super::user_data::ChatID;

use telegram_bot::{Api, ReplyMarkup, ReplyKeyboardMarkup, ListeningMethod, MessageType, Chat, ListeningAction
};

use std::error::Error;

pub struct TelegramMessageSender<'a> {
    bot_api: &'a Api
}

impl<'a> TelegramMessageSender<'a> {
    pub fn new(bot_api: &'a Api) -> Self {
        Self {bot_api}
    }
} 

impl<'a> MessageSender for TelegramMessageSender<'a> {
    fn send_text(&mut self, chat_id: ChatID, text: String) {
        if let Err(error) = self.bot_api.send_message(chat_id, text, None, None, None, None) {
            error!("Failed to send text message to {}: {}", chat_id, error.description());
        }
    }

    fn send_menu(&mut self,
                 chat_id: ChatID,
                 text: String,
                 menu: Vec<Vec<String>>) {
        let reply_markup = ReplyMarkup::Keyboard(ReplyKeyboardMarkup {
                                                keyboard: menu,
                                                one_time_keyboard: Some(true),
                                                selective: Some(true),
                                                ..Default::default()
                                            });
        if let Err(error) = self.bot_api
            .send_message(chat_id,
                          text,
                          None,
                          None,
                          None,
                          Some(reply_markup)) {
            error!("Failed to send menu message to {}: {}", chat_id, error.description());
        }
    }
}

pub fn process_messages(bot_api: &Api, message_processor: &mut MessageProcessor) {
    let mut listener = bot_api.listener(ListeningMethod::LongPoll(None));

    listener
        .listen(|update| {
            println!("Got message: {:?}", update);

            update
                .message
                .as_ref()
                .map(|ref message| {
                    match message.msg {
                        MessageType::Text(ref text) => {
                            if message_processor.is_new_message(update.update_id) {
                                let chat_id = match message.chat {
                                    Chat::Private { id, .. } => id,
                                    Chat::Group { id, .. } => id,
                                    Chat::Channel { id, .. } => id,
                                };

                                let last_name = message.from.last_name.as_ref().map(|string| string.as_str());
                                message_processor.process_message(chat_id, 
                                                                  message.from.first_name.as_str(),
                                                                  last_name,
                                                                  text);
                            }
                        }
                        _ => {}
                    };
                });

            Result::Ok(ListeningAction::Continue)
        })
        .expect("Result of the bot listening failed");
}