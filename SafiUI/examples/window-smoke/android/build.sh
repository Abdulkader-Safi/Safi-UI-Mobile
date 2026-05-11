#!/usr/bin/env bash
# Build the Android smoke app:
#   1. Compile the Rust crate as `libsafi_ui_window_smoke.so` for arm64-v8a
#      via `cargo ndk` (with `--features device-build` so SDL3 is pulled in).
#   2. Copy SDL3's vendored Android Java glue (SDLActivity et al) out of the
#      cargo cache into the gradle source tree so MainActivity can extend it.
#   3. Optionally wipe the gradle build cache for a clean rebuild.
#   4. Hand off to ./gradlew to assemble the APK.
#
# Args are order-independent. Recognised tokens:
#   debug               → Debug APK (default)
#   release             → Release APK
#   clean               → wipe app/build/ + .gradle/ before building
#
# Examples:
#   ./build.sh                  # debug
#   ./build.sh clean            # wipe + debug
#   ./build.sh release          # release
#   ./build.sh clean release    # wipe + release
#
# Environment:
#   ANDROID_NDK_HOME  — explicit NDK path. If unset, the script picks the
#                       highest-versioned NDK found under $ANDROID_HOME/ndk/
#                       (defaulting to $HOME/Library/Android/sdk on macOS).

set -euo pipefail

# Start from a clean slate so the script's auto-detection isn't shadowed by
# inherited shell env. ANDROID_HOME / ANDROID_NDK_HOME are still picked up
# below if exported by the user — we re-evaluate them inside the script.
unset CMAKE_TOOLCHAIN_FILE ANDROID_ABI ANDROID_PLATFORM ANDROID_NDK ANDROID_NDK_ROOT

mode="debug"
do_clean=0

for arg in "$@"; do
    case "${arg}" in
        debug)         mode="debug" ;;
        release)       mode="release" ;;
        clean)         do_clean=1 ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown arg: ${arg}" >&2
            echo "Usage: $0 [debug|release] [clean]" >&2
            exit 2
            ;;
    esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/../../../" && pwd)"
app_jni_dir="${script_dir}/app/src/main/jniLibs"
app_java_dir="${script_dir}/app/src/main/java"

# --- Resolve ANDROID_HOME (Android SDK root) ---------------------------------
if [[ -z "${ANDROID_HOME:-}" ]]; then
    if [[ -d "${ANDROID_SDK_ROOT:-}" ]]; then
        export ANDROID_HOME="${ANDROID_SDK_ROOT}"
    elif [[ -d "${HOME}/Library/Android/sdk" ]]; then
        export ANDROID_HOME="${HOME}/Library/Android/sdk"
    else
        echo "ERROR: ANDROID_HOME not set and no SDK found at default macOS location." >&2
        echo "Install Android Studio or export ANDROID_HOME=/path/to/android/sdk." >&2
        exit 1
    fi
fi

# Gradle needs sdk.dir in local.properties OR ANDROID_HOME env. Generate
# local.properties if missing so the project also opens correctly in Android Studio.
if [[ ! -f "${script_dir}/local.properties" ]]; then
    echo "sdk.dir=${ANDROID_HOME}" > "${script_dir}/local.properties"
fi

# --- Resolve ANDROID_NDK_HOME -------------------------------------------------
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    sdk_root="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-${HOME}/Library/Android/sdk}}"
    if [[ ! -d "${sdk_root}/ndk" ]]; then
        echo "ERROR: ANDROID_NDK_HOME not set and no NDKs found under ${sdk_root}/ndk." >&2
        echo "Install an NDK via Android Studio (SDK Manager → SDK Tools → NDK)" >&2
        echo "or set ANDROID_NDK_HOME explicitly." >&2
        exit 1
    fi
    # Pick highest-versioned NDK (sort -V is version-aware).
    ndk_version="$(ls "${sdk_root}/ndk" | sort -V | tail -n1)"
    export ANDROID_NDK_HOME="${sdk_root}/ndk/${ndk_version}"
    echo "==> Auto-detected ANDROID_NDK_HOME=${ANDROID_NDK_HOME}"
fi
if [[ ! -d "${ANDROID_NDK_HOME}" ]]; then
    echo "ERROR: ANDROID_NDK_HOME=${ANDROID_NDK_HOME} does not exist." >&2
    exit 1
fi

# --- Clean (optional) ---------------------------------------------------------
if [[ "${do_clean}" -eq 1 ]]; then
    echo "==> Wiping Gradle build cache"
    rm -rf "${script_dir}/app/build" "${script_dir}/.gradle" "${script_dir}/build"
fi

# --- Rust .so ----------------------------------------------------------------
# sdl3-sys's build script invokes CMake to build SDL3 from source. That
# cmake invocation does not inherit cargo-ndk's Android awareness — it
# needs the standard CMake Android env vars + toolchain file pointed at
# the NDK explicitly. Set them here so the nested cmake call resolves the
# Android target correctly.
export ANDROID_NDK="${ANDROID_NDK_HOME}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_HOME}"
# Point at our wrapper which pins ANDROID_ABI=arm64-v8a before delegating
# to the real android.toolchain.cmake. Without this the cmake-rs + sdl3-sys
# nested CMake invocation defaults to armeabi-v7a and fails on NDK r27+.
export CMAKE_TOOLCHAIN_FILE="${script_dir}/cmake/android-arm64.toolchain.cmake"

