use std::env;

use dotenv::dotenv;
use futures::StreamExt;
use regex::Regex;
use telegram_bot::*;

enum ReplaceOption {
    Suffixed { prefix: &'static str },
    Plain { replacement: &'static str },
}

struct MatchPattern {
    cut: Regex,
    word: Regex,
    replacement: ReplaceOption,
}

impl MatchPattern {
    pub fn replace(&self, ending: &str) -> String {
        match self.replacement {
            ReplaceOption::Suffixed { prefix } => {
                prefix.to_owned() + ending.to_lowercase().as_str()
            }
            ReplaceOption::Plain { replacement } => replacement.to_owned(),
        }
    }
}

fn fold_join(acc: String, v: String) -> String {
    let formatted = format!(", {}", v);
    let owned_v = v.to_owned();
    let next_v = if acc == String::from("") || owned_v == String::from("") {
        v.to_owned()
    } else {
        formatted
    };

    acc + next_v.as_str()
}

type MatchPatternArray = [MatchPattern; 3];

fn get_praise_pattern() -> Regex {
    return Regex::new(r"^(молодец|спасибо|хороший\sбот|thanks|good\sbot)").unwrap();
}

fn test_for_praise(text: &str, praise_pattern: Regex) -> bool {
    let match_text = praise_pattern.find(text);
    match_text.is_some()
}

fn handle_text(text: &str, patterns: &MatchPatternArray) -> String {
    patterns
        .iter()
        .map(|pattern| {
            pattern
                .word
                .find_iter(text)
                .map(|_match| {
                    // safe unwrap, because match is checked a line above
                    let match_text = text.get(_match.start().._match.end()).unwrap();
                    let match_end = pattern.cut.replace(match_text, "").into_owned();
                    pattern.replace(&match_end) + "*"
                })
                .fold(String::new(), fold_join)
        })
        .fold(String::new(), |s, x| fold_join(s, x.to_owned()))
}

fn get_patterns() -> MatchPatternArray {
    [
        {
            MatchPattern {
                cut: Regex::new(r"(?i)питерск").unwrap(),
                word: Regex::new(r"(?i)питерск[\wа-я]*").unwrap(),
                replacement: ReplaceOption::Suffixed {
                    prefix: "Пидорск"
                },
            }
        },
        {
            MatchPattern {
                cut: Regex::new(r"(?i)питерец").unwrap(),
                word: Regex::new(r"(?i)питерец").unwrap(),
                replacement: ReplaceOption::Plain {
                    replacement: "Пидор",
                },
            }
        },
        {
            MatchPattern {
                cut: Regex::new(r"(?i)питерц").unwrap(),
                word: Regex::new(r"(?i)питерц[\wа-я]*").unwrap(),
                replacement: ReplaceOption::Suffixed {
                    prefix: "Пидор"
                },
            }
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN is set");
    let api = Api::new(token);
    let patterns = get_patterns();

    let me = api.send(GetMe).await?;

    let mut stream = api.stream();
    while let Some(update_unwrapped) = stream.next().await {
        let Ok(update) = update_unwrapped else {
            println!("Caught an error in update: {:?}", update_unwrapped.err());
            continue;
        };

        if let UpdateKind::Message(ref message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                match message.reply_to_message.clone() {
                    Some(reply) => match *reply {
                        MessageOrChannelPost::Message(replied_to) => {
                            if replied_to.from == me && test_for_praise(data, get_praise_pattern())
                            {
                                api.send(message.text_reply("🥰")).await?;
                            }
                        }
                        _ => (),
                    },
                    None => (),
                };

                let answer = handle_text(data, &patterns);
                if answer != String::from("") {
                    api.send(message.text_reply(handle_text(data, &patterns)))
                        .await?;
                } else {
                    ()
                };
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test;
