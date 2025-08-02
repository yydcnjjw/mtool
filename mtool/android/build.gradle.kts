plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}

android {
    namespace = "org.yydcnjjw.mtool"
    compileSdk = 36

    defaultConfig {
        applicationId = "org.yydcnjjw.mtool"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    sourceSets {
        getByName("main") {
            val dxAndroidAppDir = rootDir.resolve("target/dx/mtool/debug/android/app/app")
            assets {
                srcDir("${dxAndroidAppDir.resolve("src/main/assets")}")
            }
            jniLibs {
                srcDir("${dxAndroidAppDir.resolve("src/main/jniLibs")}")
            }
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }

    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation(project(":modules:mtool-dioxus:android"))
    implementation(project(":modules:mtool-sytem:android"))
}