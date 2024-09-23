use base64::prelude::*;
use image::{codecs::gif::GifDecoder, AnimationDecoder};
use twitcheventsub::{Emote, EmoteBuilder, TwitchApi, TwitchEventSubApi};

use std::{collections::HashMap, fs};

use std::io::{Cursor, Read, Write};

use crate::EMOTE_BUFFER_FILE;

pub fn print_if_loaded<T: Into<String>>(
  id: T,
  emote_buffer: &mut HashMap<String, u32>,
  chained_emote: bool,
) -> bool {
  if let Some(id) = emote_buffer.get(&id.into()) {
    print!(
      "\x1b_Ga=p,i={},q=1,r=1,H={}\x1b\\",
      id,
      if chained_emote { -1 } else { 0 }
    );
    true
  } else {
    false
  }
}

pub fn print_animated_emote<S: Into<String>, T: Into<String>>(
  emote_url: S,
  id: T,
  emote_buffer: &mut HashMap<String, u32>,
) {
  let new_image_data = attohttpc::get(emote_url.into()).send().unwrap();

  if let Ok(gif) = GifDecoder::new(Cursor::new(new_image_data.bytes().unwrap())) {
    let highest_key = emote_buffer.iter().map(|(k, v)| *v).max().unwrap_or(0) + 1;
    let frames = gif.into_frames().collect_frames().unwrap();
    let mut first = true;
    for frame in frames {
      let (n, d) = frame.delay().numer_denom_ms();
      let delay = n as f32 / d as f32;
      if first {
        print!(
          "\x1b_Gi={},a=t,s={},v={},q=1,f=32;{}\x1b\\",
          highest_key,
          frame.buffer().width(),
          frame.buffer().height(),
          BASE64_STANDARD.encode(
            &frame
              .buffer()
              .bytes()
              .map(|a| a.unwrap())
              .collect::<Vec<_>>()[..]
          )
        );
        first = false;
      }
      print!(
        "\x1b_Gi={},s={},v={},z={},q=1,a=f,f=32;{}\x1b\\",
        highest_key,
        frame.buffer().width(),
        frame.buffer().height(),
        delay,
        BASE64_STANDARD.encode(
          &frame
            .buffer()
            .bytes()
            .map(|a| a.unwrap())
            .collect::<Vec<_>>()[..]
        )
      );
    }
    print!("\x1b_Ga=p,i={},q=1,r=1\x1b\\", highest_key);
    print!("\x1b_Ga=a,s=3,q=1,v=1,i={}\x1b\\", highest_key);
    let id = id.into();
    emote_buffer.insert(id.to_owned(), highest_key);
    let mut file = fs::File::options()
      .append(true)
      .create(true)
      .open(EMOTE_BUFFER_FILE)
      .unwrap();
    file
      .write_all(format!("{} {}\n", highest_key, id).as_bytes())
      .unwrap();
  }
}

pub fn print_static_emote<S: Into<String>, T: Into<String>>(
  emote_url: S,
  id: T,
  emote_buffer: &mut HashMap<String, u32>,
) {
  let highest_key = emote_buffer.iter().map(|(k, v)| *v).max().unwrap_or(0) + 1;

  let new_image_data = attohttpc::get(emote_url.into()).send().unwrap();

  print!(
    "\x1b_Gi={},q=1,f=100;{}\x1b\\",
    highest_key,
    BASE64_STANDARD.encode(new_image_data.bytes().unwrap())
  );
  print!("\x1b_Ga=p,i={},q=1,r=1\x1b\\", highest_key);

  let id = id.into();
  emote_buffer.insert(id.to_owned(), highest_key);
  let mut file = fs::File::options()
    .append(true)
    .create(true)
    .open(EMOTE_BUFFER_FILE)
    .unwrap();
  file
    .write_all(format!("{} {}\n", highest_key, id).as_bytes())
    .unwrap();
}

pub fn print_emote(
  twitch: &mut TwitchEventSubApi,
  emote: Emote,
  emote_buffer: &mut HashMap<String, u32>,
  chained_emote: bool,
) {
  let id = &emote.id;
  if !print_if_loaded(id, emote_buffer, chained_emote) {
    if let Some(emote_url) = EmoteBuilder::builder()
      .animate_or_fallback_on_static()
      .dark()
      .scale3()
      .build(twitch, &emote)
    {
      if emote_url.animated {
        print_animated_emote(emote_url.url, id, emote_buffer);
      } else {
        print_static_emote(emote_url.url, id, emote_buffer);
      }
    };
  }
}
