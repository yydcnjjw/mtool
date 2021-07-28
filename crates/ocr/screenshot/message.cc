#include <QBuffer>
#include <QCoreApplication>
#include <QImage>
#include <message.hpp>

#include <vector>

void Message::call(QImage const &img) {
  auto vec = std::make_unique<std::vector<uint8_t>>(img.sizeInBytes(), 0);
  QBuffer buffer;
  buffer.setData(reinterpret_cast<char const *>(vec->data()), vec->size());
  buffer.open(QIODevice::WriteOnly);

  img.save(&buffer, "PNG");

  _cb(std::move(vec));

  QCoreApplication::quit();
}
