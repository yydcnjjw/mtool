#pragma once

#include <memory>
#include <ocr/src/app.rs.h>
#include <rust/cxx.h>

namespace rust {

// using message_cb = Fn<void(std::unique_ptr<std::vector<uint8_t>>)>;

void qml_register_message(App const &);
void qt_quit();

} // namespace rust
