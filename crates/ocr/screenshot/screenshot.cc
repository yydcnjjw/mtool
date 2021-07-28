#include <screenshot.hpp>
#include <message.hpp>

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickImageProvider>
#include <QScreen>


class ScreenImageProvider : public QQuickImageProvider {
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

int qt_run(int argc, char *argv[]) {
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


  auto msg = std::make_shared<Message>();
  qmlRegisterSingletonInstance("demo", 1, 0, "Message", msg.get());


  engine.load(url);

  return app.exec();
}
