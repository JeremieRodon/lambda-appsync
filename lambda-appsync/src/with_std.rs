#[macro_use(format)]
extern crate std;

mod stdlib {
    pub mod alloc {
        pub use std::borrow;
        pub use std::collections;
        pub use std::string;
        pub use std::vec;
    }
}
