mod action;
mod keybinding;
mod keyboard;
mod keymap;

pub use action::*;
pub use keybinding::Keybinding;
pub use mkeybinding::*;

#[macro_export]
macro_rules! generate_keymap {
    ($(($kbd:expr, $action:expr),)+) => {
        $crate::prelude::KeyMap::<$crate::prelude::Action>::new_with_vec(
            vec![
                $(
                    (
                        $kbd,
                        $action
                    )
                ),+
            ]
        )
    };
}

#[macro_export]
macro_rules! local_action {
    ($action:expr) => {
        $crate::prelude::Action::Local(::std::sync::Arc::new(
            ::mapp::send_wrapper::SendWrapper::new(Box::new($action)),
        ))
    };
}

#[macro_export]
macro_rules! shared_action {
    ($action:expr) => {
        $crate::prelude::Action::Shared(::std::sync::Arc::new(Box::new($action)))
    };
}

#[macro_export]
macro_rules! __tail_ident {
    ($($deref:ident).*) => {
        $crate::__tail_ident![@ $($deref).*]
    };

    ($($deref:ident)* @ $head:ident $( . $tail:ident)+) => {
        $crate::__tail_ident![$($deref)* $head @ $($tail).+]
    };

    ($($deref:ident)* @ $last:ident) => {
        $last
    };
}

#[macro_export]
macro_rules! __call {
    ($fn:expr, $($es:ident),*) => {
        $fn($( $crate::__tail_ident![$es] ),*)
    };
}

#[macro_export]
macro_rules! local_action_fn {
    ($fn:expr, $($rest:tt)*) => {
        $crate::local_action!({
            to_owned![$($rest)*];
            move || {
                to_owned![$($rest)*];
                $crate::__call![$fn, $($rest)*]
            }
        })
    };
}
