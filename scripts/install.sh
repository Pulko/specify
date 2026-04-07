#!/usr/bin/env bash
# Installs the latest (or pinned) prebuilt specify binary from GitHub Releases.
# Usage: curl -fsSL https://raw.githubusercontent.com/Pulko/specify/main/scripts/install.sh | bash
#
# Environment:
#   SPECIFY_REPO         GitHub repo root URL (default: https://github.com/Pulko/specify)
#   SPECIFY_VERSION      Pin version: "0.1.1" or "v0.1.1" (default: latest release)
#   SPECIFY_INSTALL_DIR  Directory for the binary (default: ~/.cargo/bin, else ~/.local/bin)

set -uo pipefail

DEFAULT_REPO="https://github.com/Pulko/specify"
REPO="${SPECIFY_REPO:-$DEFAULT_REPO}"
REPO="${REPO%/}"

die() {
  echo "specify install: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

sha256_of_file() {
  local f="$1"
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$f" | awk '{print $2}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$f" | awk '{print $1}'
  else
    die "need openssl, sha256sum, or shasum to verify checksums"
  fi
}

expected_sha256_from_file() {
  awk 'NF {print $1; exit}' "$1"
}

detect_triple() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64) TRIPLE="x86_64-unknown-linux-gnu" ;;
        aarch64 | arm64)
          die "no prebuilt binary for Linux ARM. Try: cargo install --git ${REPO}.git"
          ;;
        *) die "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        arm64) TRIPLE="aarch64-apple-darwin" ;;
        x86_64) TRIPLE="x86_64-apple-darwin" ;;
        *) die "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    *) die "unsupported OS: $os (this script supports Linux and macOS)" ;;
  esac
}

resolve_install_dir() {
  if [[ -n "${SPECIFY_INSTALL_DIR:-}" ]]; then
    echo "$SPECIFY_INSTALL_DIR"
    return
  fi
  if [[ -n "${CARGO_HOME:-}" ]] && [[ -d "${CARGO_HOME}/bin" ]]; then
    echo "${CARGO_HOME}/bin"
    return
  fi
  if [[ -d "${HOME}/.cargo/bin" ]]; then
    echo "${HOME}/.cargo/bin"
    return
  fi
  echo "${HOME}/.local/bin"
}

resolve_tag_and_ver() {
  if [[ -n "${SPECIFY_VERSION:-}" ]]; then
    local v="${SPECIFY_VERSION}"
    if [[ "$v" == v* ]]; then
      TAG="$v"
      VER="${v#v}"
    else
      TAG="v${v}"
      VER="$v"
    fi
    return
  fi
  local json
  json="$(curl -fsSL -H "Accept: application/vnd.github+json" "${REPO}/releases/latest")" || die "could not fetch latest release from ${REPO}"
  TAG="$(printf '%s' "$json" | sed -n 's/.*"tag_name":"\([^"]*\)".*/\1/p' | head -n1)"
  [[ -n "$TAG" ]] || die "could not parse tag_name from GitHub response"
  VER="${TAG#v}"
}

need_cmd curl
need_cmd tar

TRIPLE=""
detect_triple
resolve_tag_and_ver

STEM="specify-v${VER}-${TRIPLE}"
ASSET="${STEM}.tar.gz"
URL="${REPO}/releases/download/${TAG}/${ASSET}"

if command -v specify >/dev/null 2>&1; then
  current="$(specify -V 2>/dev/null | awk '{print $2}')"
  if [[ -n "$current" && "$current" == "$VER" ]]; then
    echo "specify is already at ${TAG}."
    exit 0
  fi
  if [[ -n "$current" ]]; then
    echo "Updating specify from ${current} to ${TAG} ..."
  fi
fi

INSTALL_DIR="$(resolve_install_dir)"
mkdir -p "$INSTALL_DIR" || die "could not create install directory: ${INSTALL_DIR}"

TMP=""
cleanup() {
  [[ -n "$TMP" ]] && rm -rf "$TMP"
}
trap cleanup EXIT
TMP="$(mktemp -d)" || die "could not create temp directory"

curl -fsSL -o "${TMP}/${ASSET}" "$URL" || die "download failed: ${URL}"
curl -fsSL -o "${TMP}/${ASSET}.sha256" "${URL}.sha256" || die "checksum download failed: ${URL}.sha256"

expected="$(expected_sha256_from_file "${TMP}/${ASSET}.sha256")"
actual="$(sha256_of_file "${TMP}/${ASSET}")"
[[ -n "$expected" && -n "$actual" ]] || die "could not read SHA256 checksum"
[[ "${expected}" == "${actual}" ]] || die "SHA256 mismatch (file may be corrupted)"

tar -xzf "${TMP}/${ASSET}" -C "$TMP" || die "could not extract archive"
BIN_SRC="${TMP}/${STEM}/specify"
[[ -f "$BIN_SRC" ]] || die "expected binary not found in archive: ${STEM}/specify"

mv -f "$BIN_SRC" "${INSTALL_DIR}/specify" || die "could not install binary to ${INSTALL_DIR}"
chmod +x "${INSTALL_DIR}/specify" || true

echo "Installed specify ${TAG} to ${INSTALL_DIR}/specify"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Add ${INSTALL_DIR} to PATH, for example:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
