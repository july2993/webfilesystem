
extern crate hyper;
extern crate regex;
extern crate fuse;
extern crate libc;
extern crate futures;
extern crate tokio_core;
extern crate time;
extern crate url;

pub mod filesystem;

pub use filesystem::WebFilesystem;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

