#!/usr/bin/env bash
# Flutter-style one-shot iOS Simulator run: boot a simulator, build the
# example for `aarch64-apple-ios-sim`, install, launch with stdout/stderr
# piped to this terminal.
#
# Usage:
#   ./scripts/dev-ios.sh [example] [flags]
#
# Positional:
#   example                Name under examples/ (default: window-smoke)
#
# Flags:
#   --release              Build release config (default: debug)
#   --clean                Wipe Xcode DerivedData before building
#   --simulator <name>     Boot this simulator (default: iPhone 15 Pro)
#   --no-launch            Build + install only; skip launch
#   --no-logs              Build + install + launch; no log fallback
#
# ⚠️ SDL_GPU Metal does NOT initialise on the iOS Simulator (see
# examples/window-smoke/TESTING.md). The app boots, Swift main runs,
# lifecycle logs stream — but the render pass errors. Use a real iPhone
# via Xcode for visual checks.
#
# Eventual replacement: PRD §20 `safi dev --target ios` (todo 33).

set -euo pipefail

EXAMPLE="window-smoke"
MODE="debug"
DO_CLEAN=0
SIM_NAME=""        # auto-detect if unset (see resolve_default_sim)
NO_LAUNCH=0
NO_LOGS=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)     MODE="release"; shift ;;
        --clean)       DO_CLEAN=1; shift ;;
        --simulator)   SIM_NAME="$2"; shift 2 ;;
        --no-launch)   NO_LAUNCH=1; shift ;;
        --no-logs)     NO_LOGS=1; shift ;;
        -h|--help)     grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        --*)           echo "Unknown flag: $1" >&2; exit 2 ;;
        *)             EXAMPLE="$1"; shift ;;
    esac
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"
example_dir="${workspace_dir}/examples/${EXAMPLE}"
ios_dir="${example_dir}/ios"

if [[ ! -d "${ios_dir}" ]]; then
    echo "ERROR: examples/${EXAMPLE}/ios not found." >&2
    exit 1
fi

xcodeproj="$(find "${ios_dir}" -maxdepth 1 -name '*.xcodeproj' -print -quit)"
if [[ -z "${xcodeproj}" ]]; then
    echo "ERROR: no .xcodeproj under ${ios_dir}" >&2
    exit 1
fi
project_name="$(basename "${xcodeproj}" .xcodeproj)"

if ! command -v xcrun >/dev/null 2>&1; then
    echo "ERROR: xcrun not on PATH (install Xcode + command-line tools)." >&2
    exit 1
fi

cat <<'WARN' >&2
⚠️  iOS Simulator SDL_GPU caveat: the virtual Metal device fails SDL_GPU
   init (MTLGPUFamilyApple3). The app boots and lifecycle logs stream,
   but render passes will error. For full visuals use a real iPhone via
   Xcode. See examples/window-smoke/TESTING.md.

WARN

# ---------- resolve simulator UDID ----------
# If --simulator was passed, look it up by name. Otherwise auto-pick a sane
# default: prefer newest "iPhone … Pro Max" → "Pro" → any iPhone, scanning
# the highest-versioned iOS runtime first.
pick_first_match() {
    local pattern="$1"
    xcrun simctl list devices available \
        | awk '
            /^-- iOS [0-9.]+/ { runtime=$0; next }
            /^-- / { runtime=""; next }
            runtime != "" && $0 ~ pat {
                line=$0
                if (match(line, /\(([0-9A-Fa-f-]{36})\)/)) {
                    udid=substr(line, RSTART+1, RLENGTH-2)
                    name=line
                    sub(/^[[:space:]]+/, "", name)
                    sub(/[[:space:]]*\(.*$/, "", name)
                    print runtime "\t" name "\t" udid
                }
            }
        ' pat="${pattern}" \
        | sort -r \
        | head -n1
}

if [[ -n "${SIM_NAME}" ]]; then
    UDID="$(xcrun simctl list devices "${SIM_NAME}" available 2>/dev/null \
        | grep -Eo '[0-9A-Fa-f-]{36}' | head -n1 || true)"
    if [[ -z "${UDID}" ]]; then
        echo "ERROR: simulator '${SIM_NAME}' not found." >&2
        echo "Available:" >&2
        xcrun simctl list devices available | sed -n '/-- iOS/,/^-- /p' >&2
        exit 1
    fi
else
    for pattern in 'iPhone.*Pro Max' 'iPhone.*Pro' 'iPhone'; do
        match="$(pick_first_match "${pattern}" || true)"
        [[ -n "${match}" ]] && break
    done
    if [[ -z "${match}" ]]; then
        echo "ERROR: no iPhone simulator available." >&2
        echo "Install one via Xcode → Settings → Platforms." >&2
        exit 1
    fi
    SIM_NAME="$(echo "${match}" | awk -F'\t' '{print $2}')"
    UDID="$(echo "${match}" | awk -F'\t' '{print $3}')"
fi
echo "==> Using simulator: ${SIM_NAME} (${UDID})"

# ---------- boot simulator (idempotent) ----------
xcrun simctl boot "${UDID}" 2>/dev/null || true
open -a Simulator --args -CurrentDeviceUDID "${UDID}"
xcrun simctl bootstatus "${UDID}" -b

# ---------- build ----------
build_args=("sim")
[[ "${MODE}" = "release" ]] && build_args+=("release")
[[ "${DO_CLEAN}" -eq 1 ]] && build_args+=("clean")

echo "==> Building ${EXAMPLE} (sim, ${MODE}) via examples/${EXAMPLE}/ios/build.sh"
( cd "${ios_dir}" && ./build.sh ${build_args[@]+"${build_args[@]}"} )

# ---------- locate built .app ----------
config="Debug"
[[ "${MODE}" = "release" ]] && config="Release"

derived="${HOME}/Library/Developer/Xcode/DerivedData"
app="$(find "${derived}" \
    -maxdepth 5 -type d -name '*.app' \
    -path "*${project_name}-*/Build/Products/${config}-iphonesimulator/*" \
    2>/dev/null | sort | tail -n1)"
if [[ -z "${app}" || ! -d "${app}" ]]; then
    echo "ERROR: built .app not found under ${derived}/${project_name}-*" >&2
    exit 1
fi
echo "==> App bundle: ${app}"

bundle_id="$(plutil -extract CFBundleIdentifier raw "${app}/Info.plist")"
if [[ -z "${bundle_id}" ]]; then
    echo "ERROR: could not read CFBundleIdentifier from ${app}/Info.plist" >&2
    exit 1
fi

trap 'echo; echo "App:    ${app}"; echo "Bundle: ${bundle_id}"' EXIT

echo "==> Installing ${app}"
xcrun simctl install "${UDID}" "${app}"

if [[ "${NO_LAUNCH}" -eq 1 ]]; then
    echo "==> Installed. Skipping launch (--no-launch)."
    if [[ "${NO_LOGS}" -eq 0 ]]; then
        echo "==> Tailing system log for ${project_name} (Ctrl-C to exit)…"
        xcrun simctl spawn "${UDID}" log stream \
            --predicate "process == \"${project_name}\"" --style compact
    fi
    exit 0
fi

echo "==> Launching ${bundle_id} (Ctrl-C kills the app)…"
xcrun simctl launch --console-pty "${UDID}" "${bundle_id}"
