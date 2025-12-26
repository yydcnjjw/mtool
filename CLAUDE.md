# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based application called "mtool" that uses a modular architecture with a multi-crate workspace. The application provides a cross-platform GUI application built with Dioxus, supporting desktop, mobile, and potentially web platforms.

## Architecture

- **Workspace Structure**: The project uses a Rust workspace with crates in the `crates/` directory and modules in the `modules/` directory
- **Core Components**:
  - `mapp`: Core application framework with dependency injection and async support
  - `mtool`: Main application binary that assembles modules based on feature flags
  - Multiple feature-flagged modules for different functionality

- **Key Modules**:
  - `mtool-core`: Core functionality with optional Python integration via PyO3
  - `mtool-system`: System-level event handling
  - `mtool-dioxus`: GUI components using the Dioxus framework
  - `mtool-p2p`: Peer-to-peer networking capabilities
  - `mtool-storage`: Data storage functionality
  - `mtool-assistant`: Assistant/AI-related features
  - `mtool-cmdpal`: Command palette functionality
  - `mtool-proxy`: Proxy functionality
  - `mtool-apps`: Application management

- **Cross-Platform Support**:
  - Desktop (Windows, Linux, macOS) via Dioxus desktop
  - Mobile (Android) via Dioxus mobile with feature flag 'mobile'
  - Python integration via PyO3 for cross-platform scripting

## Build System

The project uses a hybrid build system with:
- **Cargo**: Rust package manager and build system
- **Dioxus CLI**: For cross-platform GUI applications
- **Gradle**: For Android builds
- **Just**: For common development commands (defined in justfile)

## Common Development Commands

### Building
```bash
# Build the main application
cargo build -p mtool

# Build with desktop features (default)
cargo build -p mtool --features desktop

# Build with mobile features
cargo build -p mtool --features mobile --no-default-features

# Build with server features
just build-server

# Build for Android
just build-android
```

### Running
```bash
# Run development version
cargo run -p mtool

# Run with specific features
cargo run -p mtool --features desktop

# Run Android version
just run-android

# Run with development server
just serve
```

### Other Useful Commands
```bash
# Build for Windows
just build

# Package for distribution
just package

# Watch and rebuild Tailwind CSS
just watch-tailwindcss

# Build Emacs module
just build-emacs

# Build Bevy-enhanced version
just build-bevy
```

## Key Technologies

- **Rust**: Core language and ecosystem
- **Dioxus**: Cross-platform GUI framework
- **PyO3**: Python integration for scripting
- **Tokio**: Async runtime
- **Bevy**: Optional game engine integration (when using bevy features)
- **Tailwind CSS**: Styling with DaisyUI components
- **SeaORM**: Optional database ORM
- **Clap**: Command-line argument parsing

## Development Workflow

- Features are controlled by Cargo feature flags in the main `mtool` crate
- Cross-compilation is handled via Dioxus and Just commands
- The application uses a plugin/module architecture via the `mapp` crate
- Python integration is optional and enabled with the "python" feature flag