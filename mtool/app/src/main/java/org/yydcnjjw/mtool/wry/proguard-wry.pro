# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class org.yydcnjjw.mtool.wry.* {
  native <methods>;
}

-keep class org.yydcnjjw.mtool.wry.WryActivity {
  public <init>(...);

  void setWebView(org.yydcnjjw.mtool.wry.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class org.yydcnjjw.mtool.wry.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class org.yydcnjjw.mtool.wry.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class org.yydcnjjw.mtool.wry.RustWebChromeClient,org.yydcnjjw.mtool.wry.RustWebViewClient {
  public <init>(...);
}
