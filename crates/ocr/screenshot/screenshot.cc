#include <message.hpp>
#include <screenshot.hpp>

#include <QClipboard>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickImageProvider>
#include <QScreen>
#include <iostream>

ScreenImageProvider::ScreenImageProvider()
    : QQuickImageProvider(QQuickImageProvider::Pixmap) {}

QPixmap ScreenImageProvider::requestPixmap(const QString &id, QSize *size,
                                           const QSize &requestedSize) {

  auto screen = QGuiApplication::primaryScreen();

  if (!screen) {
    return QPixmap();
  }

  return screen->grabWindow(0);
}

std::unique_ptr<QQmlImageProviderBase> new_screen_image_provider() {
  return std::make_unique<ScreenImageProvider>();
}

int qt_run(int argc, char **argv, rust_callback cb) {
  QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);

  QGuiApplication app(argc, argv);

  QQmlApplicationEngine engine;
  const QUrl url(QStringLiteral("qrc:/main.qml"));
  QObject::connect(
      &engine, &QQmlApplicationEngine::objectCreated, &app,
      [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
          QCoreApplication::exit(-1);
      },
      Qt::QueuedConnection);

  engine.addImageProvider(QLatin1String("screen"), new ScreenImageProvider);

  auto msg = std::make_shared<Message>(cb);
  qmlRegisterSingletonInstance("demo", 1, 0, "Message", msg.get());

  engine.load(url);

  return app.exec();
}

void qt_quit() { QCoreApplication::quit(); }
