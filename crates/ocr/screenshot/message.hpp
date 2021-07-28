#include <QtQml/qqml.h>

class Message : public QObject {
  Q_OBJECT

  QML_ELEMENT
public:
  Q_INVOKABLE void call(QImage const&);
};
