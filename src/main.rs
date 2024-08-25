use rand::thread_rng;
use rand::{distributions::Uniform, Rng};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::fs;
use std::io::{stdin, Read, Write};
use std::process::Stdio;
use std::process::{exit, Command};
use std::str::SplitWhitespace;
use std::time::Instant;
use std::{thread, time::Duration};
use websocket::native_tls::MidHandshakeTlsStream;
use websocket::url::form_urlencoded::Target;

use base64::prelude::*;
use crossterm::csi;

use enum_all_variants::AllVariants;

use sysinfo::{Components, Disks, Networks, System};

use colored::*;
use colors_transform::{Color, Rgb};

mod modules;

use modules::{consts::*, emotes};
//use modules::TwitchChat;

use twitch_eventsub::{
  error, Event, EventSubError, ResponseType, Subscription, TokenAccess, TwitchEventSubApi,
  TwitchHttpRequest, TwitchKeys, *,
};

#[derive(PartialEq, AllVariants)]
enum RankVariety {
  Common,
  Uncommon,
  CarnivorousGarden,
  VeganGarden,
  SmoothedMeat,
  SmoothedVeganMeat,
  BinChicken,
  DirtyBinChicken,
  Edged,
  Sour,
  Creamed,
  Explosive,
  Trackmaniac,
  Fimsh,
  LongFimsh,
  Nean,
  Holee,
}

impl Display for RankVariety {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match *self {
        RankVariety::Common => "common",
        RankVariety::Uncommon => "uncommon",
        RankVariety::CarnivorousGarden => "carnivorous garden",
        RankVariety::VeganGarden => "vegan garden",
        RankVariety::SmoothedMeat => "smoothed meat",
        RankVariety::SmoothedVeganMeat => "smoothed vegan meat",
        RankVariety::Fimsh => "fimsh",
        RankVariety::Holee => "HOLEE",
        RankVariety::BinChicken => "Bin Chicken",
        RankVariety::DirtyBinChicken => "Dirty Bin Chicken",
        RankVariety::Edged => "Edged",
        RankVariety::Sour => "Sour",
        RankVariety::Creamed => "Creamed",
        RankVariety::Explosive => "Explosive",
        RankVariety::Trackmaniac => "Trackmaniac",
        RankVariety::LongFimsh => "Long fimsh",
        RankVariety::Nean => "Nean",
      }
    )
  }
}

