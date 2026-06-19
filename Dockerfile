FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive \
    HOME=/root \
    CARGO_HOME=/root/.cargo \
    RUSTUP_HOME=/root/.rustup \
    PATH=/root/.cargo/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    nasm \
    grub-pc-bin \
    grub-common \
    xorriso \
    mtools \
    dosfstools \
    qemu-system-x86_64 \
    qemu-system-arm \
    python3 \
    python3-pip \
    curl \
    ca-certificates \
    git \
    gcc-multilib \
    g++-multilib \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    wget \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --profile minimal \
 && rustup component add rust-src rustfmt clippy \
 && rustup set auto-self-update disable

WORKDIR /workspace

COPY . /workspace

# Should fix the nightly issue 
COPY hello /workspace/hello
COPY compositor /workspace/compositor

CMD ["bash"]
