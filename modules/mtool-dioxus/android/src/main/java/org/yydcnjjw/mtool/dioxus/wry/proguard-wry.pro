# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class org.yydcnjjw.mtool.wry.* {
  native <methods>;
}

-keep class org.yydcnjjw.mtool.dioxus.wry.WryActivity {
  public <init>(...);

  void setWebView(org.yydcnjjw.mtool.dioxus.wry.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class org.yydcnjjw.mtool.dioxus.wry.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class org.yydcnjjw.mtool.dioxus.wry.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class org.yydcnjjw.mtool.dioxus.wry.RustWebChromeClient,org.yydcnjjw.mtool.dioxus.wry.RustWebViewClient {
  public <init>(...);
}
