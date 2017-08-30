// #![allow(overflowing_literals)]
// todo: request the same url to many times, reduce it;


#![allow(unused_imports)]
use fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory,ReplyOpen};
use std::path::Path;
use libc::{ENOENT, ENOSYS};
use std::string::String;
use std::vec::Vec;
use url::{Url, Host, ParseError};


use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use std::fmt::Write as fWrite;
use std::io::Read;
use std::ffi::OsStr;
use std::str;
use std::option::Option;
use std::clone;
use std::error::Error;
use regex::Regex;
use std::collections::HashMap;
use time::Timespec;

pub struct WebFilesystem {
    pub root_url: String,
    next_seq: u64,
    inodes: HashMap<u64, Node>,
    sub_nodes: HashMap<u64, Vec<Node>>,
}

#[derive(Clone, Debug)]
struct Node {
    inode: u64,
    name: String,
    url: String,

    data: Vec<u8>,
}

impl Node {
    fn is_file(&self) -> bool {
        is_pic_url(&self.url)
    }

    fn size(&mut self) -> u64 {
        return self.get_data().len() as u64
    }

    fn get_data(&mut self) -> &Vec<u8> {
        if self.data.len() == 0 {
            let rt = get_url_data(&self.url);
            if let Ok(data) = rt {
                self.data = data;
            }
        }

        &self.data
    }
}
    // return (name, url)
    // todo: erase duplicat url
    fn get_files(body: &str, url: &str) -> Vec<(String, String)> {
        let mut files = Vec::new();

        let a_re = Regex::new("<a .*?href=\"(.*?)\".*>(.*?)</a>").unwrap();
        for cap in a_re.captures_iter(body) {
            files.push((cap[2].to_string(), cap[1].to_string()));
        }

        // let img_re = Regex::new("<img .*?alt=\"(.*?)\".*?src=\"(.*?)\".*?>").unwrap();
        // for cap in img_re.captures_iter(body) {
        //     files.push((cap[1].to_string(), cap[2].to_string()));
        // }

        let img_re = Regex::new("<img .*?src=\"(.*?)\".*?>").unwrap();
        for cap in img_re.captures_iter(body) {
            files.push((cap[1].to_string(), cap[1].to_string()));
        }

        let parse_url = Url::parse(url).unwrap();
        
        let mut new_files = Vec::new();


        #[allow(unused_variables)]
        for (oname, ourl) in files {
            let sub_url: Url;
            let purl = Url::parse(&ourl);
            if purl == Err(ParseError::RelativeUrlWithoutBase) {
                sub_url = parse_url.join(&ourl).unwrap();
            } else {
                sub_url = Url::parse(&ourl).unwrap();
            }

            let v: Vec<&str> = sub_url.path().split("/").collect();
            let mut name: String = v.last().unwrap().to_string();
            if name == "" {
                name = sub_url.host().unwrap().to_string();
            }

            new_files.push((name, sub_url.clone().into_string()));
        }


        new_files.sort();
        new_files.dedup_by_key( |f| f.1.clone() );
        new_files
    }

    fn is_pic_url(url: &str) -> bool {
        if url.ends_with(".jpg") {
            return true
        }
        if url.ends_with(".jpeg") {
            return true
        }
        if url.ends_with(".gif") {
            return true
        }
        if url.ends_with(".png") {
            return true
        }
        if url.ends_with(".webp") {
            return true
        }

        false
    }

    fn get_url_data(url: &str) -> Result<Vec<u8>, String> {
        let client = Client::new();
		let mut response = match client.get(url).send() {
        	Ok(response) => response,
        	Err(err) => {
                println!("get url: {} err: {}", url, err);
                return Err(err.to_string())
            }
    	};

        let mut buf = Vec::new();

        match response.read_to_end(&mut buf) {
            Ok(_) => {
                println!("get url: {} data len: {}", url, buf.len());
                // println!("get url: {} data: {:?}", url, String::from_utf8(buf.clone()));
                Ok(buf)
            }
            Err(err) => Err(err.to_string()),
        }
    }
 

impl WebFilesystem {
    pub fn new(root_url: &String) -> WebFilesystem {
        let mut fs = WebFilesystem {
            root_url: root_url.clone(),
            inodes: HashMap::new(),
            next_seq: 2,
            sub_nodes: HashMap::new(),
        };

        let root_node = Node{
            inode: 1,
            name: "fs".to_string(),
            url: root_url.clone(),
            data: Vec::new(),
        };

        fs.inodes.insert(1, root_node);

        fs
    }

    fn get_seq(&mut self) -> u64 {
        let rt = self.next_seq;
        self.next_seq += 1;
        
        rt
    }

    fn get_sub_nodes(&mut self, node: &Node) -> Vec<Node> {
        if let Some(sub_nodes) = self.sub_nodes.get(&node.inode) {
            // avoid clone?
            return sub_nodes.clone();
        }

        match get_url_data(&node.url) {
            Ok(data) => {
                let files = get_files(&String::from_utf8(data).unwrap(), &node.url);
                let mut ret = Vec::new();
                for file in files {
                    let node = Node {
                        inode: self.get_seq(),
                        name: file.0,
                        url: file.1,
                        data: Vec::new(),
                    };
                    ret.push(node.clone());
                    self.inodes.insert(node.inode, node);
                }
                self.sub_nodes.insert(node.inode, ret.clone());
                ret
            },
            Err(err) => {
                println!("get url: {} err: {}", node.url, err);
                Vec::new()
            }
        }
    }