#[derive(AllVariants, Debug, Clone, PartialEq)]
enum ChatCommands {
  Hello,
  Meat,
  Processing,
  Discord,
  //  Optical,
  Throne,
  Owlyfans,
  HowToQuote,
  Quote,
  Commands,
  Ram,
  Github,
  Lurk,
  Lurking,
  Loork,
  Luwurk,
  DotFiles,
  NeoFetch,
  Editor,
  Distro,
  Projects,
  Pronouns,
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
  Rank,
  Ranks,
  OwlBeCringe,
  Holee,
  Heckies,
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

macro_rules! gp {
  ($c:expr) => {
    concat!("\x1B_G", $c, "\x1b\\")
  };
}

fn run_tts(text: String) {
  let mut file = fs::File::create(SPEECH_FILE).unwrap();
  file.write_all(format!("{}\n", text).as_bytes()).unwrap();
  file.flush().unwrap();

  thread::spawn(|| {
    if let Ok(_) = Command::new("dsnote").arg("./speech").output() {
      thread::sleep(Duration::from_millis(1000));
      if let Err(e) = Command::new("dsnote")
        .arg("--action")
        .arg("start-reading")
        .output()
      {
        error!("TTS failed to read: {}", e);
      }
    }
  });
}

pub fn print_fragments(
  twitch: &mut TwitchEventSubApi,
  emote_buffer: &mut HashMap<String, u32>,
  fragments: &Vec<Fragments>,
  colour: Rgb,
) {
  for fragment in fragments {
    match fragment.kind {
      FragmentType::Emote => {
        if let Some(emote) = &fragment.emote {
          emotes::print_emote(twitch, emote.clone(), emote_buffer);
        }
      }
      _ => {
        print!(
          "{}",
          String::from(format!("{}", fragment.text)).custom_color(CustomColor::new(
            colour.get_red() as u8,
            colour.get_green() as u8,
            colour.get_blue() as u8,
          ),)
        );
      }
    }
  }
  println!("")
}

pub struct ChatMessage {
  id: String,
  username: String,
  message: Vec<Fragments>,
  username_colour: Rgb,
  message_colour: Rgb,
}

impl ChatMessage {
  pub fn print(&self, twitch: &mut TwitchEventSubApi, emote_buffer: &mut HashMap<String, u32>) {
    if self.username.to_lowercase() != STREAM_ACCOUNT {
      print!(
        "{}:",
        String::from(format!("{}", self.username)).custom_color(CustomColor::new(
          self.username_colour.get_red() as u8,
          self.username_colour.get_green() as u8,
          self.username_colour.get_blue() as u8
        ))
      );
    }
    print_fragments(twitch, emote_buffer, &self.message, self.message_colour);
  }
}

impl From<&MessageData> for ChatMessage {
  fn from(value: &MessageData) -> Self {
    let username_colour = if value.colour.is_empty() {
      Rgb::from_hex_str("#2979ff").unwrap()
    } else {
      Rgb::from_hex_str(&value.colour).unwrap()
    };
    let message_colour = username_colour.adjust_hue(90.0).set_lightness(80.0);
    ChatMessage {
      id: value.message_id.to_owned(),
      username: value.chatter.name.to_owned(),
      message: value.message.fragments.to_owned(),
      username_colour,
      message_colour,
    }
  }
}

pub fn recreate_chat<T: Into<String>>(
  deleted_message_id: T,
  past_chat_messages: &mut Vec<ChatMessage>,
  twitch: &mut TwitchEventSubApi,
  emote_buffer: &mut HashMap<String, u32>,
) {
  let _ = Command::new("clear").output().unwrap();
  let deleted_message_id = deleted_message_id.into();
  past_chat_messages.retain(|c| c.id != deleted_message_id);
  past_chat_messages.iter().for_each(|message| {
    message.print(twitch, emote_buffer);
  })
}

fn main() {
  let keys = TwitchKeys::from_secrets_env().unwrap();
  let redirect_url = "http://localhost:3000";

  println!("Owlbot booting up!");
  let twitch = TwitchEventSubApi::builder(keys.clone())
    .set_redirect_url(redirect_url)
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    //  .is_run_remotely()
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
      Subscription::PermissionReadModerator,
      //Subscription::ChannelPollBegin,
      //Subscription::ChannelPollProgress,
      //Subscription::ChannelPollEnd,
      //Subscription::ChannelGoalBegin,
      //Subscription::ChannelGoalProgress,
      //Subscription::ChannelGoalEnd,
      Subscription::ChannelHypeTrainBegin,
      Subscription::ChannelHypeTrainProgress,
      Subscription::ChannelHypeTrainEnd,
      //Subscription::ChannelShoutoutCreate,
      //Subscription::ChannelShoutoutReceive,
      Subscription::ChatMessage,
      //Subscription::BanTimeoutUser,
      Subscription::PermissionDeleteMessage,
      Subscription::PermissionReadChatters,
      Subscription::PermissionSendAnnouncements,
      Subscription::ModeratorDeletedMessage,
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
    .build();

  let mut twitch = twitch.unwrap();
  println!("Owlbot has been equipped!");

  let mut bots_recently_vanquished = 0;
  let mut time_since_last_vanquish = Instant::now();

  let mut emote_buffer = HashMap::new();
  let mut rank_buffer = HashMap::new();

  let mut holy_counter = 0;
  let mut heckies_counter = 0;

  let mut counter_cooldown = Instant::now();

  if let Ok(kitty_data) = fs::read_to_string("kitty_emotes") {
    for line in kitty_data.lines() {
      match line.split_whitespace().collect::<Vec<_>>().to_vec()[..] {
        [idx, emote_id] => {
          emote_buffer.insert(emote_id.to_string(), idx.parse::<u32>().unwrap());
        }
        _ => {}
      }
    }
  }

  if let Ok(viewer_ranks) = fs::read_to_string(RANK_BUFFER_FILE) {
    for line in viewer_ranks.lines() {
      match line.split_whitespace().collect::<Vec<_>>().to_vec()[..] {
        [viewer, rank_num] => {
          rank_buffer.insert(viewer.to_string(), rank_num.parse::<u32>().unwrap());
        }
        _ => {}
      }
    }
  }

  if let Ok(counters) = fs::read_to_string(COUNTERS_FILE) {
    for line in counters.lines() {
      match line.split_whitespace().collect::<Vec<_>>().to_vec()[..] {
        ["Holy", count] => {
          holy_counter = count.parse::<u128>().unwrap();
        }
        ["Heckies", count] => {
          heckies_counter = count.parse::<u128>().unwrap();
        }
        _ => {}
      }
    }
  }

  if let Ok(chatters) = twitch.get_chatters() {
    for chatter in chatters.data {
      if !rank_buffer.contains_key(&chatter.name) {
        rank_buffer.insert(chatter.name, 0);
      }
    }
  }

  // Happy -  English Britsh (Piper Semaine PrudenceMedium Female) - Happy TTS
  // Nervous - Piper Jenny Medium Female
  // NoFun VCTK p236 medium
  // Clear - Piper Amy Low female
  // Robot - RHVoice Slt Female
  // Calm - Piper Kathleen Low Female
  // Polite - Piper HFC Medium female
  // Cute - VCTK p236 medium

  // piper lessac high female - seems legit

  {
    let mut rng = thread_rng();

    let mut all_messages = HashMap::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut recent_loops: u32 = 0;
    let mut duration = 0;

    let mut followers_in_last_10secs = 0;
    let mut follower_timer = 0.0;
    let delta_time = Instant::now();

    let mut past_chat_messages: Vec<ChatMessage> = Vec::new();

    let mut tts_queue: Vec<String> = Vec::new();
    let mut last_message_spoken = Instant::now();
    let mut wait_duration = 5;

    let reward_response = twitch.create_custom_reward(CreateCustomReward {
      title: "TestCustomReward".to_string(),
      cost: 500,
      ..Default::default()
    });
    if let Ok(rewards) = reward_response {
      if rewards.data.len() > 0 {
        let _ = twitch.delete_custom_reward(rewards.data[0].id.to_owned());
      }
    }

    loop {
      if recent_loops > 100 {
        duration = 1;
      } else {
        recent_loops += 1;
      }

      follower_timer -= delta_time.elapsed().as_secs_f32();
      if follower_timer <= 0.0 {
        if followers_in_last_10secs > 10 {
          follower_timer = 120.0;
          let _ = twitch.send_chat_message("Warning: Channel may be current being follow botted!");
        } else {
          followers_in_last_10secs = 0;
          follower_timer = 10.0;
        }
      }

      if tts_queue.len() > 0 && last_message_spoken.elapsed().as_secs() > wait_duration {
        let text = tts_queue.remove(0);
        wait_duration = (text.len() / 100 * 5).max(5).min(18) as u64;
        run_tts(text);
        last_message_spoken = Instant::now();
      }

      if bots_recently_vanquished > 0 {
        if time_since_last_vanquish.elapsed().as_secs_f32() > 30.0 {
          let _ = twitch.send_chat_message(format!(
            "{} bot/s were sent to Owlkatraz, give OwlBot many pats.",
            bots_recently_vanquished
          ));
          bots_recently_vanquished = 0;
        }
      }

      for message in twitch.receive_all_messages(Some(Duration::from_millis(duration))) {
        recent_loops = 0;
        match message {
          ResponseType::Ready => {
            println!("Owlbot is eager to send bots to Owlkatraz!");
          }
          ResponseType::Event(event) => {
            match event {
              Event::Raid(raid_data) => {
                println!(
                  "Raid from {} with {} viewers!",
                  raid_data.from_broadcaster.name, raid_data.viewers
                );
                if raid_data.viewers >= 5 {
                  let _ =
                    twitch.send_chat_message(format!("!so {}", raid_data.from_broadcaster.name));
                }
              }
              Event::Follow(follow_data) => {
                followers_in_last_10secs += 1;
              }
              Event::AdBreakBegin(break_data) => {
                //twitch.send_chat_message(format!(
                //  "A {}min Ad has attacked! I try my best to not do anything interesting.",
                //  break_data.duration_seconds / 60
                //));
              }
              Event::MessageDeleted(deleted_message) => {
                println!("Message was deleted ID: {}", deleted_message.message_id);
                recreate_chat(
                  deleted_message.message_id,
                  &mut past_chat_messages,
                  &mut twitch,
                  &mut emote_buffer,
                );
              }
              Event::ChannelPointsAutoRewardRedeem(auto_redeem) => {
                let message = auto_redeem.message.text;
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

                if title.contains("TTS") {
                  if input.split_whitespace().count() > 1 {
                    tts_queue.push(format!("{} says {}", user, input));
                  } else {
                    tts_queue.push(format!("{}", input));
                  }
                }

                let mut great_fimsh_points: i32 = 0;
                if let Some(viewer_num) = rank_buffer.get_mut(&user) {
                  let mut points = 0;
                  if title.contains("RankUp") {
                    points = (rng.gen::<f32>() * 3.0).floor() as u32;
                    *viewer_num += points;
                    if points > 0 {
                      let _ = twitch.send_chat_message(format!(
                        "{}'s rank went up a little bit! (+{}P)",
                        user, points
                      ));
                    } else {
                      great_fimsh_points = (rng.gen::<f32>() * 3.0).ceil() as i32;
                      if great_fimsh_points == 3 {
                        *viewer_num -= 4;
                        great_fimsh_points += 1;
                        let _ = twitch.send_chat_message(format!("{}'s rank got stuck and then was help by the great fimsh, so it gave some of it's points to the great fimsh!", user));
                      } else {
                        let _ = twitch.send_chat_message(format!(
                          "{}'s rank didn't budge because the great fimsh stole it!",
                          user
                        ));
                      }
                    }
                  }
                  if title.contains("RankDown") {
                    great_fimsh_points =
                      -((rng.gen::<f32>() * 4.0).floor() as i32).max(*viewer_num as i32);

                    let _ = twitch.send_chat_message(format!(
                      "The great fimsh's rank went down a little bit! (-{}P)",
                      great_fimsh_points
                    ));
                  }

                  if title.contains("Feed the fimsh") {
                    *viewer_num += 1;
                    points = 1;
                  }
                  if title.contains("Feed the fimsh! x10") {
                    *viewer_num += 8;
                    points = 8;
                  }

                  if points != 0 {
                    let mut file = fs::File::create(RANK_BUFFER_FILE).unwrap(); //options()
                                                                                // .append(false)
                                                                                //   .create(true)
                                                                                // .open(RANK_BUFFER_FILE)
                                                                                //.unwrap();
                    let mut rank_buffer_string = String::new();
                    for (key, value) in rank_buffer.iter() {
                      rank_buffer_string = format!("{}\n{} {}", rank_buffer_string, key, value);
                    }

                    file
                      .write_all(format!("{}\n", rank_buffer_string).as_bytes())
                      .unwrap();
                  }

                  if great_fimsh_points != 0 {
                    if let Some(great_fimsh_number) = rank_buffer.get_mut(THE_GREAT_FIMSH) {
                      *great_fimsh_number =
                        (*great_fimsh_number as i32 + great_fimsh_points).max(0) as u32;
                      let _ = twitch.send_chat_message(format!(
                        "The great fimsh now possesses {}P",
                        great_fimsh_number
                      ));
                    }
                  }
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
                  gifty.user.name.unwrap_or("Anonymous".to_owned()),
                  gifty.total,
                  gifty.tier
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
                println!("Train Begin: {:?}", hype_train);
              }
              Event::HypeTrainProgress(train_progress) => {
                println!("Train Progress: {:?}", train_progress);
              }
              Event::HypeTrainEnd(hype_end) => {
                println!("The hype train ended at level {}!", hype_end.level);
              }
              Event::PredictionBegin(prediction_begin) => {
                //println!("{:#?}", prediction_begin);
              }
              Event::PredictionProgress(prediction_progress) => {
                //println!("{:#?}", prediction_progress);
              }
              Event::PredictionEnd(prediction_end) => {
                // println!("{:#?}", prediction_end);
              }
              Event::PredictionLock(prediction_lock) => {
                //println!("{:#?}", prediction_lock);
              }
              Event::PollBegin(begin_data) => {
                // println!("{:#?}", begin_data);
              }
              Event::PollProgress(progress_data) => {
                //println!("{:#?}", progress_data);
              }
              Event::PollEnd(end_data) => {
                //println!("{:#?}", end_data);
              }
              Event::HypeTrainBegin(begin_train) => {}
              Event::HypeTrainProgress(train_data) => {
                println!("{:?}", train_data);
              }
              Event::HypeTrainEnd(train_data) => {
                println!("{:?}", train_data);
              }
              Event::ChatMessage(message_data) => {
                let chat_message = ChatMessage::from(&message_data);
                let username = message_data.chatter.name;
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
                  && !rank_buffer.contains_key(&username)
                  && username.to_lowercase() != STREAM_ACCOUNT
                {
                  let is_link = lower_message
                    .split('.')
                    .skip(1)
                    .any(|s| s.len() > 1 && s.chars().take(2).all(char::is_alphabetic));

                  let sus_words = [
                    "cheap",
                    "view",
                    "streamrise",
                    "onlyfans",
                    "http",
                    "promotion",
                    "activate",
                    ".ly",
                    ".com",
                    ".to",
                    "free",
                    ".store",
                    ".xyz",
                    ".org",
                    "hosthub",
                  ];

                  if is_link
                    || sus_words
                      .iter()
                      .filter(|sussy| lower_message.contains(*sussy))
                      .count()
                      > 1
                  {
                    if let Ok(_) = twitch.delete_message(&message_id) {
                      bots_recently_vanquished += 1;
                      time_since_last_vanquish = Instant::now();
                      recreate_chat(
                        message_id,
                        &mut past_chat_messages,
                        &mut twitch,
                        &mut emote_buffer,
                      );
                    }

                    continue;
                  }
                }

                if username.to_lowercase() != STREAM_ACCOUNT {
                  chat_message.print(&mut twitch, &mut emote_buffer);
                  past_chat_messages.push(chat_message);
                  if past_chat_messages.len() > 20 {
                    past_chat_messages.remove(0);
                  }
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

                if !rank_buffer.contains_key(&username) {
                  rank_buffer.insert(username.to_owned(), 0);
                }

                if message.contains("modcheck") {
                  let _ = twitch.send_chat_message_with_reply(
                    "Owlbat is here to mod!",
                    Some(message_id.to_owned()),
                  );
                }

                if message.as_bytes()[0] as char == '!' {
                  match ChatCommands::is_command(&String::from_utf8_lossy(&message.as_bytes()[1..]))
                  {
                    (Some(command), None, parameters) => {
                      match command {
                        ChatCommands::Hello => {
                          let _ =  twitch.send_chat_message(format!("Welcome to the stream {}! owlkal1LHand owlkal1Leye owlkal1Yap owlkal1Reye owlkal1RHand", username));
                        }
                        ChatCommands::Meat => {
                          let _ = twitch.send_chat_message(format!(
                        "Find out what happened to your meat today! https://youtu.be/7tScAyNaRdQ"
                      ));
                        }
                        ChatCommands::Processing => {
                          let _ =   twitch.send_chat_message(format!(
                          "Neat little programming program for protoyping, check it out: https://processing.org/"
                     ));
                        }
                        // Second discord that looks normal is actually some
                        // kind of special characters (Cyrillic)
                        ChatCommands::Discord => {
                          let _ = twitch.send_chat_message(format!(
                            "Join Owl's discord at: https://discord.gg/8pdfBzGbgB"
                          ));
                        }
                        //ChatCommands::Optical => {
                        //  twitch.send_chat_message(format!(
                        //"Optical illusion here: https://media.discordapp.net/attachments/691453928709292032/1241676080226762814/opticalIllusion.png?ex=66559c76&is=66544af6&hm=7c46b66eba9defe28cd42ab7a139af97b9c9646fc7ce0634cea49641cada8262&=&format=webp&quality=lossless&width=907&height=510"
                        //));
                        //  }
                        ChatCommands::Throne => {
                          let _ = twitch.send_chat_message(format!(
                            "Throne wishlist: https://throne.com/owlkaline"
                          ));
                        }
                        ChatCommands::Owlyfans => {
                          let _ = twitch.send_chat_message(format!(
                        "To Support the Owl more, Support on OwlyFans: https://ko-fi.com/owlkaline"
                      ));
                        }
                        ChatCommands::HowToQuote => {
                          let _ = twitch.send_chat_message_with_reply(
                            "Type \"don\'t quote\" to quote your previous message!".to_string(),
                            Some(message_id),
                          );
                        }
                        ChatCommands::Quote => {
                          if let Ok(quotes) = fs::read_to_string(QUOTES) {
                            let lines = quotes.lines().collect::<Vec<_>>();

                            let line_count = lines.len() as f32;
                            let rng = rng.gen::<f32>();

                            let idx = (rng * line_count).floor() as usize;

                            let quote = lines[idx];
                            let _ = twitch
                              .send_chat_message_with_reply(quote.to_string(), Some(message_id));
                          } else {
                            let _ = twitch.send_chat_message_with_reply(format!("The quotes were cleared! Make your own quote by sending the quote in chat, then have your next message contain \"don\'t quote me\" to create a quote."), Some(message_id));
                          }
                        }
                        ChatCommands::Commands => {
                          let mut all_commands = "The Following commands exist:\n".to_string();
                          for variant in ChatCommands::all_variants() {
                            all_commands = format!("{}!{:?}\n", all_commands, variant);
                          }
                          let _ = twitch.send_chat_message(all_commands);
                        }
                        ChatCommands::Ram => {
                          sys.refresh_all();

                          let ram_used = sys.used_memory() as f32 / 1000000000.0;
                          let total_ram = sys.total_memory() as f32 / 1000000000.0;

                          let _ = twitch.send_chat_message(format!(
                            "Current Ram: {:.1}/{:.1} Gb ({:.0}%)",
                            ram_used,
                            total_ram,
                            (ram_used / total_ram * 100.0).round()
                          ));
                        }
                        ChatCommands::Github => {
                          let _ = twitch.send_chat_message(format!(
                            "Owl's github can be found at: https://github.com/Owlkaline",
                          ));
                        }
                        ChatCommands::Lurk
                        | ChatCommands::Loork
                        | ChatCommands::Luwurk
                        | ChatCommands::Lurking => {
                          let _ = twitch.send_chat_message_with_reply(
                            format!("Thanks for coming by, appreciate the lurk {}!", username),
                            Some(message_id),
                          );
                        }
                        ChatCommands::DotFiles => {
                          let _ = twitch.send_chat_message(format!(
                        "You can Owl's linux dot files here: https://github.com/Owlkaline/dotfiles"
                      ));
                        }
                        ChatCommands::Editor => {
                          let _ = twitch.send_chat_message(format!(
                      "I switch between Helix , Neovim and Zed currently, there is a redeem to make Owl use a new editor!"
                    ));
                        }
                        ChatCommands::Distro => {
                          let _ = twitch.send_chat_message(format!(
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
                          let _ = twitch.send_chat_message(format!("Owl is working on a Rust library that allows you to talk to the twitch API: https://github.com/owlkaline/TwitchEventSub-rs"));
                        }
                        ChatCommands::Fimsh => {
                          let _ = twitch.send_chat_message("owlkal1Fimsh".to_owned());
                        }

                        ChatCommands::Break => {
                          let _ = twitch.send_chat_message(
                            "Please break my chat bot, I'll thank you for it!".to_owned(),
                          );
                        }
                        ChatCommands::Throbber => {
                          let _ = twitch.send_chat_message(
                            "Time for them blue pills owlkal1LHand owlkal1RHand".to_owned(),
                          );
                        }
                        ChatCommands::VioletCrumble => {
                          let _ = twitch.send_chat_message("owlkal1OC");
                        }
                        ChatCommands::SO | ChatCommands::ShoutOut => {
                          let moderators = twitch
                            .get_moderators()
                            .unwrap()
                            .data
                            .iter()
                            .map(|u| u.name.to_owned())
                            .collect::<Vec<_>>();

                          if moderators.contains(&username)
                            || username.eq_ignore_ascii_case(STREAM_ACCOUNT)
                          {
                            if parameters.len() > 0 {
                              let _ = twitch.send_chat_message(format!(
                                "{} is an awesome streamer, follow them at http://twitch.tv/{}",
                                parameters[0], parameters[0],
                              ));
                              let _ = twitch.send_announcement(
                                format!(
                                  "{} is an awesome streamer, follow them at http://twitch.tv/{}",
                                  parameters[0], parameters[0],
                                ),
                                None::<String>,
                              );
                            }
                          }
                        }
                        ChatCommands::QOD | ChatCommands::QuestionOfTheDay => {
                          if let Ok(questions) = fs::read_to_string(QOD) {
                            let msg = questions
                              .lines()
                              .find(|line| !line.starts_with("//"))
                              .unwrap_or("Owl messed something up");
                            let _ = twitch.send_chat_message(msg);
                            println!("QOD: {}", msg);
                          } else {
                            let _ = twitch.send_chat_message("Question of the day, what a meme!");
                          }

                          //  for line in lines {
                          //    if line[0] != "/" {
                          //      let _ = twitch.send_chat_message(line);
                          //      found_quote = true;
                          //      break;
                          //    }
                          //  }
                          //}
                          //
                          //if !found_quote {
                          //  let _ = twitch.send_chat_message("");
                          //}

                          //if let Err(e) = twitch.send_chat_message(
                          //  //"What's your favourite rpg game and why?", //"What is the most vivid and coolest dream you have had?",
                          //  // "What do you think a cool fimsh redeem would be?",
                          //  //"What is your favourite flower?",
                          //  //   "What is your greatest goal for the next year?",
                          //  //"What do you do when one of your friends is sad, to help thm feel a little bit more comfy or less sad?"
                          //  "What tricks do you use to get take away less?",
                          //) {
                          //  println!("Error sending message: {:?}", e);
                          //}
                          //twitch.send_chat_message(
                          //  "What is the game mechanic you have most enjoyed in a 2D game?",
                          //);
                          //twitch.send_chat_message("Would you be interested in watching my vods if I was to put them onto a Vods channel on youtube?");
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
                          let _ = twitch.send_chat_message(
                            "Owl uses the Dracula theme! (https://draculatheme.com/)",
                          );
                        }
                        ChatCommands::Bones => {
                          let _ =      twitch.send_chat_message("IF YOURE NOT HAVING A GOOD TIME CRACK YOUR BONES ITS GOOD FOR YOU AND BONES ARE NOT REAL ANYWAY");
                        }
                        ChatCommands::Train => {
                          let _ = twitch.send_chat_message("choo chooooo");
                        }
                        ChatCommands::Bread => {
                          let _ = twitch.send_chat_message("🍞 I knead your loaf.");
                        }
                        ChatCommands::Rank => {
                          if let Some(viewer_num) = rank_buffer.get_mut(&username) {
                            let rank = match *viewer_num {
                              400.. => RankVariety::Holee,
                              300.. => RankVariety::Nean,
                              250.. => RankVariety::LongFimsh,
                              200.. => RankVariety::Fimsh,
                              170.. => RankVariety::Trackmaniac,
                              140.. => RankVariety::Explosive,
                              130.. => RankVariety::Creamed,
                              120.. => RankVariety::Sour,
                              110.. => RankVariety::Edged,
                              90.. => RankVariety::DirtyBinChicken,
                              70.. => RankVariety::BinChicken,
                              50.. => RankVariety::SmoothedVeganMeat,
                              30.. => RankVariety::SmoothedMeat,
                              20.. => RankVariety::VeganGarden,
                              10.. => RankVariety::CarnivorousGarden,
                              5.. => RankVariety::Uncommon,
                              _ => RankVariety::Common,
                            };
                            let _ = twitch.send_chat_message_with_reply(
                              format!(
                                "{} is a {} variety viewer ({}P)",
                                username, rank, viewer_num
                              ),
                              Some(message_id),
                            );
                          }
                        }
                        ChatCommands::Ranks => {
                          let mut response = "The available ranks are as follows: ".to_string();
                          for variant in RankVariety::all_variants() {
                            response = format!("{}, {}", response, variant);
                          }
                          let _ = twitch.send_chat_message_with_reply(response, Some(message_id));
                        }
                        ChatCommands::Pronouns => {
                          let msg = "Owl's pronouns are She/Her, thanks!";
                          let _ = twitch.send_chat_message(msg);
                          println!("{}", msg);
                        }
                        ChatCommands::OwlBeCringe => {
                          if let Ok(cringes) = fs::read_to_string(OWL_CRINGES) {
                            let lines = cringes.lines().collect::<Vec<_>>();

                            let line_count = lines.len() as f32;
                            let rng = rng.gen::<f32>();

                            let idx = (rng * line_count).floor() as usize;

                            let cringe = lines[idx];
                            let _ = twitch.send_chat_message(cringe.to_string());
                          } else {
                            let _ = twitch.send_announcement(
                              format!(
                              "Owl's out of cringes, so you best go follow tiwtch.tv/bixiavt now!"
                            ),
                              None::<String>,
                            );
                          }
                        }
                        ChatCommands::Holee => {
                          if counter_cooldown.elapsed().as_secs() > 5 {
                            counter_cooldown = Instant::now();
                            holy_counter += 1;
                            let _ = twitch.send_chat_message(format!(
                              "Owl has said holy {} times!",
                              holy_counter
                            ));
                            let mut file = fs::File::create(COUNTERS_FILE).unwrap();
                            let counters =
                              format!("Holy {}\nHeckies {}", holy_counter, heckies_counter);
                            file.write_all(counters.as_bytes()).unwrap();
                            file.flush().unwrap();
                          }
                        }
                        ChatCommands::Heckies => {
                          if counter_cooldown.elapsed().as_secs() > 5 {
                            counter_cooldown = Instant::now();
                            heckies_counter += 1;
                            let _ = twitch.send_chat_message(format!(
                              "Owl has said heckies {} times!",
                              heckies_counter
                            ));
                            let mut file = fs::File::create(COUNTERS_FILE).unwrap();
                            let counters =
                              format!("Holy {}\nHeckies {}", holy_counter, heckies_counter);
                            file.write_all(counters.as_bytes()).unwrap();
                            file.flush().unwrap();
                          }
                        }
                      }
                    }

                    (None, Some(close), _) => {
                      let _ = twitch.send_chat_message(format!(
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
          ResponseType::Close => {
            error!("Websockets decided to close.");
            break;
            //twitch.restart_websockets();
          }
          ResponseType::Error(event_sub_error) => {
            println!("{:?}", event_sub_error);
            error!("{:?}", event_sub_error);
          }
          ResponseType::RawResponse(raw_data) => {
            let response = format!("RAW response: {}", raw_data);
            warn!("{}", response.to_owned());
            println!("{}", response);
          }
          _ => {}
        }
      }
    }
  }
}
