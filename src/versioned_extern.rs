/// Drop in replacement for `extern` blocks. Sets the `link_name` of every
/// symbol to the mangled version.
#[macro_export]
macro_rules! versioned_extern {
    (fn $($rest:tt)+) => (
        versioned_extern!([fn] $($rest)+);
    );

    (pub fn $($rest:tt)+) => (
        versioned_extern!([pub fn] $($rest)+);
    );

    (pub static $name:ident : $T:path; $($rest:tt)*) => (
        versioned_extern!(
            $($rest)*
            ([pub static] $name [: $T;])
        );
    );

    (static $name:ident : $T:path; $($rest:tt)*) => (
        versioned_extern!(
            $($rest)*
            ([static] $name [: $T;])
        );
    );

    ([$($pre:tt)+] $name:ident ($($args:tt)*); $($rest:tt)*) => (
        versioned_extern!(
            $($rest)*
            ([$($pre)+] $name [($($args)*);])
        );
    );

    ([$($pre:tt)+] $name:ident ($($args:tt)*) -> $T:ty; $($rest:tt)*) => (
        versioned_extern!(
            $($rest)*
            ([$($pre)+] $name [($($args)*) -> $T;])
        );
    );

    ($(([$($pre:tt)+] $name:ident [$($post:tt)+]))+) => (
        versioned_extern! {
            $(
                [concat!(
                    stringify!($name),
                    "_v", env!("CARGO_PKG_VERSION_MAJOR"),
                    "_", env!("CARGO_PKG_VERSION_MINOR"),
                    "_", env!("CARGO_PKG_VERSION_PATCH"),
                    "_", env!("CARGO_PKG_VERSION_PRE")
                )]

                [$($pre)+]

                $name

                [$($post)+]
            )+
        }
    );

    ($([$v:expr] [$($pre:tt)+] $name:ident [$($post:tt)+])+) => (
        extern {$(
            #[link_name = $v]
            $($pre)+ $name $($post)+
        )+}
    );

    () => ();

    ($($rest:tt)*) => ($($rest)*);
}
