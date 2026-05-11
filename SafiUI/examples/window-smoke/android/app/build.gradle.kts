plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.safiui.windowsmoke"
    compileSdk = 35
    // ndkVersion intentionally left unset — Gradle will pick the highest
    // installed NDK from $ANDROID_HOME/ndk/. Tested with r26d and r30+.

    defaultConfig {
        applicationId = "com.safiui.windowsmoke"
        minSdk = 24      // PRD §9.1 — Vulkan guaranteed
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            // arm64 only — matches `cargo ndk -t arm64-v8a` output.
            abiFilters += listOf("arm64-v8a")
        }
    }

    sourceSets["main"].apply {
        java.srcDirs("src/main/kotlin")
        jniLibs.srcDirs("src/main/jniLibs")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildTypes {
        debug { isDebuggable = true }
        release {
            isMinifyEnabled = false
            isShrinkResources = false
        }
    }
}

dependencies {
    // SDL3's Android Java glue (SDLActivity, SDLAudioManager, etc.) is
    // shipped inside the SDL3 source tree. The sdl3-rs crate vendors and
    // builds SDL3 from source for the .so, but the Java helpers must be
    // pulled in separately. The `build.sh` helper copies the right files
    // out of the cargo cache into this app's java/ source set before
    // gradle assembles.
    implementation("androidx.core:core-ktx:1.13.1")
}
