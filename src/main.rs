use std::fs::read_to_string;

use clap::Parser;
use log::warn;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{MessageEntity, MessageEntityKind, User},
};

use itertools::Itertools;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct Secret {
    token: String,
    lawn_chat_id: String,
    channel_url: String,
}

#[derive(Parser, Debug)]
struct Args {
    /// path to the secret toml file
    #[clap(short, long, default_value = "secret.toml")]
    secret: String,
}

#[derive(Default, Debug)]
struct MessageBuilder {
    text: String,
    entities: Vec<MessageEntity>,
}

#[allow(unused)]
impl MessageBuilder {
    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn push_text_link(&mut self, text: &str, url: Url) {
        let offset = self.text.chars().count();
        let length = text.chars().count();
        self.text.push_str(text);
        let entity = MessageEntity {
            kind: MessageEntityKind::TextLink { url },
            offset,
            length,
        };
        self.entities.push(entity);
    }

    fn build(self) -> Option<(String, Vec<MessageEntity>)> {
        if self.text.is_empty() {
            return None;
        }
        Some((self.text, self.entities))
    }
}

fn grow(builder: &mut MessageBuilder, from_user: &User) {
    builder.push_text(&from_user.first_name);
    if let Some(last_name) = &from_user.last_name {
        builder.push_text(" ");
        builder.push_text(last_name);
    }
    builder.push_text(" 种了一棵草。");
}

fn grow_for(builder: &mut MessageBuilder, from_user: &User, to_users: &[&User]) {
    builder.push_text(&from_user.first_name);
    if let Some(last_name) = &from_user.last_name {
        builder.push_text(" ");
        builder.push_text(last_name);
    }
    builder.push_text(" 为 ");

    let names = to_users
        .iter()
        .map(|user| {
            let mut name = user.first_name.clone();
            if let Some(last_name) = &user.last_name {
                name.push(' ');
                name.push_str(last_name)
            }
            name
        })
        .collect_vec();

    builder.push_text(&names.join("、"));

    builder.push_text(" 种了一棵草。");
}

fn farm(builder: &mut MessageBuilder, message: &Message) {
    if let Some(from_user) = message.from() {
        if let Some(reply_to_message) = message.reply_to_message() && let Some(to_user) = reply_to_message.from(){
            grow_for(builder, from_user, &[to_user]);
            return;
        }
        if let Some(entities) = message.entities() {
            let to_users = entities
                .iter()
                .filter(|entity| {
                    matches!(
                        entity.kind,
                        MessageEntityKind::Mention | MessageEntityKind::TextMention { .. }
                    )
                })
                .map(|entity| match &entity.kind {
                    MessageEntityKind::Mention => User {
                        id: UserId(0),
                        is_bot: false,
                        first_name: message.text().unwrap()
                            [entity.offset..entity.offset + entity.length]
                            .to_string(),
                        last_name: None,
                        username: None,
                        language_code: None,
                        is_premium: false,
                        added_to_attachment_menu: false,
                    },
                    MessageEntityKind::TextMention { user } => user.clone(),
                    _ => unreachable!(),
                })
                .collect_vec();
            grow_for(builder, from_user, &to_users.iter().collect_vec());
            return;
        }
        grow(builder, from_user);
        return;
    }
    warn!("Farmer strikes because there is no grass to grown!");
}

#[allow(unused)]
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("teloxide error: {0}")]
    TeloxideRequest(#[from] teloxide::RequestError),
    #[error("url parse error: {0}")]
    UrlParse(String),
}

#[allow(unused)]
impl Error {
    fn url_parse_error(e: impl std::error::Error + Send + Sync) -> Self {
        Self::UrlParse(e.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let secret = read_to_string(&args.secret)?;
    let secret: Secret = toml::from_str(&secret)?;

    pretty_env_logger::init();

    let bot = Bot::new(&secret.token).auto_send();

    teloxide::repl(bot, move |message: Message, bot: AutoSend<Bot>| {
        let secret = secret.clone();
        let re = Regex::new(r"(草|cao|焯|🌱|🌿|☘️|🍀|艹)").unwrap();
        async move {
            if let Some(text) = message.text() {
                println!("{:#?}", message);

                if re.is_match(text) {
                    let mut builder = MessageBuilder::default();

                    farm(&mut builder, &message);

                    if let Some((text, entities)) = builder.build() {
                        bot.send_message(secret.lawn_chat_id, &text)
                            .entities(entities)
                            .await?;
                    }
                }
            }
            respond(())?;
            Ok::<(), Error>(())
        }
    })
    .await;

    Ok(())
}
