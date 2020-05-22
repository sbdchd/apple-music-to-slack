# apple music to slack

> Update your slack status with the current song playing via the Music.app

## Setup

1.  setup the slack app

    1. create a new app https://api.slack.com/apps?new_app=1 providing a name
       and selecting the desired Slack Workspace that you're going to run apple
       music to slack on.

    2. Under "Add features and functionality" select the "Permissions" section

    3. scroll down to "User Token Scopes" and add `users.profile:write`

    4. scoll up to the top of the page and click "Install App to Workspace".

    5. copy the `OAuth Access Token`, this will be used as the `SLACK_SECRET_TOKEN`

2.  clone the repo & compile with `cargo build`

3.  Run the binary with the env var `SLACK_SECRET_TOKEN` set to your `OAuth Access Token`

    Options:

    - run via shell

      ```sh
      export SLACK_SECRET_TOKEN=xoxp-11111-11111-11111-111111111111
      while true; do
          ./target/debug/apple-music-to-slack;
          # slack rates limit at anything less than 5 requests/second
          sleep 10;
      done
      ```

    - run via `launchd`

      ```sh
      cp ./target/debug/apple-music-to-slack /usr/local/bin/
      cp xyz.dignam.apple-music-to-slack.plist ~/Library/LaunchAgents
      launchctl load ~/Library/LaunchAgents/xyz.dignam.apple-music-to-slack.plist

      # to disable the launchd job
      launchctl unload ~/Library/LaunchAgents/xyz.dignam.apple-music-to-slack.plist
      ```

4.  Success! 🎶

## prior art

- <https://github.com/ocxo/slacktunes>
- <https://github.com/josegonzalez/python-slack-tunes>
