#[macro_use(format)]
extern crate alloc;

mod stdlib {
    pub mod alloc {
        pub use ::alloc::*;
        pub mod collections {
            pub use hashbrown::HashMap;
        }
    }
}
