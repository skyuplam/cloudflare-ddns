# cloudflare_ddns
Cloudflare DDNS

## Cross Compile to `mipsel-unknown-linux-musl` for openwrt


1. We need to know the triple for the target device, e.g. Asus RT-N56U router. According to the [Techdata](https://openwrt.org/toh/hwdata/asus/asus_rt-n56u_a1), the target should be `mipsel-unknown-linux-musl`.
2. Download the [openwrt SDK](https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz), which can be found on [this page](https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/) such that we can cross compile rust to openwrt.


```fish
cd /tmp
wget https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz
tar xf lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz
cd lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64/staging_dir

" Set env STAGING_DIR
set -x STAGING_DIR /tmp/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64/staging_dir/toolchains
set -x PATH $STAGING_DIR/bin $PATH
```

3. As the project use openssl, we also need to cross compile the openssl lib for openwrt.

```fish
cd /tmp
" Download the source
wget https://www.openssl.org/source/openssl-1.0.1t.tar.gz
tar xzf openssl-1.0.1t.tar.gz
cd openssl-1.0.1t

" Compile
env MACHINE=mipsel ARCH=musl CC=mipsel-openwrt-linux-gcc ./config shared
env MACHINE=mipsel ARCH=musl CC=mipsel-openwrt-linux-gcc make
" Export as env
set -x OPENSSL_LIB_DIR /tmp/openssl-1.0.1t/
set -x OPENSSL_INCLUDE_DIR /tmp/openssl-1.0.1t/include
```
4. Compile the program

```fish
env CC=mipsel-openwrt-linux-gcc LD=mipsel-openwrt-linux-ld LIBS=-ldl cargo build --target=mipsel-unknown-linux-musl
```
