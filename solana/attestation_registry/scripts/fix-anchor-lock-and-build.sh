#!/usr/bin/env bash
# Pin 2026-era transitive deps of the Anchor program down to versions the
# Solana 1.18 platform-tools cargo (rust 1.75) can build, then anchor build.
# Iterates: build -> identify failing crate -> pin curated version -> retry.
set -uo pipefail
export PATH="$HOME/.avm/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"
cd "$(dirname "$0")/.."

pin_for() {
  case "$1" in
    blake3) echo 1.6.1 ;;
    zeroize) echo 1.7.0 ;;
    zeroize_derive) echo 1.4.2 ;;
    cpufeatures) echo 0.2.17 ;;
    hashbrown) echo 0.14.5 ;;
    ahash) echo 0.8.11 ;;
    once_cell) echo 1.19.0 ;;
    memchr) echo 2.7.4 ;;
    borsh) echo 1.5.1 ;;
    syn) echo 2.0.72 ;;
    semver) echo 1.0.23 ;;
    rustc_version) echo 0.4.0 ;;
    getrandom) echo 0.2.15 ;;
    rand_core) echo 0.6.4 ;;
    ppv-lite86) echo 0.2.20 ;;
    libc) echo 0.2.155 ;;
    cc) echo 1.1.6 ;;
    constant_time_eq) echo 0.3.0 ;;
    arrayvec) echo 0.7.4 ;;
    bytemuck) echo 1.16.1 ;;
    bytemuck_derive) echo 1.7.0 ;;
    proc-macro-crate) echo 3.1.0 ;;
    toml_edit) echo 0.21.1 ;;
    toml_datetime) echo 0.6.11 ;;
    winnow) echo 0.5.40 ;;
    indexmap) echo 2.2.6 ;;
    equivalent) echo 1.0.1 ;;
    jobserver) echo 0.1.32 ;;
    rayon) echo 1.10.0 ;;
    rayon-core) echo 1.12.1 ;;
    bumpalo) echo 3.16.0 ;;
    serde_json) echo 1.0.120 ;;
    either) echo 1.13.0 ;;
    itertools) echo 0.13.0 ;;
    crossbeam-utils) echo 0.8.20 ;;
    crossbeam-epoch) echo 0.9.18 ;;
    crossbeam-deque) echo 0.8.5 ;;
    autocfg) echo 1.3.0 ;;
    parking_lot) echo 0.12.3 ;;
    parking_lot_core) echo 0.9.10 ;;
    lock_api) echo 0.4.12 ;;
    scopeguard) echo 1.2.0 ;;
    smallvec) echo 1.13.2 ;;
    wasm-bindgen) echo 0.2.92 ;;
    unicode-segmentation) echo 1.11.0 ;;
    unicode-ident) echo 1.0.12 ;;
    unicode-normalization) echo 0.1.23 ;;
    unicode-properties) echo 0.1.2 ;;
    heck) echo 0.4.1 ;;
    convert_case) echo 0.6.0 ;;
    bs58) echo 0.5.1 ;;
    tinyvec) echo 1.8.0 ;;
    tinyvec_macros) echo 0.1.1 ;;
    percent-encoding) echo 2.3.1 ;;
    form_urlencoded) echo 1.2.1 ;;
    url) echo 2.5.2 ;;
    idna) echo 0.5.0 ;;
    log) echo 0.4.22 ;;
    cfg-if) echo 1.0.0 ;;
    itoa) echo 1.0.11 ;;
    ryu) echo 1.0.18 ;;
    paste) echo 1.0.15 ;;
    version_check) echo 0.9.5 ;;
    subtle) echo 2.6.1 ;;
    typenum) echo 1.17.0 ;;
    generic-array) echo 0.14.7 ;;
    opaque-debug) echo 0.3.1 ;;
    keccak) echo 0.1.5 ;;
    sha2) echo 0.10.8 ;;
    sha3) echo 0.10.8 ;;
    hmac) echo 0.12.1 ;;
    pbkdf2) echo 0.12.2 ;;
    curve25519-dalek) echo 4.1.3 ;;
    ed25519-dalek) echo 2.1.1 ;;
    ed25519) echo 2.2.3 ;;
    signature) echo 2.2.0 ;;
    merlin) echo 3.0.0 ;;
    rand) echo 0.8.5 ;;
    rand_chacha) echo 0.3.1 ;;
    rand_hc) echo 0.3.2 ;;
    num-traits) echo 0.2.19 ;;
    num-integer) echo 0.1.46 ;;
    num-iter) echo 0.1.45 ;;
    num-bigint) echo 0.4.6 ;;
    num-prime) echo 0.4.4 ;;
    num-modular) echo 0.6.1 ;;
    serde) echo 1.0.204 ;;
    serde_derive) echo 1.0.204 ;;
    serde_bytes) echo 0.11.15 ;;
    bincode) echo 1.3.3 ;;
    byteorder) echo 1.5.0 ;;
    qstring) echo 0.7.2 ;;
    base64) echo 0.21.7 ;;
    bv) echo 0.11.1 ;;
    feature-probe) echo 0.1.1 ;;
    ark-std) echo 0.4.0 ;;
    ark-ff) echo 0.4.2 ;;
    ark-ec) echo 0.4.2 ;;
    ark-serialize) echo 0.4.2 ;;
    ark-poly) echo 0.4.2 ;;
    digest) echo 0.10.7 ;;
    crypto-common) echo 0.1.6 ;;
    anyhow) echo 1.0.86 ;;
    thiserror) echo 1.0.63 ;;
    thiserror-impl) echo 1.0.63 ;;
    proc-macro2) echo 1.0.86 ;;
    quote) echo 1.0.36 ;;
    unicode-xid) echo 0.2.4 ;;
    synstructure) echo 0.13.1 ;;
    derivative) echo 2.2.0 ;;
    aead) echo 0.5.2 ;;
    aes) echo 0.8.4 ;;
    cipher) echo 0.4.4 ;;
    ctr) echo 0.9.2 ;;
    aes-gcm-siv) echo 0.11.1 ;;
    polyval) echo 0.6.2 ;;
    universal-hash) echo 0.5.1 ;;
    ghash) echo 0.5.1 ;;
    inout) echo 0.1.3 ;;
    block-padding) echo 0.3.3 ;;
    hybrid-array) echo 0.2.0 ;;
    elliptic-curve) echo 0.13.8 ;;
    sec1) echo 0.7.3 ;;
    pkcs8) echo 0.10.2 ;;
    spki) echo 0.7.3 ;;
    der) echo 0.7.9 ;;
    pem-rfc7468) echo 0.7.0 ;;
    base16ct) echo 0.2.0 ;;
    base64ct) echo 1.6.0 ;;
    const-oid) echo 0.9.6 ;;
    crypto-bigint) echo 0.5.5 ;;
    ff) echo 0.13.0 ;;
    group) echo 0.13.0 ;;
    rfc6979) echo 0.4.0 ;;
    ecdsa) echo 0.16.9 ;;
    primeorder) echo 0.13.6 ;;
    lazy_static) echo 1.5.0 ;;
    spl-*-*) echo "" ;;
    *) echo "" ;;
  esac
}

