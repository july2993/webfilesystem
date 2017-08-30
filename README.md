### WebFilesystem: mount a url as a filesystem using fuse to see the pictures.



write it as a toy project to struggle with compile error using Rust.



#### usage

```
cargo run /data/hello "http://pic.baidu.com"
```

then in /data/hello you can see:

```
➜  hello ll
total 0
-rwxr-xr-x 0 root root  33K 1月   1  1970 TF.jpeg
-rwxr-xr-x 0 root root 183K 1月   1  1970 banquantupian801.png
-rwxr-xr-x 0 root root 207K 1月   1  1970 bizhi112.png
-rwxr-xr-x 0 root root  383 1月   1  1970 camera_new_off_a552294.png
-rwxr-xr-x 0 root root  386 1月   1  1970 camera_new_on_4e3e250.png
-rwxr-xr-x 0 root root  20K 1月   1  1970 duoroubanqq.jpg
-rwxr-xr-x 0 root root  11K 1月   1  1970 fengjingxiaotu.jpg
-rwxr-xr-x 0 root root  13K 1月   1  1970 gaoqingdonmanxiaotuzis.jpg
-rwxr-xr-x 0 root root 3.0K 1月   1  1970 mark_b68ff2e.png
drwxr-xr-x 0 root root  73K 1月   1  1970 pic.baidu.com
-rwxr-xr-x 0 root root 9.6K 1月   1  1970 touxixiaoqinx.jpg
-rwxr-xr-x 0 root root 3.2K 1月   1  1970 uploading.gif
drwxr-xr-x 0 root root    0 1月   1  1970 wMmF9nf54b6hAAAAABJRU5ErkJggg==
-rwxr-xr-x 0 root root 7.9K 1月   1  1970 weijuchiluntu.jpg
drwxr-xr-x 0 root root  15K 1月   1  1970 www.baidu.com
-rwxr-xr-x 0 root root  12K 1月   1  1970 xiaoqingxbanq.jpg
-rwxr-xr-x 0 root root 202K 1月   1  1970 yuanjiahua10.png
```

it will treat a img link as a file, a hyper link as a directory.
