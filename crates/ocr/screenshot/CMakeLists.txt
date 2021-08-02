cmake_minimum_required(VERSION 3.14)

project(screenshot VERSION 0.1 LANGUAGES CXX C)

set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt5 COMPONENTS Core Quick REQUIRED)
find_package(KF5GlobalAccel REQUIRED)

set(PROJECT_SOURCES
  screenshot.cc
  message.cc
  qml.qrc
)

add_library(screenshot SHARED
  ${PROJECT_SOURCES}
  )

target_link_libraries(screenshot
  PUBLIC
  Qt5::Core
  Qt5::Quick)

target_include_directories(screenshot SYSTEM
  PUBLIC
  ${CMAKE_CURRENT_SOURCE_DIR}/../../../target/cxxbridge)


add_library(screenshot-static STATIC
  ${PROJECT_SOURCES}
  )

target_link_libraries(screenshot-static
  PUBLIC
  Qt5::Core
  Qt5::Quick)

target_include_directories(screenshot-static SYSTEM
  PUBLIC
  ${CMAKE_CURRENT_SOURCE_DIR}/../../../target/cxxbridge)

add_executable(screenshot_main
  main.cc
  )
target_link_libraries(screenshot_main
  PRIVATE screenshot
  )

install(TARGETS screenshot DESTINATION .)
install(TARGETS screenshot-static DESTINATION .)
