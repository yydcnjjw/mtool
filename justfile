alias ba := build-android
alias ra := run-android

dx_out_dir := `pwd` / "target/dx/mtool/debug"
python_version := "3.13"

build-android:
    #!/usr/bin/env zsh
    set -euo pipefail
    dx_android_out_dir={{dx_out_dir}}/android
    dx_android_jnilibs_dir=$dx_android_out_dir/app/app/src/main/jniLibs/arm64-v8a
    dx_android_assets_dir=$dx_android_out_dir/app/app/src/main/assets
    dx_android_libs=$dx_android_jnilibs_dir/libdioxusmain.so

    android_toolchain=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64

    # for aws-lc
    export PATH=$android_toolchain/bin:$PATH
    export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$android_toolchain/sysroot"

    export PYO3_CROSS_PYTHON_VERSION={{python_version}}
    export PYO3_CROSS_LIB_DIR=/home/yydcnjjw/workspace/project/cross-libs/aarch64-linux-android/python-{{python_version}}

    dx build --target aarch64-linux-android -p mtool --platform android --android --no-default-features
    cp $PYO3_CROSS_LIB_DIR/libpython*.so $PYO3_CROSS_LIB_DIR/lib*_python.so $dx_android_jnilibs_dir

    llvm-strip $dx_android_libs

    dx_android_python_dir=$dx_android_assets_dir/python
    mkdir -p $dx_android_python_dir
    cp $PYO3_CROSS_LIB_DIR/python*.zip $dx_android_assets_dir
    uv build --all-packages --wheel
    uv pip install dist/*.whl --target $dx_android_python_dir/python{{python_version}}/site-packages --compile-bytecode
    cp -r $PYO3_CROSS_LIB_DIR/lib-dynload $dx_android_python_dir/python{{python_version}}

run-android: build-android
    ./gradlew :mtool:android:assembleDebug
    ./gradlew :mtool:android:installDebug

alias bs := build-server
build-server:
    cargo build --target x86_64-unknown-linux-musl -p mtool --features 'server' --no-default-features

build:
    #!/usr/bin/env zsh
    set -euo pipefail
    dx_windows_out_dir={{dx_out_dir}}/windows
    dx_windows_app_dir=$dx_windows_out_dir/app

    dx build --target x86_64-pc-windows-msvc -p mtool --platform windows --desktop --no-default-features

    uv build --all-packages --wheel
    uv pip install dist/*.whl --target $dx_windows_app_dir/site-packages --compile-bytecode

serve:
    #!/usr/bin/env zsh
    export PYTHONPATH=/mnt/d/workspace/project/cross-libs/x86_64-pc-windows-msvc/python3.13:$PYTHONPATH
    export PATH=$PYTHONPATH:$PATH
    export WSLENV=PYTHONPATH/wpl:${WSLENV}

    dx serve --target x86_64-pc-windows-msvc -p mtool --platform windows --desktop --no-default-features --addr 127.0.0.1

run: build
    #!/usr/bin/env zsh
    export PYTHONPATH=/mnt/d/workspace/project/cross-libs/x86_64-pc-windows-msvc/python3.13:$PYTHONPATH
    export PATH=$PYTHONPATH:$PATH
    export WSLENV=PYTHONPATH/wpl:${WSLENV}

    cd {{dx_out_dir}}/windows/app && ./mtool.exe

alias bp := build-python

build-python:
    #!/usr/bin/env zsh
    python_cross_libs=/mnt/d/workspace/project/cross-libs/x86_64-pc-windows-msvc/python3.13
    uv build --all-packages --wheel
    uv pip install dist/*.whl --target $python_cross_libs/site-packages --compile-bytecode

build-bevy:
    dx build --target x86_64-pc-windows-msvc -p mtool --platform windows --features 'desktop,bevy,bevy_dynamic_linking'

serve-bevy:
    dx serve --target x86_64-pc-windows-msvc -p mtool --platform windows --addr 127.0.0.1 --features 'desktop,bevy,bevy_dynamic_linking'

package:
    dx bundle -p mtool --platform windows --package-types msi --verbose

alias wt := watch-tailwindcss

watch-tailwindcss:
    npx @tailwindcss/cli -i modules/mtool-dioxus/tailwind/tailwind.css -o modules/mtool-dioxus/assets/tailwind.css --watch
