use rand::thread_rng;
use rand::Rng;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::{stdin, Read, Write};
use std::process::Command;
use std::process::Stdio;
use std::{thread, time::Duration};

use enum_all_variants::AllVariants;

use sysinfo::{Components, Disks, Networks, System};

mod modules;

use modules::consts::*;
//use modules::TwitchChat;
use obws::Client;

use twitch_eventsub::{
  error, EventSubError, MessageType, SubscriptionPermission, TokenAccess, TwitchEventSubApi,
  TwitchHttpRequest, TwitchKeys, *,
};

#[derive(AllVariants, Debug, Clone, PartialEq)]
enum ChatCommands {
  Hello,
  Meat,
  Processing,
  Discord,
  Optical,
  Throne,
  Owlyfans,
  HowToQuote,
  Quote,
  Commands,
  Ram,
  Github,
  Lurk,
  Loork,
  DotFiles,
  NeoFetch,
  Editor,
  Distro,
  EventSubLib,
  Fimsh,
  Break,
  Throbber,
  VioletCrumble,
  SO,
  ShoutOut,
  QOD,
  QuestionOfTheDay,
  Theme,
  Bones,
  Train,
}

impl ChatCommands {
  pub fn is_command(
    possible_command: &str,
  ) -> (Option<ChatCommands>, Option<ChatCommands>, Vec<String>) {
    let mut parameters = Vec::new();

    let deconstructed_command = possible_command
      .split_whitespace()
      .map(std::string::ToString::to_string)
      .collect::<Vec<_>>();

    if !possible_command.is_ascii() || deconstructed_command.len() == 0 {
      return (None, None, parameters);
    }

    let possible_command = deconstructed_command[0].clone();
    if deconstructed_command.len() > 1 {
      parameters = deconstructed_command[1..].to_vec();
    }

    let mut actual_command = None;
    let mut close_command = None;
    let mut last_distance = 3;

    for variant in ChatCommands::all_variants() {
      let command = format!("{:?}", variant).to_ascii_lowercase();
      let distance = ChatCommands::levenshtein_distance(&possible_command, &command, 0);

      if distance <= 1 {
        actual_command = Some(variant.clone());
        close_command = None;
        if distance == 0 {
          break;
        }

        if distance == 0 {}
      } else if distance < last_distance {
        close_command = Some(variant.clone());
        last_distance = distance;
      }
    }

    if close_command == Some(ChatCommands::SO) || close_command == Some(ChatCommands::ShoutOut) {
      close_command = None;
    }

    (actual_command, close_command, parameters)
  }

  pub fn levenshtein_distance(a: &str, b: &str, temp_distance: u32) -> u32 {
    if temp_distance > 10 {
      return temp_distance;
    }

    let a = a.as_bytes();
    let b = b.as_bytes();

    let a_len = a.len();
    let b_len = b.len();

    let a = String::from_utf8_lossy(&a);
    let b = String::from_utf8_lossy(&b);

    if b_len == 0 {
      // return all of a as u32
      return a.chars().count() as u32;
    } else if a_len == 0 {
      // return all of b as u32
      return b.chars().count() as u32;
    }

    let a_tail = String::from_utf8_lossy(&a.as_bytes()[1..]);
    let b_tail = String::from_utf8_lossy(&b.as_bytes()[1..]);
    let a_head = a.as_bytes()[0] as char;
    let b_head = b.as_bytes()[0] as char;

    if a_head == b_head {
      Self::levenshtein_distance(&a_tail, &b_tail, temp_distance)
    } else {
      1 + (Self::levenshtein_distance(&a_tail, &b, temp_distance + 1))
        .min(Self::levenshtein_distance(&a, &b_tail, temp_distance + 1))
        .min(Self::levenshtein_distance(
          &a_tail,
          &b_tail,
          temp_distance + 1,
        ))
    }
  }
}

