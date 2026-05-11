// Root build file. Plugin versions kept here; individual modules apply them.
plugins {
    // AGP 8.10+ supports JDK 25 build runtime; Kotlin 2.1+ fixes the
    // JavaVersion parser bug ("IllegalArgumentException: 25.0.2") that
    // earlier Kotlin versions hit when running under JDK 25.
    id("com.android.application") version "8.10.1" apply false
    id("org.jetbrains.kotlin.android") version "2.1.21" apply false
}
