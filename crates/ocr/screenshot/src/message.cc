#include "message.hpp"

#include <ocr/src/lib.rs.h>
#include <rust/cxx.h>

#include <QBuffer>
#include <QImage>
#include <QDebug>
#include <QQmlEngine>

namespace rust {

void Message::call(QImage const &img) {
  QByteArray ba;
  QBuffer buffer{&ba};
  buffer.open(QIODevice::WriteOnly);

  if (img.save(&buffer, "PNG")) {

    ocr_test(std::make_unique<std::vector<uint8_t>>(ba.begin(), ba.end()));

  } else {
    qDebug() << "Image save failure";
  }
}

void qml_register_message() {
  
  qDebug() << "register message";
  
  qmlRegisterSingletonType<Message>(
      "demo", 1, 0, "Message",
      [](QQmlEngine *engine, QJSEngine *scriptEngine) -> QObject * {
        Q_UNUSED(engine)
        Q_UNUSED(scriptEngine)

        return new Message;
      });
}

} // namespace rust
