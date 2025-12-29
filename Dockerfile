# building vic on alpine with statically linked chafa
# 
# inspired by:
# https://github.com/wong-justin/vic/issues/1#issuecomment-3658269697

# alpine has a static glib package, which is great because
# i had trouble compiling static glib with gnu libc 
# and alpine's musl is the better libc for static compilation anyways
FROM docker.io/library/rust:1.86-alpine AS builder

# chafa license: LGPL-3.0-or-later
#  glib license: LGPL-2.1-or-later
RUN apk update
RUN apk add chafa-dev glib-static

# installing musl-dev prevents this error during cargo build:
#   `note: /usr/lib/gcc/x86_64-alpine-linux-musl/14.2.0/../../../../x86_64-alpine-linux-musl/bin/ld: cannot find -ldl: No such file or directory`
# solution via: https://akisame.xyz/blog/20251104-rust-on-alpine-linux/#:~:text=without%20crt-static
# musl-dev license: MIT
RUN apk add musl-dev

# to resolve undefined references to gettext functions, which are part of gnu libc but not musl libc:
#   libintl_bindtextdomain
#   libintl_bind_textdomain_codeset
#   libintl_dgettext
#   libintl_dcgettext
#   libintl_dngettext
ENV RUSTFLAGS="-l intl"
# tell compiler about libintl
# equivalent to build.rs::println!("cargo:rustc-link-lib=static=intl")
# 
# actually not necessary:
# RUN apk add gettext-static 
#
# further reading, in case i have to mess with this again in the future:
# https://www.gnu.org/software/gettext/FAQ.html#integrating_undefined
# https://github.com/mxe/mxe/issues/2875#issuecomment-1369859887
# https://git.musl-libc.org/cgit/musl/tree/include/libintl.h
# https://buildroot.uclibc.narkive.com/gct4vzbB/build-failure-against-libintl-on-alpine#post3
# like: https://stackoverflow.com/a/21233420

# iirc chafa uses these functions to check for cpu instruction set optimizations like AVX2,
# which are present in gnu libc / libgcc but not musl libc:
#   __cpu_indicator_init_local
#   __cpu_model
ENV RUSTFLAGS="$RUSTFLAGS -lgcc"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY chafa-sys/ ./chafa-sys/
COPY src/ ./src/

RUN cargo build --release --no-default-features --features static -vv
# RUN cargo build --release --no-default-features --features dynamic -vv

RUN cargo test -vv

RUN ldd /app/target/release/vic

# FROM scratch AS export
# COPY --from=builder /app/target/release/vic /
