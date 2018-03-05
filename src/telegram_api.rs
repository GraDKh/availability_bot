use super::basic_structures::{MessageSender, MessageProcessor};
use super::user_data::ChatID;

use telegram_bot::{Api, ReplyMarkup, InlineKeyboardMarkup, UpdateKind, MessageKind, ChatRef, SendMessage,
InlineKeyboardButton, User, CallbackQuery, CanAnswerCallbackQuery, CanEditMessageReplyMarkup};

use tokio_core::reactor::Core;

use futures::Stream;

pub struct TelegramApi {
    core: Core,
    bot_api: Api,
}

impl TelegramApi {
    pub fn new(bot_token: &str) -> Self {
        let core = Core::new().unwrap();
        let bot_api = Api::configure(bot_token).build(core.handle()).unwrap();

        return Self {core, bot_api};
    }

    pub fn process_messages(self: &mut Self, message_processor: &mut MessageProcessor) {
        let mut message_sender = TelegramMessageSender::new(&self.bot_api);

        let future = self.bot_api.stream().for_each(|update| {
            fn get_last_name(user: &User) -> Option<&str> {
                user.last_name.as_ref().map(|string| string.as_str())
            }

            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text {ref data, ..} = message.kind {
                    if message_processor.is_new_message(update.id) {
                        message_processor.process_message(& mut message_sender, 
                                                          message.chat.id(),
                                                          message.from.first_name.as_str(),
                                                          get_last_name(&message.from),
                                                          data);
                    }
                        
                    
                    // Print received text message to stdout.
                    println!("<{}>: {}", &message.from.first_name, data);
                }
            }
            else if let UpdateKind::CallbackQuery(ref callback_query) = update.kind {
                message_processor.process_message(& mut message_sender, 
                                                  callback_query.message.chat.id(),
                                                  callback_query.from.first_name.as_str(),
                                                  get_last_name(&callback_query.from),
                                                  &callback_query.data);

                message_sender.send_query_reply(&callback_query);
                println!("<{}>: {}", callback_query.from.first_name, callback_query.data);
            }

            Ok(())
        });

        self.core.run(future).unwrap();
    }
}

struct TelegramMessageSender<'a> {
    bot_api : &'a Api
}

impl<'a> TelegramMessageSender<'a> {
    fn new(bot_api : &'a Api) -> Self {
        Self { bot_api}
    }

    fn send_query_reply(self: &mut Self, query: &CallbackQuery) {
        self.bot_api.spawn(query.answer(""));
        self.bot_api.spawn(query.message.edit_reply_markup::<ReplyMarkup>(Option::None));
    }
}

impl<'a> MessageSender for TelegramMessageSender<'a> {
    fn send_text(&mut self, chat_id: ChatID, text: String) {
        let chat = ChatRef::from_chat_id(chat_id);
        let message_req = SendMessage::new(chat, text);
        self.bot_api.spawn(message_req);
    }

    fn send_menu(&mut self, chat_id: ChatID, text: String, menu: Vec<Vec<String>>) {
        let reply_markup = (|| -> InlineKeyboardMarkup {
            let mut result = InlineKeyboardMarkup::new();
            for row in menu.iter() {
                let keys_row = row.iter().map(|ref key_name| InlineKeyboardButton::callback(key_name, key_name)).collect();
                result.add_row(keys_row);
            }
            result
        })();
        let chat = ChatRef::from_chat_id(chat_id);
        let mut message_req = SendMessage::new(chat, text);
        message_req.reply_markup(ReplyMarkup::InlineKeyboardMarkup(reply_markup));
        self.bot_api.spawn(message_req);
    }
}
