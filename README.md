# Cloudflare DDNS CLI Client

## Getting start

### Cross Compile to `mipsel-unknown-linux-musl` for openwrt


1. We need to know the triple for the target device, e.g. Asus RT-N56U router.
According to the [Techdata](https://openwrt.org/toh/hwdata/asus/asus_rt-n56u_a1),
the target should be `mipsel-unknown-linux-musl`.
2. Download the [openwrt SDK](https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz),
which can be found on [this page](https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/)
such that we can cross compile rust to openwrt. We have to set the path to the
toolchains folder as an environment var `$STAGING_DIR`, and set the bin folder under the
toolchains into `$PATH`.

```fish
cd /tmp
wget https://downloads.lede-project.org/releases/17.01.4/targets/ramips/rt3883/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz
tar xf lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64.tar.xz
cd lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64/staging_dir

" Set env STAGING_DIR
set -x STAGING_DIR /tmp/lede-sdk-17.01.4-ramips-rt3883_gcc-5.4.0_musl-1.1.16.Linux-x86_64/staging_dir/toolchains
set -x PATH $STAGING_DIR/bin $PATH
```

3. As the project depends openssl, we also need to cross compile the openssl lib for openwrt in order to link the compiled lib which is required by `rust-openssl` as `$OPENSSL_LIB_DIR` and `$OPENSSL_INCLUDE_DIR`.

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
env CC=mipsel-openwrt-linux-gcc LD=mipsel-openwrt-linux-ld cargo build --target=mipsel-unknown-linux-musl
```

## Usages
Type `--help` for more info


```sh
ddns --email=abc@example.com --key=abc --zone_id=zone123 list <record_name>
ddns --email=abc@example.com --key=abc --zone_id=zone123 get [record_id]
ddns --email=abc@example.com --key=abc --zone_id=zone123 update [record_name]
```

## Tested device

+ Asus RT-N56U with OpenWrt v17.01.4
