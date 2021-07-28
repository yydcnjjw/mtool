#include <QtQml/qqml.h>
#include <screenshot.hpp>

class Message : public QObject {
  Q_OBJECT

  QML_ELEMENT
public:
  Message(rust_callback cb) : _cb(cb) {}
  Q_INVOKABLE void call(QImage const &);
private:
  rust_callback _cb;
};
