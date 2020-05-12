var x = Application("Music").currentTrack;
JSON.stringify({ name: x.name(), artist: x.artist(), album: x.album() });
