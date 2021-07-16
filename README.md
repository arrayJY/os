# OS

A tool x86-64 operating system.

The os is highly inspired by [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3)/[blog_os](https://github.com/phil-opp/blog_os)/[xv6](https://github.com/mit-pdos/xv6-public).

## Run

You can run the os with `qemu-system-x86_64`.

````bash
cd user
make
cd ../os
cargo install cargo-xbuild
cargo install bootimage
cargo xrun
````

It will run a user shell.  You can run several user program with it.

## Work in Progress

* [ ] Complete process management
* [ ] Process communication
* [ ] File System

## LICENSE

[MIT](https://github.com/arrayJY/os/blob/master/LICENSE) © arrayJY

