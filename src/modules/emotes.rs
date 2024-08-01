use base64::prelude::*;
use image::{codecs::gif::GifDecoder, AnimationDecoder};
use twitch_eventsub::{Emote, TwitchApi, TwitchEventSubApi};

use std::{collections::HashMap, fs};

use std::io::{Cursor, Read, Write};

use crate::EMOTE_BUFFER_FILE;

pub fn print_emote(
  twitch: &mut TwitchEventSubApi,
  emote: Emote,
  emote_buffer: &mut HashMap<String, u32>,
) {
  if let Some(id) = emote_buffer.get(&emote.id) {
    print!("\x1b_Ga=p,i={},q=1\x1b\\", id);
  } else {
    let highest_key = emote_buffer.iter().map(|(k, v)| *v).max().unwrap_or(0) + 1;

    let channel_id = emote.owner_id.unwrap();
    if let Ok(emotes) = twitch.get_channel_emotes(channel_id) {
      let emote_data = emotes.from_id(emote.id.to_owned());
      if emote_data
        .format
        .contains(&twitch_eventsub::EmoteFormat::Animated)
      {
        // a=f
        let emote_url = emotes.from_emote(&emote_data, true);
        let new_image_data = attohttpc::get(emote_url.to_owned()).send().unwrap();

        if let Ok(gif) = GifDecoder::new(Cursor::new(new_image_data.bytes().unwrap())) {
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
          print!("\x1b_Ga=p,i={},q=1\x1b\\", highest_key);
          print!("\x1b_Ga=a,s=3,q=1,v=1,i={}\x1b\\", highest_key);
          let id = emote.id;
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
      } else {
        let emote_url = emotes.from_emote(&emote_data, false);
        let new_image_data = attohttpc::get(emote_url).send().unwrap();

        print!(
          "\x1b_Gi={},q=1,f=100;{}\x1b\\",
          highest_key,
          BASE64_STANDARD.encode(new_image_data.bytes().unwrap())
        );
        print!("\x1b_Ga=p,i={},q=1\x1b\\", highest_key);

        let id = emote.id;
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
  }
}
