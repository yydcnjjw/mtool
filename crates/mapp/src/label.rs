use std::{
    any::{type_name, TypeId},
    fmt,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Label(TypeId, &'static str, &'static str, Option<u64>);

impl Label {
    pub fn new_with_name<T>(s: &'static str) -> Self
    where
        T: 'static,
    {
        Label(TypeId::of::<T>(), type_name::<T>(), s, None)
    }

    pub fn new<T>() -> Self
    where
        T: 'static,
    {
        Label(
            TypeId::of::<T>(),
            type_name::<T>(),
            "",
            Some(rand::random()),
        )
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)?;
        if !self.2.is_empty() {
            write!(f, "::{}", self.2)?;
        }

        if let Some(v) = self.3 {
            write!(f, "::{}", v)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! define_label {
    (
        $vis:vis enum $scope:ident {
            $(
                $name:ident,
            )*
        }
    ) => {
        $vis enum $scope {
            $(
                $name,
            )*
        }

        impl From<$scope> for Label {
            fn from(v: $scope) -> Self {
                match v {
                    $(
                        $scope::$name => Label::new_with_name::<$scope>(stringify!($name)),
                    )*
                }
            }
        }
    };
    (
        $vis:vis $scope:ident, $name:ident
    ) => {
        $vis enum $scope {
            $name,
        }

        impl From<$scope> for Label {
            fn from(_: $scope) -> Self {
                Label::new_with_name::<$scope>(stringify!($name))
            }
        }
    };
}
