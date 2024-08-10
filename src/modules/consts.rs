pub const QUOTES: &str = "quotes";
pub const QOD: &str = "qod";
pub const OWL_CRINGES: &str = "owlcringe";
pub const STREAM_ACCOUNT: &str = "owlkalinevt";
pub const STREAM_TWITCH_ID: &str = "122297321";

pub const CONNECTION: &'static str = "ws://irc-ws.chat.twitch.tv:80";
pub const CONNECTION_EVENTS: &'static str =
  "wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=30";

pub const SUBSCRIBE_URL: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";
pub const SEND_MESSAGE_URL: &str = "https://api.twitch.tv/helix/chat/messages";

pub const COMMAND_PREFIX: &str = "tmi.twitch.tv";
pub const JOIN: &str = "JOIN";
pub const PASSWORD: &str = "PASS";
pub const USERNAME: &str = "NICK";
pub const MESSAGE: &str = "PRIVMSG";
pub const CAPABILITY_REQUIREMENTS: &str = "CAP REQ";
pub const PING: &str = "PING";

pub const EMOTE_BUFFER_FILE: &str = "kitty_data";
pub const RANK_BUFFER_FILE: &str = "veiwer_rank";
