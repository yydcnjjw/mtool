#pragma once
#include <memory>
#include <rust/cxx.h>

#include <QGuiApplication>
#include <QQuickImageProvider>
#include <QScreen>

using rust_callback = rust::Fn<void(std::unique_ptr<std::vector<uint8_t>>)>;
int qt_run(int argc, char **argv, rust_callback);
void qt_quit();

class ScreenImageProvider : public QQuickImageProvider {
public:
  ScreenImageProvider();

  QPixmap requestPixmap(const QString &id, QSize *size,
                        const QSize &requestedSize) override;
};

std::unique_ptr<QQmlImageProviderBase> new_screen_image_provider();
