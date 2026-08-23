#!/usr/bin/env sh
################################################################
# Copyright (c) 2026 Witalis Domitrz <witekdomitrz@gmail.com>
# AGPL License
################################################################

set -eu

image=rscript-rust
cache_dir="${XDG_CACHE_HOME:-${HOME}/.cache}/rscript"
stage=".rscript-incoming.$$" # unique per invocation

# Runs inside the container; PKG, DEPS_TOML and STAGE via environment.
# shellcheck disable=SC2016  # expanded inside the container, not here
build_script='
set -eu

step() {
  printf "==> %s\n" "$1" >&2
}

mkdir -p src
cat > src/main.rs

cat > Cargo.toml <<TOML
[package]
name = "$PKG"
version = "0.0.0"
edition = "2021"

[profile.release]
strip = true
$DEPS_TOML
TOML

step "format: cargo fmt --check"
cargo fmt --check

step "lint: cargo clippy (--all-targets, pedantic denied)"
cargo clippy --release --all-targets -- -D warnings -W clippy::pedantic

step "test: cargo test"
cargo test --release

step "docs: cargo doc (warnings denied)"
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

step "build: cargo build --release"
cargo build --release

cp "target/release/$PKG" "/cache/${STAGE}"
'

verbose=0
force=0
check=0

self=""
prog=""

# Prints the absolute path of ${1}.
abspath() {
  [ "${#}" -eq 1 ] || exit 1
  if readlink -f . > /dev/null 2>&1; then
    readlink -f -- "${1}"
    return
  fi
  _dir=$(dirname -- "${1}")
  _file=$(basename -- "${1}")
  printf '%s/%s\n' "$(cd -- "${_dir}" && pwd -P)" "${_file}"
}

# md5 of stdin.
digest_stdin() {
  if command -v md5sum > /dev/null 2>&1; then
    md5sum
  else
    md5
  fi
}

usage() {
  # env form only works when the script is reachable via PATH.
  shebang="#!/usr/bin/env ${prog}"
  command -v "${prog}" > /dev/null 2>&1 || shebang="#!${self}"
  cat << EOF
usage: ${prog} [options] <script.rs> [args...]

Compile and run a Rust source file in a docker rust container.
fmt, clippy, tests and docs are checked on every rebuild; the binary
is cached in ${cache_dir} until the source changes.

Dependencies via meta comments:
  //# dependencies:
  //# clap = { version = "4", features = ["derive"] }

Options:
  -v, --verbose       show build output
  -f, --force-rebuild force rebuild
  -c, --check         check only, do not run
  --clean-cache       remove cached binaries
  -h, --help          show this help

As a shebang, first line of an executable .rs file:
  ${shebang}
EOF
}

update_image() {
  img_log=$({
    docker pull -q rust:1
    docker build -q -t "${image}" - << 'EOF'
FROM rust:1
RUN rustup component add clippy rustfmt \
      && mkdir -p /work && chmod 1777 /work
EOF
  } 2>&1) && return 0
  # Refresh failed (e.g. offline); fall back to the local image.
  docker image inspect "${image}" > /dev/null 2>&1 || {
    printf '%s\n' "${img_log}" >&2
    return 1
  }
}

needs_build() {
  [ "${#}" -eq 3 ] || exit 1
  { [ "${1}" -eq 1 ] || [ ! -x "${2}" ]; } && return 0
  # POSIX alternative to test's non-portable -nt.
  [ -n "$(find "${3}" -newer "${2}")" ]
}

parse_deps() {
  [ "${#}" -eq 1 ] || exit 1
  awk '
    /^\/\/#/ {
      line = $0
      sub(/^\/\/#([[:space:]]|$)/, "", line)
      if (!header_seen) {
        header_seen = 1
        if (line !~ /^dependencies:$/) exit 1
        next
      }
      print line
    }
  ' "${1}"
}

derive_pkg_name() {
  [ "${#}" -eq 1 ] || exit 1
  pkg=$(printf '%s' "${1%.rs}" | tr -c 'a-zA-Z0-9_-' '_')
  # Prefix names invalid for cargo's target-directory layout.
  case ${pkg} in
    '' | [0-9]* | build | deps | examples | incremental | bins | native)
      pkg="_${pkg}"
      ;;
    *) ;;
  esac
  printf '%s\n' "${pkg}"
}

