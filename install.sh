#! /bin/bash

function rust ()
{
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
}
function deps ()
{
    sudo apt install -y build-essential make curl nasm grub-common xorriso grub-pc-bin qemu-system-x86
}


read -p "Do you wish to install the dependencies (sudo requiered) ? Y/N" -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    deps
fi
read -p "Do you wish to install the Rust toolchain ? Y/N" -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    rust
    source ~/.bashrc
    rustup toolchain install nightly --allow-downgrade
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
fi
