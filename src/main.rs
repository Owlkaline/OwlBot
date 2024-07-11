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
use websocket::url::form_urlencoded::Target;

use enum_all_variants::AllVariants;

use sysinfo::{Components, Disks, Networks, System};

mod modules;

use modules::consts::*;
//use modules::TwitchChat;
use obws::Client;

use twitch_eventsub::{
  error, Event, EventSubError, ResponseType, Subscription, TokenAccess, TwitchEventSubApi,
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
  luwurk,
  DotFiles,
  NeoFetch,
  Editor,
  Distro,
  Projects,
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
  Bread,
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

    if close_command == Some(ChatCommands::Lurk) {
      actual_command = Some(ChatCommands::Lurk);
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
    .add_subscriptions(vec![
      //Subscription::UserUpdate,
      Subscription::ChannelFollow,
      Subscription::ChannelRaid,
      //Subscription::ChannelUpdate,
      Subscription::ChannelNewSubscription,
      //Subscription::ChannelSubscriptionEnd,
      Subscription::ChannelGiftSubscription,
      Subscription::ChannelResubscription,
      Subscription::ChannelCheer,
      Subscription::ChannelPointsCustomRewardRedeem,
      Subscription::ChannelPointsAutoRewardRedeem,
      //Subscription::ChannelPollBegin,
      //Subscription::ChannelPollProgress,
      //Subscription::ChannelPollEnd,
      //Subscription::ChannelPredictionBegin,
      //Subscription::ChannelPredictionProgress,
      //Subscription::ChannelPredictionLock,
      //Subscription::ChannelPredictionEnd,
      //Subscription::ChannelGoalBegin,
      //Subscription::ChannelGoalProgress,
      //Subscription::ChannelGoalEnd,
      //Subscription::ChannelHypeTrainBegin,
      //Subscription::ChannelHypeTrainProgress,
      //Subscription::ChannelHypeTrainEnd,
      //Subscription::ChannelShoutoutCreate,
      //Subscription::ChannelShoutoutReceive,
      Subscription::ChatMessage,
      //Subscription::BanTimeoutUser,
      Subscription::DeleteMessage,
      Subscription::AdBreakBegin,
    ])
    //.add_subscription(Subscription::ChatMessage)
    //.add_subscription(Subscription::ChannelPointsCustomRewardRedeem)
    //.add_subscription(Subscription::BanTimeoutUser)
    //.add_subscription(Subscription::DeleteMessage)
    //.add_subscription(Subscription::AdBreakBegin)
    //.add_subscription(Subscription::ChannelRaid)
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

    let mut recent_loops: u32 = 0;
    let mut duration = 0;

    loop {
      if recent_loops > 100 {
        duration = 1;
      } else {
        recent_loops += 1;
      }

      for message in twitch.receive_messages(Duration::from_millis(duration)) {
        recent_loops = 0;
        match message {
          ResponseType::Event(event) => {
            match event {
              Event::Raid(raid_data) => {
                println!(
                  "Raid from {} with {} viewers!",
                  raid_data.from_broadcaster.name, raid_data.viewers
                );
                if raid_data.viewers >= 5 {
                  twitch.send_chat_message(format!("!so {}", raid_data.from_broadcaster.name));
                }
              }
              Event::AdBreakBegin(break_data) => {
                twitch.send_chat_message(format!(
                  "A {}min Ad has attacked! I try my best to not do anything interesting.",
                  break_data.duration_seconds / 60
                ));
              }
              Event::ChannelPointsAutoRewardRedeem(auto_redeem) => {
                println!("Auto point redeem: {:?}", auto_redeem);
                let message = auto_redeem.reward.message.text;
                match auto_redeem.reward.kind {
                  AutoRewardType::MessageEffect => {
                    for i in 0..3 {
                      println!("INSERT MESSAGE EFFECT: {}", message);
                    }
                  }
                  AutoRewardType::GigantifyAnEmote => {
                    for i in 0..3 {
                      println!("INSERT GIGANTIFY EMOTE: {}", message);
                    }
                  }
                  AutoRewardType::Celebration => {
                    println!(
                      "Thank you for the using the On-Screen Celebration {}!",
                      auto_redeem.user.name
                    );
                  }
                  _ => {}
                }
              }
              Event::PointsCustomRewardRedeem(reward) => {
                let title = reward.reward.title;
                let user = reward.user.name;
                let input = reward.user_input;

                if title.contains("water") {
                  println!("{} watered the Owl!", user);
                }
                if title.contains("EU") {
                  println!("{} has sent Owl to the EU!", user);
                }
                if title.contains("editor") {
                  println!("{} has requested Owl uses {}", user, input);
                }
                if title.contains("game") {
                  println!("{} has request Owl to live a rust free life.", user);
                }
              }
              Event::NewSubscription(subscription) => {
                if subscription.is_gift {
                  println!(
                    "{} received their first tier {} subscription!",
                    subscription.user.name, subscription.tier
                  );
                } else {
                  println!(
                    "{} subscribed for the first time with a tier {} sub!",
                    subscription.user.name, subscription.tier
                  );
                }
              }
              Event::GiftSubscription(gifty) => {
                println!(
                  "{} Generously Gifted {} tier {} subscriptions!",
                  gifty.user.name, gifty.total, gifty.tier
                );
              }
              Event::Resubscription(subscription) => {
                println!(
                  "{} has resubscribed for {} months total!",
                  subscription.user.name, subscription.cumulative_months
                );
                println!("    {}", subscription.message.text);
              }
              Event::Cheer(cheer) => {
                println!("{} cheered with {} bits!", cheer.user.name, cheer.bits);
              }
              Event::HypeTrainBegin(hype_train) => {
                println!("A hype train has begun!");
              }
              Event::HypeTrainEnd(hype_end) => {
                println!("The hype train ended at level {}!", hype_end.level);
              }
              Event::ChatMessage(message_data) => {
                let username = message_data.chatter.name;
                let user_id = message_data.chatter.id;
                let message = message_data.message.text;
                let message_id = message_data.message_id;

                match message_data.message_type {
                  MessageType::PowerUpsMessageEffect | MessageType::PowerUpsGigantifiedEmote => {
                    let _ = twitch.send_chat_message_with_reply(
                      format!("Thank you for supporting the channel {}!", username),
                      Some(message_id.to_owned()),
                    );
                  }
                  _ => {}
                }

                // First time chatter!
                let lower_message = message.to_ascii_lowercase();
                if !all_messages.contains_key(&username)
                  && username.to_lowercase() != STREAM_ACCOUNT
                {
                  let mut sus_words = [
                    "view",
                    "streamrise",
                    "onlyfans",
                    "http",
                    ".ly",
                    ".com",
                    ".to",
                    "promotion",
                  ];

                  if sus_words
                    .iter()
                    .filter(|sussy| lower_message.contains(*sussy))
                    .count()
                    > 1
                  {
                    // timeout viewier because its probably a bot
                    println!("{:?}", twitch.delete_message(message_id));
                    println!("Message delted");
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

                    let mut new_quote = format!("{}", quote); //~ {}", quote, username.clone());
                    if new_quote[..6] != "!quote".to_owned() {
                      new_quote = format!("\"{}\"", new_quote);
                    }
                    new_quote = format!("{} ~ {}", new_quote, username.to_owned());

                    let mut file = fs::File::options()
                      .append(true)
                      .create(true)
                      .open(QUOTES)
                      .unwrap();
                    file
                      .write_all(format!("{}\n", new_quote).as_bytes())
                      .unwrap();
                  }
                }

                let message = message.to_ascii_lowercase();

                if message.as_bytes()[0] as char == '!' {
                  match ChatCommands::is_command(&String::from_utf8_lossy(&message.as_bytes()[1..]))
                  {
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
                            "Join Owl's discord at: https://discord.gg/8pdfBzGbgB"
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
                        ChatCommands::Lurk | ChatCommands::Loork | ChatCommands::luwurk => {
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
                          #[cfg(target_os = "linux")]
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
                          #[cfg(target_os = "windows")]
                          twitch.send_chat_message(format!(
                            "The command you are looking for is !distro"
                          ));
                        }
                        ChatCommands::Projects => {
                          twitch.send_chat_message(format!("Owl is working on a Rust library that allows you to talk to the twitch API: https://github.com/lilith645/TwitchEventSub-rs"));
                        }
                        ChatCommands::Fimsh => {
                          twitch.send_chat_message("owlkal1Fimsh".to_owned());
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
                          twitch.send_chat_message("owlkal1OC");
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
                          twitch.send_chat_message("Would you be interested in watching my vods if I was to put them onto a Vods channel on youtube?");
                          //twitch
                          //  .send_chat_message("What is the best coop video game you have played?");
                          //twitch.send_chat_message("Who are you most excited to pull in ZZZ?");
                          //twitch.send_chat_message("What is your most fond programming moment?");
                          //twitch.send_chat_message("What was the last game you played that you were surpised that you liked?");
                          //twitch.send_chat_message("As a veiwer, do you know what you are wanting, when you click on a twitch channel?");
                          // twitch.send_chat_message("What kind of programming challange or language do you think would be fun to see a streamer try?");
                          //twitch.send_chat_message("What is the biggest hurdle in your way of doing what you want to do in life? Do you know the steps on how to overcome this hurdle?");
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
                        ChatCommands::Bread => {
                          twitch.send_chat_message("🍞 I knead your loaf.");
                        }
                      }
                    }

                    (None, Some(close), _) => {
                      twitch.send_chat_message(format!(
                        "Did you mean to type the !{:?} command",
                        close
                      ));
                    }

                    e => {
                      //println!("{:#?}", e);
                    }
                  }
                }
              }
              // rest of events
              _ => {}
            }
          }
          // MessageType::CustomRedeem((username, input, reward)) => {
          //   println!(
          //     "{} redeemed {} with {} Oxygen Atoms: {}",
          //     username, reward.title, reward.cost, input,
          //   );
          // }
          // MessageType::AdBreakNotification(duration) => {
          //   twitch.send_chat_message(format!("A {}min Ad has attacked! sorry for any inconviences. I try my best to not do anything interesting and hope you at least get hilarious ads!", duration / 60));
          // }
          ResponseType::Error(event_sub_error) => {
            println!("{:?}", event_sub_error);
            error!("{:?}", event_sub_error);
          }
          ResponseType::RawResponse(raw_data) => {
            let response = format!("RAW response: {}", raw_data);
            println!("{}", response);
          }
          _ => {}
        }
      }
    }
  }
}
