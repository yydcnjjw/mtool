#include <message.hpp>
#include <QCoreApplication>
#include <QImage>

#include <iostream>

void Message::call(QImage const &img) {
  std::cout << "call" << std::endl;
  QCoreApplication::quit();
}

