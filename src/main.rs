use log::info;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

const JS: &str = include_str!("get_current_track.js");

/// Music themed emojis
enum Emoji {
    Notes,
    Headphones,
    ControlKnobs,
    MusicalScore,
    Violin,
    Saxophone,
    MusicalKeyboard,
}

impl Distribution<Emoji> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Emoji {
        match rng.gen_range(0, 7) {
            0 => Emoji::Notes,
            1 => Emoji::Headphones,
            2 => Emoji::ControlKnobs,
            3 => Emoji::MusicalScore,
            4 => Emoji::Violin,
            5 => Emoji::Saxophone,
            _ => Emoji::MusicalKeyboard,
        }
    }
}

impl std::string::ToString for Emoji {
    fn to_string(&self) -> String {
        match &self {
            Emoji::Notes => ":notes:".into(),
            Emoji::Headphones => ":headphones:".into(),
            Emoji::MusicalKeyboard => ":musical_keyboard:".into(),
            Emoji::ControlKnobs => ":control_knobs:".into(),
            Emoji::MusicalScore => ":musical_score:".into(),
            Emoji::Violin => ":violin:".into(),
            Emoji::Saxophone => ":saxophone:".into(),
        }
    }
}

impl std::convert::From<Emoji> for std::string::String {
    fn from(emoji: Emoji) -> String {
        emoji.to_string()
    }
}

#[derive(Debug, Deserialize)]
struct SongInfo {
    artist: String,
    name: String,
    album: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum CurrentSong {
    Playing(SongInfo),
    Paused(SongInfo),
    Stopped,
    Off,
}

#[derive(Debug)]
enum CurrentSongError {
    AppleScriptError(std::io::Error),
    AppleScriptExecutionError(std::process::Output),
    Utf8ParseError,
    JsonParsingError(serde_json::error::Error, String),
}

impl std::convert::From<std::io::Error> for CurrentSongError {
    fn from(err: std::io::Error) -> CurrentSongError {
        CurrentSongError::AppleScriptError(err)
    }
}

impl std::convert::From<std::str::Utf8Error> for CurrentSongError {
    fn from(_: std::str::Utf8Error) -> CurrentSongError {
        CurrentSongError::Utf8ParseError
    }
}

fn get_current_song() -> Result<CurrentSong, CurrentSongError> {
    let data = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(JS)
        .output()?;

    if data.status.success() {
        let stdout = str::from_utf8(&data.stdout)?;
        // output can have new lines at the end so we need to trim that off so
        // serde doesn't explode
        serde_json::from_str(stdout.trim())
            .map_err(|err| CurrentSongError::JsonParsingError(err, String::from(stdout)))
    } else {
        Err(CurrentSongError::AppleScriptExecutionError(data))
    }
}

#[derive(Debug, Serialize)]
struct SlackProfileStatus {
    /// text content of the status
    status_text: String,
    /// emoji to display
    status_emoji: String,
    /// unix time of when the status should expire
    status_expiration: u64,
}

#[derive(Debug, Serialize)]
struct SlackProfileUpdate {
    profile: SlackProfileStatus,
}

#[derive(Debug, Deserialize)]
struct SlackProfileUpdateResponse {
    ok: bool,
    error: Option<String>,
}

#[derive(Debug)]
enum SlackProfileUpdateError {
    FailedToSend,
    JsonParseError,
    RequestFailed(SlackProfileUpdateResponse),
}

/// https://api.slack.com/methods/users.profile.set
fn update_slack_status(
    secret: &str,
    status: SlackProfileStatus,
) -> Result<(), SlackProfileUpdateError> {
    let profile_update = SlackProfileUpdate { profile: status };

    let res: SlackProfileUpdateResponse = reqwest::Client::new()
        .post("https://slack.com/api/users.profile.set")
        .header(AUTHORIZATION, format!("Bearer {}", secret))
        .json(&profile_update)
        .send()
        .map_err(|_| SlackProfileUpdateError::FailedToSend)?
        .json()
        .map_err(|_| SlackProfileUpdateError::JsonParseError)?;

    match res.ok {
        true => Ok(()),
        false => Err(SlackProfileUpdateError::RequestFailed(res)),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
    #[structopt(long, env = "SLACK_SECRET_TOKEN")]
    slack_secret_token: String,
}

fn main() {
    env_logger::init();

    let Opt { slack_secret_token } = Opt::from_args();

    match get_current_song() {
        Ok(CurrentSong::Playing(song)) => {
            info!("song info: {:#?}", song);
            let now_unix_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("problem getting current time");

            let status = SlackProfileStatus {
                status_text: format!("{} by {}", song.name, song.artist),
                status_emoji: rand::random::<Emoji>().into(),
                status_expiration: (now_unix_time + Duration::from_secs(60)).as_secs(),
            };
            info!("updating status to {:#?}", status);
            let res = update_slack_status(&slack_secret_token, status);
            info!("update: {:#?}", res);
        }
        Ok(CurrentSong::Paused(song)) => {
            info!("song paused: {:#?}", song);
        }
        Ok(CurrentSong::Off) => {
            info!("music app not running");
        }
        Ok(CurrentSong::Stopped) => {
            info!("no song currently selected in music app");
        }
        Err(err) => {
            info!("error fetching song {:#?}", err);
        }
    }
}
