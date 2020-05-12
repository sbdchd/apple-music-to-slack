use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;
use std::str;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

const JS: &str = include_str!("get_current_track.js");

#[derive(Debug, Deserialize)]
struct CurrentSong {
    pub artist: String,
    pub name: String,
    pub album: String,
}

#[derive(Debug)]
enum CurrentSongError {
    AppleScriptError(std::io::Error),
    AppleScriptExecutionError(std::process::Output),
    Utf8ParseError,
    JsonParsingError(serde_json::error::Error, String),
}

fn get_current_song() -> Result<CurrentSong, CurrentSongError> {
    let data = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(JS)
        .output()
        .map_err(CurrentSongError::AppleScriptError)?;

    if data.status.success() {
        let stdout = str::from_utf8(&data.stdout).map_err(|_| CurrentSongError::Utf8ParseError)?;
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

/// https://api.slack.com/methods/users.profile.set
fn update_slack_status(secret: &str, status: SlackProfileStatus) -> Result<(), String> {
    let profile_update = SlackProfileUpdate { profile: status };

    let res: SlackProfileUpdateResponse = reqwest::Client::new()
        .post("https://slack.com/api/users.profile.set")
        .header(AUTHORIZATION, format!("Bearer {}", secret))
        .json(&profile_update)
        .send()
        .map_err(|_| "failed to send error")?
        .json()
        .map_err(|_| "failed to parse response as json")?;

    match res.ok {
        true => Ok(()),
        false => Err(format!("Request failed {:?}", res)),
    }
}

fn main() {
    let slack_secret_token = env::var("SLACK_SECRET_TOKEN").expect("SLACK_SECRET_TOKEN required");

    let cur_song = get_current_song().expect("problem fetching current song");

    let now_unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("problem getting current time");

    let status = SlackProfileStatus {
        status_text: format!("{} by {}", cur_song.name, cur_song.artist),
        status_emoji: ":notes:".into(),
        status_expiration: (now_unix_time + Duration::new(5 * 60, 0)).as_secs(),
    };

    let res = update_slack_status(&slack_secret_token, status);
    println!("{:?}", res);
}
