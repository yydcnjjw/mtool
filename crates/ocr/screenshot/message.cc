#include <QBuffer>
#include <QCoreApplication>
#include <QImage>
#include <message.hpp>

#include <vector>

void Message::call(QImage const &img) {
  QByteArray ba;
  QBuffer buffer{&ba};
  buffer.open(QIODevice::WriteOnly);

  if (img.save(&buffer, "PNG")) {
    _cb(std::make_unique<std::vector<uint8_t>>(ba.begin(), ba.end()));
  } else {
    qDebug() << "Image save failure";
  }
}
