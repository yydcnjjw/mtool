dx-android-out-dir := "target/dx/mtool/debug/android"
dx-android-libs := dx-android-out-dir / "app/app/src/main/jniLibs/arm64-v8a/libdioxusmain.so"

alias ba := build-android
alias ra := run-android

build-android:
    dx build --target aarch64-linux-android -p mtool --platform android
    llvm-strip {{dx-android-libs}}

run-android: build-android
    ./gradlew :mtool:android:assembleDebug
    ./gradlew :mtool:android:installDebug


generate-android-pkg:
    export WRY_ANDROID_PACKAGE=org.yydcnjjw.mtool.wry
    export WRY_ANDROID_KOTLIN_FILES_OUT_DIR=$(pwd)/mtool/app/src/main/java/org/yydcnjjw/mtool/wry


build:
     dx build --target x86_64-pc-windows-msvc -p mtool --platform windows --features 'bevy,bevy_dynamic_linking'

serve:
     dx serve --target x86_64-pc-windows-msvc -p mtool --platform windows --addr 127.0.0.1 --features 'bevy,bevy_dynamic_linking'

package:
     dx bundle -p mtool --platform windows --package-types msi --verbose

alias wt := watch-tailwindcss

watch-tailwindcss:
    npx @tailwindcss/cli -i modules/mtool-dioxus/tailwind/tailwind.css -o modules/mtool-dioxus/assets/tailwind.css --watch