echo "==> Building Rust .so (arm64-v8a, ${mode})"
profile_flag=""
if [[ "${mode}" = "release" ]]; then profile_flag="--release"; fi

# Force the cdylib to declare a DT_NEEDED dependency on libc++_shared.so so
# Android's dynamic linker resolves SDL3's C++ helpers (`__gxx_personality_v0`,
# `__cxa_*`) at dlopen time. Without -lc++_shared in the link command, rustc
# emits the cdylib with undefined C++ symbols and dlopen fails.
#
# --no-gc-sections keeps SDL3's `Java_org_libsdl_app_SDLActivity_*` JNI
# exports alive. They're called from Java via JNI, not from any Rust code,
# so the default --gc-sections strips them and the JVM throws
# UnsatisfiedLinkError on `nativeGetVersion`.
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-lc++_shared -C link-arg=-Wl,--no-gc-sections -C link-arg=-Wl,--export-dynamic-symbol=Java_org_libsdl_app_*"

( cd "${workspace_dir}" && \
  cargo ndk -t arm64-v8a -o "${app_jni_dir}" \
    build -p safi-ui-window-smoke --features device-build ${profile_flag} )

# --- SDL3 Java glue ----------------------------------------------------------
# SDL3 ships its Android Java glue inside the sdl3-src source crate that
# the build-from-source-static feature pulls in. The crate is unpacked
# into the cargo registry (not into `target/`), so search there.
cargo_registry="${CARGO_HOME:-${HOME}/.cargo}/registry/src"
sdl_src=$(find "${cargo_registry}" -type d -path "*/sdl3-src-*/SDL/android-project/app/src/main/java/org/libsdl/app" 2>/dev/null | head -n1 || true)
if [[ -z "${sdl_src}" ]]; then
    echo "ERROR: could not locate SDL3 Android Java glue. Did cargo build run?" >&2
    exit 1
fi
echo "==> Copying SDL3 Java glue from ${sdl_src}"
mkdir -p "${app_java_dir}/org/libsdl/app"
cp -R "${sdl_src}/." "${app_java_dir}/org/libsdl/app/"

# --- libc++_shared.so --------------------------------------------------------
# SDL3 has C++ helpers (audio routing, etc.) referencing symbols from the
# shared C++ runtime (__gxx_personality_v0, __cxa_*). Cargo's cdylib link
# step doesn't pull these in, so we bundle libc++_shared.so alongside our
# .so in jniLibs so dlopen resolves them at runtime.
libcxx_shared="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
if [[ ! -f "${libcxx_shared}" ]]; then
    echo "ERROR: libc++_shared.so not found at ${libcxx_shared}" >&2
    exit 1
fi
echo "==> Copying libc++_shared.so to jniLibs"
cp "${libcxx_shared}" "${app_jni_dir}/arm64-v8a/libc++_shared.so"

# --- libSDL3.so (shared build) -----------------------------------------------
# sdl3-sys produces libSDL3.so under target/<triple>/<profile>/build/sdl3-sys-*/out/lib/.
# Ship it alongside our .so so that loading it triggers SDL3's JNI_OnLoad,
# which RegisterNatives()-registers SDLActivity's native methods. Without
# this, Java's SDLActivity.nativeGetVersion() throws UnsatisfiedLinkError.
sdl3_so=$(find "${workspace_dir}/target/aarch64-linux-android/${mode}/build" \
    -path "*/sdl3-sys-*/out/lib/libSDL3.so" -type f 2>/dev/null | head -n1 || true)
if [[ -z "${sdl3_so}" ]]; then
    echo "ERROR: libSDL3.so not found under target/. Is sdl3-rs configured with build-from-source (shared, not static)?" >&2
    exit 1
fi
echo "==> Copying libSDL3.so from ${sdl3_so}"
cp "${sdl3_so}" "${app_jni_dir}/arm64-v8a/libSDL3.so"

# --- Gradle assemble ---------------------------------------------------------
cd "${script_dir}"

# Bootstrap the gradle wrapper if missing. Requires `gradle` on PATH
# (install via Android Studio's SDK Manager → "Gradle" or `brew install
# gradle` on macOS).
if [[ ! -f "./gradlew" ]]; then
    if command -v gradle >/dev/null 2>&1; then
        echo "==> Bootstrapping gradle wrapper (one-time)"
        gradle wrapper --gradle-version 8.10.2
    else
        echo "ERROR: ./gradlew is missing and \`gradle\` is not on PATH." >&2
        echo "Install gradle (e.g. \`brew install gradle\`) or open this" >&2
        echo "android/ directory in Android Studio once so it generates" >&2
        echo "the wrapper, then re-run ./build.sh." >&2
        exit 1
    fi
fi

# Use the same gradle-task casing trick portably (avoid bash ${var^} which
# isn't in `set -u` strict mode-friendly when the var is bound at the top).
case "${mode}" in
    debug)   gradle_task="assembleDebug" ;;
    release) gradle_task="assembleRelease" ;;
esac
echo "==> Gradle ${gradle_task}"
./gradlew "${gradle_task}"

echo
echo "APK:"
find app/build/outputs/apk -name '*.apk' -print 2>/dev/null
