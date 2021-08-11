#pragma once

#include <QObject>

#include <rust/message.hpp>

namespace rust {
class Message : public QObject {
  Q_OBJECT
public:
  Message(App const &app, QObject *parent = nullptr)
      : QObject(parent), _app(app) {}

  Q_INVOKABLE void call(QImage const &);

private:
  App const &_app;
};
} // namespace rust