extract_deps() {
  [ "${#}" -eq 1 ] || exit 1
  deps=""
  # shellcheck disable=SC2310
  deps=$(parse_deps "${1}") || true
  if [ -n "${deps}" ]; then
    printf '\n[dependencies]\n%s\n' "${deps}"
  fi
}

run_container() {
  [ "${#}" -eq 4 ] || exit 1
  docker run --rm -i \
    -u "$(id -u):$(id -g)" \
    -e PKG="${1}" \
    -e DEPS_TOML="${2}" \
    -e STAGE="${stage}" \
    -v "${3}":/cache \
    -w /work \
    "${image}" \
    sh -c "${build_script}" < "${4}"
}

build() {
  [ "${#}" -eq 3 ] || exit 1
  build_verbose=${1}
  script_path=${2}
  script_cache_dir=${3}
  script_name=${script_path##*/}

  # shellcheck disable=SC2310
  if ! update_image; then
    echo "rscript: image update failed" >&2
    exit 1
  fi

  pkg=$(derive_pkg_name "${script_name}")
  deps_toml=$(extract_deps "${script_path}")

  mkdir -p "${script_cache_dir}"

  rc=0
  build_log=""
  # shellcheck disable=SC2310
  if [ "${build_verbose}" -eq 0 ]; then
    build_log=$(run_container \
      "${pkg}" "${deps_toml}" \
      "${script_cache_dir}" "${script_path}" 2>&1) || rc=${?}
  else
    # shellcheck disable=SC2310
    run_container \
      "${pkg}" "${deps_toml}" \
      "${script_cache_dir}" "${script_path}" >&2 || rc=${?}
  fi

  if [ "${rc}" -ne 0 ]; then
    [ "${build_verbose}" -eq 0 ] && printf '%s\n' "${build_log}" >&2
    echo "rscript: ${script_name}: build failed" >&2
    exit "${rc}"
  fi

  # Same directory as the staged file: atomic rename(2). With
  # concurrent builds of the same script, both renames succeed and the
  # last one wins; each staged file was written independently.
  bin="${script_cache_dir}/${script_name}"
  mv -f "${script_cache_dir}/${stage}" "${bin}"
  printf '%s\n' "${bin}"
}

main() {
  case ${0} in
    */*) self=${0} ;;
    *) self=$(command -v "${0}") ;;
  esac
  self=$(abspath "${self}")
  prog=${self##*/}

  while [ "${#}" -gt 0 ]; do
    case ${1} in
      -h | --help)
        usage
        exit 0
        ;;
      -v | --verbose)
        verbose=1
        ;;
      -f | --force-rebuild)
        force=1
        ;;
      -c | --check)
        check=1
        ;;
      --clean-cache)
        mkdir -p "${cache_dir}"
        find "${cache_dir}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
        exit 0
        ;;
      --)
        shift
        break
        ;;
      -*)
        echo "rscript: unknown option: ${1}" >&2
        exit 2
        ;;
      *)
        break
        ;;
    esac
    shift
  done
  [ "${#}" -gt 0 ] || {
    echo "rscript: missing script" >&2
    exit 2
  }

  script=$(abspath "${1}")
  shift
  name=${script##*/}
  hash=$(printf '%s' "${script}" | digest_stdin)
  hash=${hash%% *}
  dir="${cache_dir}/${hash}"
  bin="${dir}/${name}"

  # shellcheck disable=SC2310
  if needs_build "${force}" "${bin}" "${script}"; then
    bin=$(build "${verbose}" "${script}" "${dir}")
  else
    [ -f "${script}" ] || {
      rm -f -- "${bin}"
      rmdir "${dir}" 2> /dev/null || true
      echo "rscript: ${name}: no such file (stale binary removed)" >&2
      exit 1
    }
    [ "${check}" -eq 1 ] && {
      echo "rscript: ${name}: checks passed (cached)" >&2
      exit 0
    }
  fi

  [ "${check}" -eq 1 ] && {
    echo "rscript: ${name}: checks passed" >&2
    exit 0
  }

  exec "${bin}" "${@}"
}

main "${@}"
