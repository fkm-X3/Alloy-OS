FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    HOME=/root \
    CARGO_HOME=/root/.cargo \
    RUSTUP_HOME=/root/.rustup \
    PATH=/root/.cargo/bin:/root/.local/i686-elf/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    nasm \
    grub-pc-bin \
    grub-common \
    xorriso \
    mtools \
    qemu-system-x86 \
    python3 \
    python3-pip \
    curl \
    ca-certificates \
    git \
    bison \
    flex \
    libgmp3-dev \
    libmpc-dev \
    libmpfr-dev \
    texinfo \
    wget \
    tar \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --profile minimal \
 && rustup component add rust-src rustfmt clippy \
 && rustup set auto-self-update disable

WORKDIR /workspace

COPY build-toolchain.sh /workspace/build-toolchain.sh
RUN chmod +x /workspace/build-toolchain.sh \
 && sed -i 's/\r$//' /workspace/build-toolchain.sh \
 && /workspace/build-toolchain.sh \
 && rm -rf ~/toolchain-build

COPY . /workspace

CMD ["bash"]
