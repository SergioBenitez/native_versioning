#![feature(trace_macros)]

trace_macros!(true);

#[macro_use]
extern crate native_versioning;

mod c {
    pub type long = u16;
}

versioned_extern! {
    static demo: c::long;

    pub static demo2: usize;

    #[cfg(test)]
    #[doc = "hi"]
    fn f() -> usize;

    pub fn g();
}

pub fn main() { }
