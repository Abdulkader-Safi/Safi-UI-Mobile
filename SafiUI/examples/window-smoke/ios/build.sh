#!/usr/bin/env bash
# Build the iOS smoke app:
#   1. Compile the Rust crate as `libsafi_ui_window_smoke.a` for either
#      `aarch64-apple-ios` (real device) or `aarch64-apple-ios-sim`
#      (simulator on Apple Silicon).
#   2. Stage the staticlib into Sources/build/<platform>/ so the Xcode
#      project's `$(PLATFORM_NAME)` search path picks up the right one.
#   3. Copy SDL3's vendored headers next to the static libs so the
#      bridging header can `#import <SDL3/...>`.
#   4. Optionally wipe Xcode's DerivedData for this project so build
#      setting changes take effect without a stale-cache stall.
#   5. Hand off to xcodebuild to assemble the .app.
#
# Args are order-independent. Recognised tokens:
#   sim | simulator     → target the iOS Simulator (default)
#   device | ios        → target a real iPhone
#   debug               → Debug configuration (default)
#   release             → Release configuration
#   clean               → wipe ~/Library/Developer/Xcode/DerivedData
#                         for this project before building
#
# Examples:
#   ./build.sh                          # sim, debug
#   ./build.sh clean                    # wipe DerivedData + sim debug
#   ./build.sh device release           # device, release
#   ./build.sh clean device release     # wipe + device release

set -euo pipefail

dest="sim"
mode="debug"
do_clean=0

for arg in "$@"; do
    case "${arg}" in
        sim|simulator) dest="sim" ;;
        device|ios)    dest="device" ;;
        debug)         mode="debug" ;;
        release)       mode="release" ;;
        clean)         do_clean=1 ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown arg: ${arg}" >&2
            echo "Usage: $0 [sim|device] [debug|release] [clean]" >&2
            exit 2
            ;;
    esac
done

case "${dest}" in
    sim)
        rust_target="aarch64-apple-ios-sim"
        platform_dir="iphonesimulator"
        xcode_sdk="iphonesimulator"
        xcode_dest="generic/platform=iOS Simulator"
        ;;
    device)
        rust_target="aarch64-apple-ios"
        platform_dir="iphoneos"
        xcode_sdk="iphoneos"
        xcode_dest="generic/platform=iOS"
        ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/../../../" && pwd)"
build_dir="${script_dir}/Sources/build/${platform_dir}"

# Derive the project name from the .xcodeproj sitting next to this script
# so the DerivedData glob stays correct even if someone renames it.
xcodeproj="$(find "${script_dir}" -maxdepth 1 -name '*.xcodeproj' -print -quit)"
if [[ -z "${xcodeproj}" ]]; then
    echo "ERROR: no .xcodeproj found in ${script_dir}" >&2
    exit 1
fi
project_name="$(basename "${xcodeproj}" .xcodeproj)"

profile_flag=""
target_dir="debug"
config="Debug"
if [[ "${mode}" = "release" ]]; then
    profile_flag="--release"
    target_dir="release"
    config="Release"
fi

if [[ "${do_clean}" -eq 1 ]]; then
    derived_data_glob="${HOME}/Library/Developer/Xcode/DerivedData/${project_name}-*"
    echo "==> Wiping Xcode DerivedData (${derived_data_glob})"
    # shellcheck disable=SC2086
    rm -rf ${derived_data_glob}
fi

echo "==> Building Rust staticlib (${rust_target}, ${mode})"
# Match the Xcode deployment target. Without this cargo defaults to an
# older minimum that misses Darwin stack-probe intrinsics SDL3 references,
# breaking the cdylib link step.
export IPHONEOS_DEPLOYMENT_TARGET=16.0
( cd "${workspace_dir}" && \
  cargo build -p safi-ui-window-smoke --features device-build \
    --target "${rust_target}" ${profile_flag} )

mkdir -p "${build_dir}"
cp "${workspace_dir}/target/${rust_target}/${target_dir}/libsafi_ui_window_smoke.a" \
   "${build_dir}/libsafi_ui_window_smoke.a"

# SDL3 vendored headers — copy out of the sdl3-sys build dir so #import works.
sdl_headers=$(find "${workspace_dir}/target/${rust_target}" -type d -name "SDL3" -path "*/include/*" 2>/dev/null | head -n1 || true)
if [[ -z "${sdl_headers}" ]]; then
    echo "ERROR: could not locate SDL3 headers under target/${rust_target}. Did cargo build run?" >&2
    exit 1
fi
echo "==> Copying SDL3 headers from ${sdl_headers}"
rm -rf "${script_dir}/build/sdl3-include"
mkdir -p "${script_dir}/build/sdl3-include/SDL3"
cp -R "${sdl_headers}/." "${script_dir}/build/sdl3-include/SDL3/"

# --- libSDL3.dylib (shared build) ---
# Mirrors what android/build.sh does for libSDL3.so. Without this, Xcode
# can't resolve _SDL_* symbols at link time and at runtime the app can't
# dlopen SDL3 from the .app bundle.
sdl3_dylib=$(find "${workspace_dir}/target/${rust_target}/${target_dir}/build" \
    -path "*/sdl3-sys-*/out/lib/libSDL3*.dylib" -type f 2>/dev/null \
    | head -n1 || true)
if [[ -z "${sdl3_dylib}" ]]; then
    echo "ERROR: libSDL3.dylib not found under target/${rust_target}/${target_dir}/build." >&2
    echo "Is sdl3-rs configured with build-from-source (shared, not static)?" >&2
    exit 1
fi
echo "==> Copying libSDL3.dylib from ${sdl3_dylib}"
cp "${sdl3_dylib}" "${build_dir}/libSDL3.dylib"

# Rewrite the dylib's install-name to @rpath so dyld resolves it from the
# .app's Frameworks/ at runtime. Idempotent.
install_name_tool -id "@rpath/libSDL3.dylib" "${build_dir}/libSDL3.dylib"

echo "==> xcodebuild (${config}, ${xcode_sdk})"
cd "${script_dir}"
xcodebuild \
    -project "${project_name}.xcodeproj" \
    -scheme "${project_name}" \
    -configuration "${config}" \
    -sdk "${xcode_sdk}" \
    -destination "${xcode_dest}" \
    build CODE_SIGN_IDENTITY="" CODE_SIGNING_REQUIRED=NO CODE_SIGNING_ALLOWED=NO
