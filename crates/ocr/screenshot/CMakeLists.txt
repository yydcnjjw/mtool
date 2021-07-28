cmake_minimum_required(VERSION 3.14)

project(screenshot VERSION 0.1 LANGUAGES CXX C)

set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt5 COMPONENTS Core Quick REQUIRED)

set(PROJECT_SOURCES
  screenshot.cc
  message.cc
  qml.qrc
)

add_library(screenshot SHARED
  ${PROJECT_SOURCES}
  )

target_compile_definitions(screenshot
  PRIVATE $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>)
target_link_libraries(screenshot
  PRIVATE Qt5::Core Qt5::Quick)

add_executable(screenshot_main
  main.c
  )
target_link_libraries(screenshot_main
  PRIVATE screenshot
  )

install(TARGETS screenshot DESTINATION .)
