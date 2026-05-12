#!/usr/bin/env bash
# Flutter-style one-shot Android run: pick a device (or boot an emulator),
# build the example, install, launch, stream logs.
#
# Usage:
#   ./scripts/dev-android.sh [example] [flags]
#
# Positional:
#   example          Name under examples/ (default: window-smoke)
#
# Flags:
#   --release        Build release APK (default: debug)
#   --clean          Wipe gradle build cache before building
#   --avd <name>     Boot a specific AVD if no device is connected
#   --device <id>    Use this connected device (`adb devices` shows it)
#   --no-launch      Build + install only; skip launch and logcat
#   --no-logs        Build + install + launch; skip logcat tail
#
# Eventual replacement: PRD §20 `safi dev --target android` (todo 33).

set -euo pipefail

# ---------- arg parsing ----------
EXAMPLE="window-smoke"
MODE="debug"
DO_CLEAN=0
AVD=""
DEVICE=""
NO_LAUNCH=0
NO_LOGS=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)    MODE="release"; shift ;;
        --clean)      DO_CLEAN=1; shift ;;
        --avd)        AVD="$2"; shift 2 ;;
        --device)     DEVICE="$2"; shift 2 ;;
        --no-launch)  NO_LAUNCH=1; shift ;;
        --no-logs)    NO_LOGS=1; shift ;;
        -h|--help)    grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        --*)          echo "Unknown flag: $1" >&2; exit 2 ;;
        *)            EXAMPLE="$1"; shift ;;
    esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"
example_dir="${workspace_dir}/examples/${EXAMPLE}"
android_dir="${example_dir}/android"

if [[ ! -d "${android_dir}" ]]; then
    echo "ERROR: examples/${EXAMPLE}/android not found." >&2
    exit 1
fi

# ---------- resolve ANDROID_HOME ----------
if [[ -z "${ANDROID_HOME:-}" ]]; then
    if [[ -d "${ANDROID_SDK_ROOT:-}" ]]; then
        export ANDROID_HOME="${ANDROID_SDK_ROOT}"
    elif [[ -d "${HOME}/Library/Android/sdk" ]]; then
        export ANDROID_HOME="${HOME}/Library/Android/sdk"
    elif [[ -d "${HOME}/Android/Sdk" ]]; then
        export ANDROID_HOME="${HOME}/Android/Sdk"
    else
        echo "ERROR: ANDROID_HOME not set and no SDK at default location." >&2
        echo "Install Android Studio or export ANDROID_HOME=/path/to/android/sdk." >&2
        exit 1
    fi
fi

ADB="${ANDROID_HOME}/platform-tools/adb"
EMU="${ANDROID_HOME}/emulator/emulator"

if [[ ! -x "${ADB}" ]]; then
    echo "ERROR: adb not found at ${ADB}." >&2
    exit 1
fi

# ---------- pick a device ----------
attached_device=""
list_devices() {
    "${ADB}" devices | awk '$2 == "device" {print $1}'
}

if [[ -n "${DEVICE}" ]]; then
    if "${ADB}" -s "${DEVICE}" get-state &>/dev/null; then
        attached_device="${DEVICE}"
    else
        echo "ERROR: device '${DEVICE}' not in 'device' state." >&2
        "${ADB}" devices
        exit 1
    fi
else
    attached_device="$(list_devices | head -n1 || true)"
fi

if [[ -z "${attached_device}" ]]; then
    if [[ ! -x "${EMU}" ]]; then
        echo "ERROR: no connected device and no emulator binary at ${EMU}." >&2
        exit 1
    fi
    if [[ -z "${AVD}" ]]; then
        AVD="$("${EMU}" -list-avds 2>/dev/null | head -n1 || true)"
    fi
    if [[ -z "${AVD}" ]]; then
        echo "ERROR: no AVDs found. Create one via Android Studio →" >&2
        echo "Device Manager, or:  avdmanager create avd -n Pixel_8_API_35 ..." >&2
        exit 1
    fi
    echo "==> Booting emulator: ${AVD}"
    "${EMU}" -avd "${AVD}" -no-snapshot -no-audio -no-boot-anim >/dev/null 2>&1 &
    "${ADB}" wait-for-device
    echo "==> Waiting for boot to complete…"
    until [[ "$("${ADB}" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; do
        sleep 1
    done
    attached_device="$(list_devices | head -n1)"
    echo "==> Emulator ready: ${attached_device}"
fi

export ANDROID_SERIAL="${attached_device}"

# ---------- build ----------
build_args=()
[[ "${MODE}" = "release" ]] && build_args+=("release")
[[ "${DO_CLEAN}" -eq 1 ]] && build_args+=("clean")

echo "==> Building ${EXAMPLE} (${MODE}) via examples/${EXAMPLE}/android/build.sh"
( cd "${android_dir}" && ./build.sh ${build_args[@]+"${build_args[@]}"} )

apk_dir="${android_dir}/app/build/outputs/apk/${MODE}"
apk="${apk_dir}/app-${MODE}.apk"
if [[ ! -f "${apk}" ]]; then
    apk="$(find "${apk_dir}" -name '*.apk' -print -quit 2>/dev/null || true)"
fi
if [[ -z "${apk}" || ! -f "${apk}" ]]; then
    echo "ERROR: APK not found under ${apk_dir}" >&2
    exit 1
fi

# ---------- package id via aapt ----------
aapt="$(find "${ANDROID_HOME}/build-tools" -name aapt -type f 2>/dev/null | sort -V | tail -n1 || true)"
if [[ -z "${aapt}" ]]; then
    echo "ERROR: aapt not found under ${ANDROID_HOME}/build-tools." >&2
    exit 1
fi
package_id="$("${aapt}" dump badging "${apk}" 2>/dev/null \
    | sed -n "s/^package: name='\\([^']*\\)'.*/\\1/p" | head -n1)"
if [[ -z "${package_id}" ]]; then
    echo "ERROR: could not read package id from APK." >&2
    exit 1
fi

trap 'echo; echo "APK:     ${apk}"; echo "Package: ${package_id}"' EXIT

echo "==> Installing ${apk}"
"${ADB}" install -r "${apk}"

if [[ "${NO_LAUNCH}" -eq 1 ]]; then
    echo "==> Installed. Skipping launch (--no-launch)."
    exit 0
fi

echo "==> Launching ${package_id}"
"${ADB}" shell monkey -p "${package_id}" -c android.intent.category.LAUNCHER 1 >/dev/null

if [[ "${NO_LOGS}" -eq 1 ]]; then
    exit 0
fi

echo "==> Streaming logs (Ctrl-C to exit)…"
"${ADB}" logcat -c
"${ADB}" logcat -s "safi-ui-window-smoke" "SDL"
