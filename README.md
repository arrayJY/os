# OS

A toy x86-64 operating system.

This project is highly inspired by [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3)/[blog_os](https://github.com/phil-opp/blog_os)/[xv6](https://github.com/mit-pdos/xv6-public).

## Features

* Virtual memory 
* Dynamic memory management
* Multiple processes management
* Non-preemptive scheduling (FCFS)
* An interactive shell in user space

## Run

You can run os with `qemu-system-x86_64`.

````bash
cargo install cargo-binutils
rustup component add llvm-tools-preview
cd user
make
cd ../os
cargo install cargo-xbuild
cargo install bootimage
cargo xrun
````

It will run a interactive shell. You can run several user programs with it.

## Work in Progress

* [ ] Process concurrency
* [ ] Process communication
* [ ] File System

## LICENSE

[MIT](https://github.com/arrayJY/os/blob/master/LICENSE) © arrayJY

