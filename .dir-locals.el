;;; Directory Local Variables            -*- no-byte-compile: t -*-
;;; For more information see (info "(emacs) Directory Variables")

((nil . ((lsp-enable-file-watchers . nil)
         ;; (lsp-rust-all-features . nil)
         (eval . (progn
                   (let ((extra-env (make-hash-table)))
                     (cond
                      ((string-equal lsp-rust-analyzer-cargo-target "x86_64-pc-windows-msvc")
                       (let ((includes "/home/yydcnjjw/.xwin-cache/splat/sdk/include/ucrt:/home/yydcnjjw/.xwin-cache/splat/crt/include"))
                         (puthash "AWS_LC_SYS_INCLUDES" includes extra-env)))
                      ((string-equal lsp-rust-analyzer-cargo-target "aarch64-linux-android")
                       (let* ((android-ndk (getenv "ANDROID_NDK"))
                              (android-toolchain (expand-file-name "toolchains/llvm/prebuilt/linux-x86_64" android-ndk))
                              (env-path (concat (expand-file-name "bin" android-toolchain) path-separator
                                                (getenv "PATH"))))
                         ;; for aws-lc
                         (puthash "PATH" env-path extra-env)
                         (puthash "ANDROID_NDK" android-ndk extra-env)
                         (puthash "BINDGEN_EXTRA_CLANG_ARGS" (format "--sysroot=%s" (expand-file-name "sysroot" android-toolchain)) extra-env))))
                     (setq-local lsp-rust-analyzer-cargo-extra-env extra-env)))
               ))))
