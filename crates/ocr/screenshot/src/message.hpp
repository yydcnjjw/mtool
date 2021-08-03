#pragma once

#include <rust/message.hpp>

#include <QObject>

namespace rust {
class Message : public QObject {
  Q_OBJECT
public:
  Message(QObject *parent = nullptr) : QObject(parent) {}

  Q_INVOKABLE void call(QImage const &);
};
} // namespace rust
