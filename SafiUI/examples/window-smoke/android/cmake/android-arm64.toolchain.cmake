# Wrapper toolchain — pins ANDROID_ABI before delegating to the real
# android.toolchain.cmake. Workaround for the cmake-rs + sdl3-sys chain
# not passing -DANDROID_ABI to the nested CMake invocation, which would
# otherwise default to armeabi-v7a and fail against NDK r27+ (no 32-bit
# prebuilts).
#
# build.sh exports ANDROID_NDK_HOME before invoking cargo, so the include
# path resolves at configure time.

set(ANDROID_ABI arm64-v8a)
set(ANDROID_PLATFORM android-24)   # matches PRD §9.1 minSdk
# Use shared C++ runtime — build.sh copies libc++_shared.so into the APK's
# jniLibs/arm64-v8a/ next to our .so so dlopen resolves SDL3's C++ symbols
# (`__gxx_personality_v0`, `__cxa_*`) at runtime.
set(ANDROID_STL c++_shared)

if(NOT DEFINED ENV{ANDROID_NDK_HOME})
    message(FATAL_ERROR
        "android-arm64.toolchain.cmake: ANDROID_NDK_HOME env var is required.")
endif()

include("$ENV{ANDROID_NDK_HOME}/build/cmake/android.toolchain.cmake")
