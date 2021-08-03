#include <rust/screen_image_provider.hpp>

#include <QGuiApplication>
#include <QQuickImageProvider>
#include <QScreen>

namespace rust {

struct ScreenImageProvider : public QQuickImageProvider {
public:
  ScreenImageProvider() : QQuickImageProvider(QQuickImageProvider::Pixmap) {}

  QPixmap requestPixmap(const QString &id, QSize *size,
                        const QSize &requestedSize) override {
    auto screen = QGuiApplication::primaryScreen();

    if (!screen) {
      return QPixmap();
    }

    return screen->grabWindow(0);
  }
};

ScreenImageProvider const *new_screen_image_provider() {
  return new ScreenImageProvider();
}

} // namespace rust
