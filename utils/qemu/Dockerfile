## SPDX-License-Identifier: MIT OR Apache-2.0
##
## Copyright (c) 2017-2020 Andre Richter <andre.o.richter@gmail.com>
## Copyright (c) 2019-2020 Nao Taco <naotaco@gmail.com>
FROM ubuntu:20.04

ARG VCS_REF

LABEL org.label-schema.vcs-ref=$VCS_REF \
    org.label-schema.vcs-url="https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials"

LABEL maintainer="The resources team <resources@teams.rust-embedded.org>, Andre Richter <andre.o.richter@gmail.com>"

# Ruby gems
COPY Gemfile .

RUN set -ex;                                        \
    tempPkgs='                                      \
        automake                                    \
        bison                                       \
        build-essential                             \
        flex                                        \
        git                                         \
        libtool                                     \
        pkg-config                                  \
        wget                                        \
    ';                                              \
    apt-get update;                                 \
    apt-get install -q -y --no-install-recommends   \
        $tempPkgs                                   \
        # persistent packages
        ca-certificates                             \
        gdb-multiarch                               \
        libpixman-1-dev                             \
        libglib2.0-dev                              \
        libusb-1.0.0-dev                            \
        locales                                     \
        python3                                     \
        python-setuptools                                     \
		ninja-build \
        python3-pip\
        ruby                                        \
        ruby-dev                                    \
    ;                                               \
	pip3 install setuptools;						\
    # Ruby dependencies
    gem install bundler;                            \
    bundle install --retry 3 --without development; \
    # QEMU
    git clone git://git.qemu.org/qemu.git;                     \
    cd qemu;                                                   \
    git checkout tags/v5.2.0-rc0;                                  \
    ./configure --target-list=aarch64-softmmu --enable-modules \
        --enable-tcg-interpreter --enable-debug-tcg            \
        --python=/usr/bin/python3;                             \
    make -j8;                                                  \
    make install;                                              \
    cd ..;                                                     \
    rm -rf qemu;                                               \
    # Openocd
    git clone --depth 1 https://git.code.sf.net/p/openocd/code openocd; \
    cd openocd;                                                         \
    ./bootstrap;                                                        \
    ./configure --enable-ftdi;                                          \
    make -j8;                                                           \
    make install;                                                       \
    # GDB
    wget -P ~ git.io/.gdbinit;                      \
    # GCC AArch64 tools
    wget https://developer.arm.com/-/media/Files/downloads/gnu-a/9.2-2019.12/binrel/gcc-arm-9.2-2019.12-x86_64-aarch64-none-elf.tar.xz; \
    tar -xf gcc-arm-9*;                                                                                                                 \
    cp gcc-arm-9*/bin/aarch64-none-elf-objdump gcc-arm-9*/bin/aarch64-none-elf-nm /usr/local/bin/;                                      \
    rm -rf gcc-arm-9*;                                                                                                                  \
    # Cleanup
    apt-get purge -y --auto-remove $tempPkgs;       \
    apt-get autoremove -q -y;                       \
    apt-get clean -q -y;                            \
    rm -rf /var/lib/apt/lists/*

# Locales
RUN locale-gen en_US.UTF-8

ENV LANG=en_US.UTF-8  \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8 \
    RUBYOPT=-W0

# Openocd
COPY rpi3.cfg /openocd/
COPY rpi4.cfg /openocd/

# GDB
COPY auto /root/.gdbinit.d/auto
