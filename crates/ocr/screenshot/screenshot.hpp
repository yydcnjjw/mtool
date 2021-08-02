#pragma once

#include <rust/cxx.h>
#include <memory>

using rust_callback = rust::Fn<void(std::unique_ptr<std::vector<uint8_t>>)>;
int qt_run(int argc, char **argv, rust_callback);
void qt_quit();
