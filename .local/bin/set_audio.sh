#!/usr/bin/env bash

function list_cards() {
  pactl list short cards | cut --field 2
}

function get_cards_profiles() {
  card="$1"
  pactl list cards |
    awk '/^Card /{f=0} f; /Name: '"${card}"'/{f=1}' |   # Get properties of a card
    awk '/Active Profile:/{f=0} f; /Profiles:$/{f=1}' | # Get profiles
    sed 's/^[ \t]*//' | cut -d ' ' -f 1 | sed 's/:$//g' # Select profiles only
}

function output_only() {
  grep 'output:' | (grep --invert-match 'input:' || true)
}

function input_only() {
  grep 'input:' | (grep --invert-match 'output:' || true)
}

function output_and_input() {
  grep 'output:' | grep 'input:'
}

function output_and_input_and_only_output() {
  input=$(cat)
  # shellcheck disable=SC2310
  echo "${input}" | output_and_input || true
  echo "${input}" | output_only
}

function off_filter() {
  grep '^off$'
}

function set_card_profile() {
  card="$1"

  case "$2" in
  out) profiles_filter=output_only ;;
  in) profiles_filter=input_only ;;
  both) profiles_filter=output_and_input_and_only_output ;;
  off) profiles_filter=off_filter ;;
  *) echo "Unsupported profile filter: $2" && return 1 ;;
  esac

  profile="$(get_cards_profiles "${card}" |
    "${profiles_filter}" | head --lines=1)"
  if [[ -z ${profile} ]]; then
    echo "No ${profiles_filter} match for ${card}"
    return
  fi

  echo "Setting ${profile} for ${card}"
  pactl set-card-profile "${card}" "${profile}"
}

function set_cards() {
  if [[ $# -lt "1" ]]; then
    echo "Usage: $0 <out_or_both_card> [<in_card>]"
  elif [[ $# -eq "1" ]]; then
    out_card_filter="$1"
    in_card_filter="$1"
  else
    out_card_filter="$1"
    in_card_filter="$2"
  fi

  for card in $(list_cards |
    grep --invert-match "${out_card_filter}" |
    grep --invert-match "${in_card_filter}"); do
    set_card_profile "${card}" off
  done

  out_card="$(list_cards |
    (grep "${out_card_filter}" || true) | head --lines=1)"
  in_card="$(list_cards |
    (grep "${in_card_filter}" || true) | head --lines=1)"

  if [[ ${out_card} == "${in_card}" ]]; then
    if [[ -z ${out_card} ]]; then
      echo "No card matching any of ${out_card} ${in_card}"
    else
      set_card_profile "${out_card}" both
    fi
  else
    if [[ -z ${out_card} ]]; then
      echo "No out card matching ${out_card_filter}"
    else
      set_card_profile "${out_card}" out
    fi
    if [[ -z ${in_card} ]]; then
      echo "No in card matching ${in_card}"
    else
      set_card_profile "${in_card}" in
    fi
  fi
}

function list_sinks() {
  pactl list short sinks | cut --field 2
}

function list_sources() {
  pactl list short sources | cut --field 2 |
    grep --invert-match "\.monitor$"
}

function disable_sink_or_source() {
  what="$1" # sink or source
  filter="$2"
  for which in $(list_"${what}"s | grep --invert-match "${filter}"); do
    echo "Muting ${what}: ${which}"
    pactl set-"${what}"-volume "${which}" 0
    pactl set-"${what}"-mute "${which}" 1
  done
}

function set_default_sink_or_source() {
  if [[ $# -lt "4" ]]; then
    echo "Usage: $0 <what - 'sink' or 'source'> <filter> <volume> <muted - 0 or 1>"
    return 1
  fi
  what="$1" # sink or source
  filter="$2"
  volume="$3"
  muted="$4" # 0 or 1

  which=$(list_"${what}"s |
    (grep "${filter}" || true) | head --lines=1)
  if [[ -z ${which} ]]; then
    echo "No ${what} matching ${filter}"
    return 0
  fi

  echo "Setting default ${what} ${which} with volume: ${volume} and muted status: ${muted}"

  pactl set-"${what}"-mute "${which}" "${muted}"
  pactl set-"${what}"-volume "${which}" "${volume}"
  pactl set-default-"${what}" "${which}"
}

function set_sink_or_source() {
  disable_sink_or_source "$@"
  set_default_sink_or_source "$@"
}
function set_audio_help() {
  echo "Cards in:"
  list_cards
  echo ""
  echo "Outputs (sinks):"
  list_sinks
  echo ""
  echo "Inputs (sources):"
  list_sources
  echo ""
  echo "For muting use 0/1, for volume use 0%-100%"
  exit 0
}

function main() {
  set -euo pipefail

  if [[ $# -eq 0 ]]; then
    set_audio_help
  fi

  # Default values
  out_volume=40% && in_volume=100% && out_muted=1 && in_muted=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
    --help) set_audio_help ;;
    --both-filter) out_filter="$2" && in_filter="$2" && shift 2 ;;
    --out-filter) out_filter="$2" && shift 2 ;;
    --in-filter) in_filter="$2" && shift 2 ;;
    --out-volume) out_volume="$2" && shift 2 ;;
    --in-volume) in_volume="$2" && shift 2 ;;
    --out-muted) out_muted="$2" && shift 2 ;;
    --in-muted) in_muted="$2" && shift 2 ;;
    *) echo "Unknown param $1" && exit 1 ;;
    esac
  done

  set_cards "${out_filter}" "${in_filter}"
  # sink is out
  set_sink_or_source sink "${out_filter}" "${out_volume}" "${out_muted}"
  # source is in
  set_sink_or_source source "${in_filter}" "${in_volume}" "${in_muted}"
}

if [[ $# -ne 1 ]] || [[ ${1} != "--source-only" ]]; then
  main "${@}"
fi