fn main() {
  let keys = TwitchKeys::from_secrets_env().unwrap();
  let redirect_url = "http://localhost:3000";

  let mut twitch = TwitchEventSubApi::builder(keys.clone())
    .set_redirect_url(redirect_url)
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    .add_subscription(SubscriptionPermission::ChatMessage)
    .add_subscription(SubscriptionPermission::CustomRedeem)
    .add_subscription(SubscriptionPermission::BanTimeoutUser)
    .add_subscription(SubscriptionPermission::DeleteMessage)
    .add_subscription(SubscriptionPermission::AdBreakBegin)
    .add_subscription(SubscriptionPermission::ChannelRaid)
    // .add_subscription(SubscriptionPermission::Custom(("channel.chat.message".to_owned(), "user:read:chat+user:write:chat".to_owned(),
    //   EventSubscription {
    //     kind: "channel.chat.message".to_owned(),
    //     version: "1".to_string(),
    //     condition: Condition {
    //       broadcaster_user_id: Some(keys.broadcaster_account_id.to_owned()),
    //       moderator_user_id: None,
    //       user_id: Some(keys.broadcaster_account_id.to_owned()),
    //       reward_id: None,
    //     },
    //     transport: Transport::new(""),
    //   })
    //))
    .build()
    .unwrap();

  {
    let mut rng = thread_rng();

    let mut all_messages = HashMap::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    loop {
      for message in twitch.receive_messages() {
        match message {
          MessageType::ChatMessage(message_data) => {
            let username = message_data.username;
            let user_id = message_data.user_id;
            let message = message_data.message;
            let message_id = message_data.message_id;

            // First time chatter!
            let lower_message = message.to_ascii_lowercase();
            if !all_messages.contains_key(&username) && username.to_lowercase() != STREAM_ACCOUNT {
              if (lower_message.contains("view") || lower_message.contains("onlyfans"))
                && (lower_message.contains("http")
                  || lower_message.contains(".ly")
                  || lower_message.contains(".com")
                  || lower_message.contains(".to"))
              {
                // timeout viewier because its probably a bot
                twitch.delete_message(message_id);
                //twitch.timeout_user(user_id, 5, "You are probably a bot, get rekt.");
                continue;
              }
            }

            if username.to_lowercase() == STREAM_ACCOUNT {
              println!("{}", message);
            } else {
              println!("{}: {}", username, message);
            }
            // comment
            all_messages
              .entry(username.clone())
              .and_modify(|msg: &mut Vec<String>| msg.push(message.clone()))
              .or_insert(vec![message.clone()]);

            let possible_quote = message.to_ascii_lowercase();
            if (possible_quote.contains("don't quote")
              || possible_quote.contains("dont quote")
              || possible_quote.contains("do not quote"))
              && username.to_lowercase() != STREAM_ACCOUNT
            {
              if let Some(msgs) = all_messages.get(&username) {
                let quote = if msgs.len() > 1 {
                  msgs[msgs.len() - 2].clone()
                } else {
                  msgs[0].clone()
                };

                let new_quote = format!("{} ~ {}", quote, username.clone());

                let mut file = fs::File::options().append(true).open(QUOTES).unwrap();
                file
                  .write_all(format!("{}\n", new_quote).as_bytes())
                  .unwrap();
              }
            }

            let message = message.to_ascii_lowercase();

            if message.as_bytes()[0] as char == '!' {
              match ChatCommands::is_command(&String::from_utf8_lossy(&message.as_bytes()[1..])) {
                (Some(command), None, parameters) => {
                  match command {
                    ChatCommands::Hello => {
                      twitch.send_chat_message(format!("Welcome to the stream {}! owlkal1LHand owlkal1Leye owlkal1Yap owlkal1Reye owlkal1RHand", username));
                    }
                    ChatCommands::Meat => {
                      twitch.send_chat_message(format!(
                        "Find out what happened to your meat today! https://youtu.be/7tScAyNaRdQ"
                      ));
                    }
                    ChatCommands::Processing => {
                      twitch.send_chat_message(format!(
                          "Neat little programming program for protoyping, check it out: https://processing.org/"
                     ));
                    }
                    // Second discord that looks normal is actually some
                    // kind of special characters (Cyrillic)
                    ChatCommands::Discord => {
                      twitch.send_chat_message(format!(
                        "Join my discord at: https://discord.gg/8pdfBzGbgB"
                      ));
                    }
                    ChatCommands::Optical => {
                      twitch.send_chat_message(format!(
                      "Optical illusion here: https://media.discordapp.net/attachments/691453928709292032/1241676080226762814/opticalIllusion.png?ex=66559c76&is=66544af6&hm=7c46b66eba9defe28cd42ab7a139af97b9c9646fc7ce0634cea49641cada8262&=&format=webp&quality=lossless&width=907&height=510"
                    ));
                    }
                    ChatCommands::Throne => {
                      twitch.send_chat_message(format!(
                        "Throne wishlist: https://throne.com/owlkaline"
                      ));
                    }
                    ChatCommands::Owlyfans => {
                      twitch.send_chat_message(format!(
                        "To Support the Owl more, Support on OwlyFans: https://ko-fi.com/owlkaline"
                      ));
                    }
                    ChatCommands::HowToQuote => {
                      twitch.send_chat_message(
                        "Type \"don\'t quote\" to quote your previous message!".to_string(),
                      );
                    }
                    ChatCommands::Quote => {
                      if let Ok(quotes) = fs::read_to_string(QUOTES) {
                        let lines = quotes.lines().collect::<Vec<_>>();

                        let line_count = lines.len() as f32;
                        let rng = rng.gen::<f32>();

                        let idx = (rng * line_count).floor() as usize;

                        let quote = lines[idx];
                        twitch.send_chat_message(quote.to_string());
                      } else {
                        twitch.send_chat_message(format!("The quotes were cleared! Make your own quote by sending the quote in chat, then have your next message contain \"don\'t quote me\" to create a quote."));
                      }
                    }
                    ChatCommands::Commands => {
                      let mut all_commands = "The Following commands exist:\n".to_string();
                      for variant in ChatCommands::all_variants() {
                        all_commands = format!("{}!{:?}\n", all_commands, variant);
                      }
                      twitch.send_chat_message(all_commands);
                    }
                    ChatCommands::Ram => {
                      sys.refresh_all();

                      let ram_used = sys.used_memory() as f32 / 1000000000.0;
                      let total_ram = sys.total_memory() as f32 / 1000000000.0;

                      twitch.send_chat_message(format!(
                        "Current Ram: {:.1}/{:.1} Gb ({:.0}%)",
                        ram_used,
                        total_ram,
                        (ram_used / total_ram * 100.0).round()
                      ));
                    }
                    ChatCommands::Github => {
                      twitch.send_chat_message(format!(
                        "Owl's github can be found at: https://github.com/lilith645",
                      ));
                    }
                    ChatCommands::Lurk | ChatCommands::Loork => {
                      twitch.send_chat_message(format!(
                        "Thanks for coming by, appreciate the lurk {}!",
                        username
                      ));
                    }
                    ChatCommands::DotFiles => {
                      twitch.send_chat_message(format!(
                        "You can Owl's linux dot files here: https://github.com/lilith645/dotfiles"
                      ));
                    }
                    ChatCommands::Editor => {
                      twitch.send_chat_message(format!(
                      "I switch between Helix , Neovim and Zed currently, there is a redeem to make Owl use a new editor!"
                    ));
                    }
                    ChatCommands::Distro => {
                      twitch.send_chat_message(format!(
                        "The distro Owl uses is {} on kernel {}",
                        System::long_os_version().unwrap_or("".to_string()),
                        System::kernel_version().unwrap_or("".to_string())
                      ));
                    }
                    ChatCommands::NeoFetch => {
                      Command::new("neofetch")
                        .arg("--disable")
                        .args(["memory", "Theme", "icons", "WM", "Terminal", "shell"])
                        .arg("--color_blocks")
                        .arg("off")
                        .arg("--ascii_distro")
                        .arg(" Manjaro_small")
                        .arg("--gap")
                        .arg("0")
                        .status()
                        .unwrap();
                      //neofetch --disable memory Theme icons WM Terminal shell
                    }
                    ChatCommands::EventSubLib => {
                      twitch.send_chat_message(format!("Owl is working on a Rust library that allows you to talk to the twitch API: https://github.com/lilith645/TwitchEventSub-rs"));
                    }
                    ChatCommands::Fimsh => {
                      twitch.send_chat_message("🐠".to_owned());
                    }

                    ChatCommands::Break => {
                      twitch.send_chat_message(
                        "Please break my chat bot, I'll thank you for it!".to_owned(),
                      );
                    }
                    ChatCommands::Throbber => {
                      twitch.send_chat_message(
                        "Time for them blue pills owlkal1LHand owlkal1RHand".to_owned(),
                      );
                    }
                    ChatCommands::VioletCrumble => {
                      twitch.send_chat_message("owlkal1Vc");
                    }
                    ChatCommands::SO | ChatCommands::ShoutOut => {
                      if parameters.len() > 0 {
                        twitch.send_chat_message(format!(
                          "{} is an awesome streamer, follow them at http://twitch.tv/{}",
                          parameters[0], parameters[0],
                        ));
                      }
                    }
                    ChatCommands::QOD | ChatCommands::QuestionOfTheDay => {
                      //twitch.send_chat_message("As a veiwer, do you know what you are wanting, when you click on a twitch channel?");
                      twitch.send_chat_message("What is the biggest hurdle in your way of doing what you want to do in life? Do you know the steps on how to overcome this hurdle?");
                    }
                    ChatCommands::Theme => {
                      twitch.send_chat_message(
                        "Owl uses the Dracula theme! (https://draculatheme.com/)",
                      );
                    }
                    ChatCommands::Bones => {
                      twitch.send_chat_message("IF YOURE NOT HAVING A GOOD TIME CRACK YOUR BONES ITS GOOD FOR YOU AND BONES ARE NOT REAL ANYWAY");
                    }
                    ChatCommands::Train => {
                      twitch.send_chat_message("choo chooooo");
                    }
                  }
                }

                (None, Some(close), _) => {
                  twitch
                    .send_chat_message(format!("Did you mean to type the !{:?} command", close));
                }
                _ => {}
              }
            }
          }
          MessageType::CustomRedeem((username, input, reward)) => {
            println!(
              "{} redeemed {} with {} Oxygen Atoms: {}",
              username, reward.title, reward.cost, input,
            );
          }
          MessageType::AdBreakNotification(duration) => {
            twitch.send_chat_message(format!("A {}min Ad has attacked! sorry for any inconviences. I try my best to not do anything interesting and hope you at least get hilarious ads!", duration / 60));
          }
          MessageType::SubscribeError(failed_to_subscribe) => {
            println!("Failed to subscribe to {:?}", failed_to_subscribe);
          }
          MessageType::Error(event_sub_error) => {
            println!("{:?}", event_sub_error);
            error!("{:?}", event_sub_error);
          }
          MessageType::RawResponse(raw_data) => {
            let response = format!("RAW response: {}", raw_data);
            println!("{}", response);
          }
          _ => {}
        }
      }
    }
  }
}