for i in $(seq 1 40); do
  OUT=$(anchor build 2>&1)
  if echo "$OUT" | grep -qE "Finished|To deploy"; then
    echo "BUILD OK"
    ls target/deploy/
    exit 0
  fi
  RAW=$(echo "$OUT" | grep -oE 'failed to download `[a-z0-9_-]+ v[0-9]+\.[0-9]+\.[0-9]+' | head -1)
  if [ -z "$RAW" ]; then
    RAW=$(echo "$OUT" | grep -oE 'package `[a-z0-9_-]+ v[0-9]+\.[0-9]+\.[0-9]+' | head -1 | sed 's/package `/failed to download `/')
  fi
  NAME=$(echo "$RAW" | sed -E 's/failed to download `([a-z0-9_-]+) v.*/\1/')
  VER=$(echo "$RAW" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
  if [ -z "$NAME" ]; then
    echo "$OUT" | tail -20
    echo "NO-CRATE-IDENTIFIED"
    exit 1
  fi
  TARGET=$(pin_for "$NAME")
  if [ -z "$TARGET" ]; then
    echo ">>> unknown culprit: $NAME $VER — no curated pin, stopping"
    exit 2
  fi
  echo ">>> culprit: $NAME $VER -> pin $TARGET"
  RESULT=$(cargo +1.79.0 update -p "$NAME@$VER" --precise "$TARGET" 2>&1 | grep -E "Downgrad|error" | head -3)
  echo "$RESULT"
  if ! echo "$RESULT" | grep -q "Downgrad"; then
    echo "PIN-FAILED for $NAME — stopping"
    exit 4
  fi
done
echo "MAX-ITERATIONS-REACHED"
exit 3
