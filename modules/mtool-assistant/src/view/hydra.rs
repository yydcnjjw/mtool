// use dioxus::prelude::*;
// use mapp::{
//     dpi::{PhysicalPosition, PhysicalSize},
//     prelude::*,
//     tracing::warn,
// };
// use mtool_dioxus::{
//     components::WindowView,
//     desktop::{use_global_shortcut, window, HotKeyState},
//     prelude::*,
// };

// use crate::context::{AssistantContext, AssistantMode};

// #[component]
// pub fn MainView() -> Element {
//     init_window();

//     init_keybinding();

//     rsx! {
//         WindowView {
//             DockView {  }
//         }
//     }
// }

// #[component]
// fn DockView() -> Element {
//     use_hook(|| {});

//     rsx! {
//         div {
//             class: "@container h-screen flex flex-row-reverse bg-transparent",
//             div {
//                 class: "w-screen @3xs:w-full transition-[width] duration-300 ease-in-out bg-transparent",
//                 "test"
//             }
//         }
//     }
// }

// #[component]
// fn ShowDockView() -> Element {
//     let onmousedown = move |e: Event<MouseData>| {
//         if e.modifiers().shift() {
//             let _ = window().drag_window();
//         }
//     };

//     let ctx = use_app_resource::<Res<AssistantContext>>().suspend()?;
//     let mode = ctx().mode_change_signal();

//     let onclick = use_callback({
//         move |_: Event<MouseData>| {
//             ctx().toggle_mode();
//         }
//     });

//     rsx! {
//         WindowView {
//             div {
//                 class: "flex flex-col items-center h-screen",
//                 onmousedown,
//                 button {
//                     class: "btn btn-wide btn-xl",
//                     onclick,
//                     {
//                         match mode() {
//                             AssistantMode::Desktop => "Desktop",
//                             AssistantMode::RemoteDesktop => "Remote Dekstop",
//                         }
//                     }
//                 }
//                 // SuspenseBoundary {
//                 //     fallback: |_| rsx! {
//                 //         div { "Loading media player control" }
//                 //     },
//                 //     MediaPlayerControl {  }
//                 // }
//             }
//         }
//     }
// }

// #[derive(Clone, Copy)]
// enum WindowMode {
//     ShowDock,
//     Dock,
// }

// fn init_keybinding() {
//     let mut mode = use_signal(|| WindowMode::Dock);

//     if let Err(e) = use_global_shortcut("alt+z", move |state| {
//         if state == HotKeyState::Pressed {
//             match mode() {
//                 WindowMode::ShowDock => {
//                     mode.set(WindowMode::Dock);
//                     let win = window();
//                     let (width, height) = (32, 256);
//                     let window_size = PhysicalSize::new(width, height);
//                     let _ = win.request_inner_size(window_size);

//                     if let Some(monitor) = win.current_monitor() {
//                         let monitor_size = monitor.size();
//                         win.set_outer_position(PhysicalPosition::new(
//                             monitor_size.width - width,
//                             64,
//                         ));
//                     }
//                 }
//                 WindowMode::Dock => {
//                     mode.set(WindowMode::ShowDock);
//                     let win = window();
//                     let (width, height) = (300, 256);
//                     let window_size = PhysicalSize::new(width, height);
//                     let _ = win.request_inner_size(window_size);

//                     if let Some(monitor) = win.current_monitor() {
//                         let monitor_size = monitor.size();
//                         win.set_outer_position(PhysicalPosition::new(
//                             monitor_size.width - width,
//                             64,
//                         ));
//                     }
//                 }
//             }
//         }
//     }) {
//         warn!("{:?}", e);
//     }
// }

// fn init_window() {
//     use_hook(move || {
//         let win = window();
//         win.set_visible(true);
//     });
// }
