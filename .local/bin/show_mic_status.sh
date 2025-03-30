#!/usr/bin/env sh
exec notify-send "🎤 $(pactl get-source-mute @DEFAULT_SOURCE@)"
