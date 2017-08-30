extern crate fuse;
extern crate webfile;

use std::env;
// use fuse::Filesystem;
use webfile::WebFilesystem;	


fn main() {
    let mountpoint = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: {} <MOUNTPOINT> <URL>", env::args().nth(0).unwrap());
            return;
        }
    };

    let url = match env::args().nth(2) {
        Some(url) => url,
        None => {
            println!("Usage: {} <MOUNTPOINT> <URL>", env::args().nth(0).unwrap());
            return;
        }
    };

    let fs = WebFilesystem::new(&url.to_string());

    fuse::mount(fs, &mountpoint, &[]).expect("mount leave");
    // let rt = fuse::spawn_mount(fs, &mountpoint, &[])
}

