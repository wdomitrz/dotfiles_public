#!/usr/bin/env bash
################################################################
# Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################
set -euo pipefail

tts_command=(uvx pocket-tts generate --quiet)

use_stdin=false && verbose=false && dry_run=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --stdin) use_stdin="$2" && shift 2 ;;
    --verbose) verbose="$2" && shift 2 ;;
    --dry-run) dry_run="$2" && shift 2 ;;
    *) echo "Unsupported option: $2" && exit 1 ;;
  esac
done

if "${use_stdin}"; then
  input_command=(cat)
elif command -v pasta > /dev/null; then
  input_command=(pasta)
elif command -v pbpaste > /dev/null; then
  input_command=(pbpaste)
elif command -v xclip > /dev/null; then
  input_command=(xclip -out -selection clipboard)
elif command -v xsel > /dev/null; then
  input_command=(xsel --clipboard --output)
else
  echo "Error: no input_command" >&2 && exit 1
fi
if "${verbose}"; then
  echo "input_command: ${input_command[*]}"
fi

if command -v afplay > /dev/null; then
  play_sound_command=(afplay)
elif command -v paplay > /dev/null; then
  play_sound_command=(paplay)
elif command -v aplay > /dev/null; then
  play_sound_command=(aplay)
elif command -v mpv > /dev/null; then
  play_sound_command=(mpv --no-terminal)
elif command -v ffplay > /dev/null; then
  play_sound_command=(ffplay -nodisp -autoexit -loglevel quiet)
else
  echo "Error: no play_sound_command" >&2 && exit 1
fi
if "${verbose}"; then
  echo "play_sound_command: ${play_sound_command[*]}"
fi

temp_audio="$(mktemp /tmp/speech_XXXXXX.wav)"
# shellcheck disable=SC2064
trap "rm -f '${temp_audio}'" EXIT SIGINT SIGTERM

if ! "${dry_run}"; then
  "${input_command[@]}" | xargs -0 -I {} "${tts_command[@]}" --output-path "${temp_audio}" --text "{}"
fi

if ! "${dry_run}"; then
  "${play_sound_command[@]}" "${temp_audio}"
fi
