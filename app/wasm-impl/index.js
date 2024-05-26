export function loadFile(b64) {
  MIDI.loadPlugin({
    soundfontUrl: "https://gleitz.github.io/midi-js-soundfonts/MusyngKite/",
    onsuccess: function () {
      console.log("player...");
      MIDI.Player.loadFile("data:audio/midi;base64," + b64, function () {
        MIDI.Player.start();
        MIDI.Player.BPM = 60;
      });
    },
  });
}
