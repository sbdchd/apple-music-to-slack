var app;
try {
  app = Application("Music");
} catch (e) {
  app = Application("iTunes");
}

var isRunning = app.running();
if (isRunning) {
  var playerState = app.playerState();
  if (playerState != "stopped") {
    var song = app.currentTrack;
    JSON.stringify({
      type: playerState === "playing" ? "Playing" : "Paused",
      name: song.name(),
      artist: song.artist(),
      album: song.album(),
    });
  } else {
    JSON.stringify({
      type: "Stopped",
    });
  }
} else {
  JSON.stringify({
    type: "Off",
  });
}
