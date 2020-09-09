#include "message.hpp"

#include <QBuffer>
#include <QDebug>
#include <QGuiApplication>
#include <QImage>
#include <QQmlEngine>

namespace rust {

void Message::call(QImage const &img) {
  QByteArray ba;
  QBuffer buffer{&ba};
  buffer.open(QIODevice::WriteOnly);

  if (img.save(&buffer, "PNG")) {
    _app.ocr(std::make_unique<std::vector<uint8_t>>(ba.begin(), ba.end()));
  } else {
    qDebug() << "Image save failure";
  }
}

void qml_register_message(App const &app) {
  qmlRegisterSingletonInstance("demo", 1, 0, "Message", new Message{app});
}

void qt_quit() { QGuiApplication::quit(); }

} // namespace rust
