#!/usr/bin/env bash
# Build on Linux x86_64 with the toolchain pinned by android.yml.
# The APK is signed with a NEW TEMPORARY TEST KEY on every invocation.
# It is installable, but not a production release and cannot update a build
# signed by another invocation. Never upload or reuse the temporary keystore.
set -euo pipefail

project_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd -- "$project_root"

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
    printf '%s\n' 'Official Android Linux SDK/NDK tools require an x86_64 host.' >&2
    exit 1
fi
for tool in cargo rustc python3 keytool java realpath; do
    command -v "$tool" >/dev/null || { printf 'Missing tool: %s\n' "$tool" >&2; exit 1; }
done
: "${ANDROID_HOME:?Set ANDROID_HOME to an SDK containing platform 34 and build-tools 35.0.0}"
ANDROID_HOME="$(realpath -- "$ANDROID_HOME")"
export ANDROID_HOME ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/28.2.13676358"
# cargo-apk reads ANDROID_NDK_ROOT; rust-skia reads ANDROID_NDK.
export ANDROID_NDK="$ANDROID_NDK_ROOT"
export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
build_tools="$ANDROID_HOME/build-tools/35.0.0"
for tool in aapt zipalign apksigner; do
    [[ -x "$build_tools/$tool" ]] || { printf 'Missing Android tool: %s\n' "$build_tools/$tool" >&2; exit 1; }
done
[[ -f "$ANDROID_HOME/platforms/android-34/android.jar" ]] || { printf '%s\n' 'Missing SDK platform android-34.' >&2; exit 1; }
[[ -x "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/clang" ]] || { printf '%s\n' 'Missing NDK r28c Linux x86_64 toolchain.' >&2; exit 1; }
[[ "$(rustc --version)" == 'rustc 1.90.0 '* ]] || { printf '%s\n' 'Rust 1.90.0 is required.' >&2; exit 1; }

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/relay-price-calc-m.XXXXXXXX")"
work_dir="$(realpath -- "$work_dir")"
: > "$work_dir/.relay-price-calc-m-build"
cleanup() {
    # Only remove the canonical directory created and marked by this invocation.
    if [[ "$work_dir" == /*/relay-price-calc-m.* && -f "$work_dir/.relay-price-calc-m-build" ]]; then
        rm -rf -- "$work_dir"
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

export CARGO_TARGET_DIR="$work_dir/target"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS='-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384'
export CARGO_APK_RELEASE_KEYSTORE="$work_dir/ci-test.p12"
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android
printf '%s\n' 'WARNING: generating a disposable TEST signing key; this is NOT a production signing identity.'
keytool -genkeypair -noprompt -storetype PKCS12 \
    -keystore "$CARGO_APK_RELEASE_KEYSTORE" -alias androiddebugkey \
    -storepass android -keypass android -keyalg RSA -keysize 2048 \
    -validity 3650 -dname 'CN=relay-price-calc-m CI TEST ONLY,O=relay-price-calc-m,C=ZZ'

# cargo-apk release requires a signing key even though its first signed APK is
# not distributed. We re-align its unsigned input and then sign the aligned APK.
# cargo-apk 0.10.0 does NOT accept --locked (exit code 2). Prepare the
# locked dependency graph with Cargo, then ensure packaging did not change it.
cargo fetch --locked --target aarch64-linux-android
lock_sha256="$(sha256sum Cargo.lock)"
cargo apk build --lib --release --target aarch64-linux-android --features ui
[[ "$(sha256sum Cargo.lock)" == "$lock_sha256" ]] || {
    printf '%s\n' 'Cargo.lock changed during packaging; refusing to publish APK.' >&2
    exit 1
}
mapfile -d '' -t unaligned_apks < <(python3 - "$CARGO_TARGET_DIR" <<'PY'
import pathlib
import sys
for path in sorted(pathlib.Path(sys.argv[1]).rglob("*-unaligned.apk")):
    sys.stdout.buffer.write(str(path).encode() + b"\0")
PY
)
if (( ${#unaligned_apks[@]} != 1 )); then
    printf 'Expected one unsigned *-unaligned.apk, found %s.\n' "${#unaligned_apks[@]}" >&2
    exit 1
fi

"$build_tools/zipalign" -P 16 -f -v 4 "${unaligned_apks[0]}" "$work_dir/aligned.apk"
"$build_tools/apksigner" sign \
    --ks "$CARGO_APK_RELEASE_KEYSTORE" --ks-key-alias androiddebugkey \
    --ks-pass env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD \
    --key-pass env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD \
    --v4-signing-enabled false --out "$work_dir/signed.apk" "$work_dir/aligned.apk"
"$build_tools/apksigner" verify --verbose --print-certs "$work_dir/signed.apk"
"$build_tools/zipalign" -c -P 16 -v 4 "$work_dir/signed.apk"
python3 scripts/verify-apk.py "$work_dir/signed.apk" --aapt "$build_tools/aapt"

# Include source license and bundled-font/dependency notices, never a keystore.
shopt -s nullglob
legal_sources=(LICENSE* NOTICE* THIRD*)
[[ ! -d licenses ]] || legal_sources+=(licenses)
if (( ${#legal_sources[@]} == 0 )); then
    printf '%s\n' 'Missing license/notice materials; refusing to produce an incomplete artifact.' >&2
    exit 1
fi
mkdir -p dist/legal
for source in "${legal_sources[@]}"; do
    cp -R -- "$source" dist/legal/
done
cp -- "$work_dir/signed.apk" dist/relay-price-calc-m-arm64-test.apk
{
    printf '%s\n' 'relay-price-calc-m — disposable TEST-signed APK'
    printf '%s\n' 'NOT a production signing key. Each build gets a different certificate.'
    printf '%s\n' 'To install over a build from another run, uninstall that old app first (this clears its local data).'
    printf '%s\n' 'arm64-v8a; minimum API 26; target API 34; sideloading baseline, not a Play compliance claim.'
    printf '%s\n' 'Slint 1.14.1 Android NativeActivity backend uses Skia, not renderer-software.'
    printf '%s\n' 'ELF/ZIP 16KB checks passed; real 16KB-device, IME, Chinese-font and inset testing is still required.'
    rustc --version
    cargo apk version
    printf 'Source commit: %s\n' "${GITHUB_SHA:-not-recorded}"
} > dist/BUILD-INFO.txt
( cd dist && sha256sum relay-price-calc-m-arm64-test.apk > SHA256SUMS )
printf '%s\n' 'Created dist/relay-price-calc-m-arm64-test.apk and SHA256SUMS (TEST key only).'
