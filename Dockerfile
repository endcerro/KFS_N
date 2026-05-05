FROM rust:1.95-bullseye

RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

RUN apt update

RUN apt install -y build-essential make nasm grub-common xorriso grub-pc-bin

WORKDIR /kfs

CMD ["make"]