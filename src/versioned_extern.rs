/// Drop in replacement for `extern` blocks. Sets the `link_name` of every
/// symbol to the mangled version.
#[macro_export]
macro_rules! versioned_extern {
    ($(#[$($attr:tt)*])* fn $($rest:tt)+) => (
        versioned_extern!([$(#[$($attr)*])* fn] $($rest)+);
    );

    ($(#[$($attr:tt)*])* pub fn $($rest:tt)+) => (
        versioned_extern!([$(#[$($attr)*])* pub fn] $($rest)+);
    );

    ($(#[$($attr:tt)*])* static $($rest:tt)+) => (
        versioned_extern!([$(#[$($attr)*])* static] $($rest)+);
    );

    ($(#[$($attr:tt)*])* pub static $($rest:tt)+) => (
        versioned_extern!([$(#[$($attr)*])* pub static] $($rest)+);
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

    ([$($pre:tt)+] $name:ident : $T:path; $($rest:tt)*) => (
        versioned_extern!(
            $($rest)*
            ([$($pre)+] $name [: $T;])
        );
    );

    ($(([$($pre:tt)+] $name:ident [$($post:tt)+]))+) => (
        versioned_extern! {
            $(
                [concat!(stringify!($name), "_", env!("NATIVE_VERSIONING_VERSION"))]

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

    ($($rest:tt)*) => ($($rest)*);
}