    fn find_node(&self, ino: u64) -> Option<Node> {
        let tmp = self.inodes.get(&ino);
        match tmp {
            None => {
                None
            },
            Some(find) => {
                Some(find.clone())
            }
        }
    }

}

impl Filesystem for WebFilesystem {
	fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, mut reply: ReplyDirectory) {
    	println!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);

        let tmp = self.find_node(ino);
        let find;
    
        match tmp {
            None => {
                println!("return err");
                reply.error(ENOENT);
                return
            },
            Some(_find) => {
                find = _find;
            }
        }

        let nodes = self.get_sub_nodes(&find);
        for i in offset..nodes.len() as u64 {
            let n = &nodes[i as usize];
            let filetype = if n.is_file() {FileType:: RegularFile} else {FileType::Directory};
            let rt = reply.add(n.inode, i+1, filetype, &Path::new(&n.name));
            println!("add node: {:?}", n);
            if rt == true {
                println!("add node return full buf");
                break;
            }
        }
        println!("readdir reply ok");
        reply.ok();
	}

    fn read( &mut self, _req: &Request, _ino: u64, _fh: u64, _offset: u64, _size: u32, reply: ReplyData) {
    	println!("read(ino={}, offset={}, _size={})", _ino, _offset, _size);

        match self.inodes.get(&_ino) {
            Some(node) => {
                if node.is_file() == false {
                    println!("return err");
                    reply.error(ENOENT);
                    return
                }

                match get_url_data(&node.url) {
                    Ok(data) => {
                        // println!("{} data len: {}", node.url, data.len());
                        let mut end: usize = _offset as usize + _size as usize;
                        if end > data.len() {
                            end = data.len();
                        }
                        reply.data(&data.as_slice()[_offset as usize .. end]);
                        return
                    },
                    Err(err) => {
                        println!("err: {}", err);
                        reply.error(ENOENT);
                        return
                    }
                }
            },
            None => {
                println!("return err");
                reply.error(ENOENT);
            }
        }
    }


    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
    	println!("open(ino={}, _flags={})", _ino, _flags);

        match self.inodes.get(&_ino) {
            Some(_) => {
                reply.opened(_ino, _flags);
            },
            None => {
                println!("return err");
                reply.error(ENOENT);
            }
        }        
    }

	fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
	    println!("getattr(ino={})", ino);

        match self.inodes.get(&ino) {
            Some(node) => {
	        let ts = Timespec::new(0, 0);
    	    let attr = FileAttr {
    	        ino: ino,
    	        size: node.clone().size(),
    	        blocks: 0,
    	        atime: ts,
    	        mtime: ts,
    	        ctime: ts,
    	        crtime: ts,
    	        kind: if node.is_file() {FileType::RegularFile} else {FileType::Directory},
    	        perm: 0o755,
    	        nlink: 0,
    	        uid: 0,
    	        gid: 0,
    	        rdev: 0,
    	        flags: 0,
    	    };
	        let ttl = Timespec::new(1, 0);
	        reply.attr(&ttl, &attr);

            },
            None => {
                println!("return err");
                reply.error(ENOENT);
            }
        }
	}


	fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
	    println!("lookup(parent={}, name={:?})", parent, name);
        match self.find_node(parent) {
            Some(pnode) => {
                let nodes = self.get_sub_nodes(&pnode);
                for mut node in nodes {
                    if node.name.as_str() == name {
	                    let ts = Timespec::new(0, 0);
    	                let attr = FileAttr {
    	                    ino: node.inode,
    	                    size: node.size(),
    	                    blocks: 0,
    	                    atime: ts,
    	                    mtime: ts,
    	                    ctime: ts,
    	                    crtime: ts,
    	                    kind: if node.is_file() {FileType::RegularFile} else {FileType::Directory},
    	                    perm: 0o755,
    	                    nlink: 0,
    	                    uid: 0,
    	                    gid: 0,
    	                    rdev: 0,
    	                    flags: 0,
    	                };
                        reply.entry(&Timespec::new(1, 0), &attr, 0);
                        return
                    }
                }

	            reply.error(ENOENT);
	            return;

            },
            None => {
	            reply.error(ENOENT);
	            return;
            }
        }
	}

}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_get_files() {
        let url = "http://www.baidu.com/";
        let body = "<a href=\"./a.html\">a name</a> <img alt=\"img name\" src=\"img.png\" width=\"20\">";
        let files = get_files(body, url);
        println!("get files: {:?}", files);
        assert_ne!(files.len(), 0);

        let expect = vec![("a.html".to_string(), "http://www.baidu.com/a.html".to_string()), ("img.png".to_string(), "http://www.baidu.com/img.png".to_string())];
        assert_eq!(files, expect);
    }

    #[test]
    fn test_get_url_data() {
        let url = "http://www.baidu.com";
        let rt = get_url_data(url);
        if let Err(err) = rt {
            panic!(err);
        } else {
            println!("url_data len: {}", rt.unwrap().len());
        }
    }

    #[test]
    fn see_url_files() {
        let url = "http://www.baidu.com";
        let data = get_url_data(url).unwrap();

        let files = get_files(str::from_utf8(&data[..]).unwrap(), url);
        println!("*files*: {:?}", files);
    }

}
