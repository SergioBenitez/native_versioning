// #![feature(trace_macros)]

// trace_macros!(true);

#[macro_use]
extern crate native_versioning;

mod c {
    pub type long = u16;
}

versioned_extern! {
    static demo: c::long;
}

pub fn main() {
    
}
