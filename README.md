# Hodiny

Striking clock.

Each strike is a configurable MIDI event, synthesised using a SF2 soundfont.

Hard-coded Westminster chimes are also included, when setting striking to the `kind = 2`.

## Soundfont

Legendary [Jeux14.sf2](https://www.realmac.info/jeux1.htm) organ soundfont which includes sounds of bells is recommended and required for the sample configurations to work.

Download it manually and unpack or get a copy from [github](https://github.com/nickbailey/smrg-live/blob/master/config/includes.chroot/usr/local/share/soundfonts/jeux14.sf2), please.

## Usage

```bash
$ echo "Please get the soundfount first." 
$ ./hodiny [hodiny.toml]
```

Please see `script/` for permanent scheduling hints for linux and windows OS.
## Dependencies
* `rustysynth` ... from sf2 to synthesized waveform
* `tinyaudio` ... plays the waveform