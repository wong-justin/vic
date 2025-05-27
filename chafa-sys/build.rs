use bindgen;
use std::env;
use std::path::PathBuf;
// https://docs.rs/cc/latest/cc/#compile-time-requirements
use cc;

// a good summary of goals for a Rust bindings library:
// https://internals.rust-lang.org/t/statically-linked-c-c-libraries/17175
// other references:
// https://patchwork.kernel.org/project/qemu-devel/patch/20210907121943.3498701-14-marcandre.lureau@redhat.com/#24435093
// https://users.rust-lang.org/t/help-creating-a-sys-crate/12624/3

// use bindgen;
// good: bindgen automates binding creation from C headers
// bad: bindgen requires libclang, which I guess is supposed to come with llvm as well
// (https://rust-lang.github.io/rust-bindgen/requirements.html)

// chafa must be installed in apt install style so that there's include/ files
// since the include/ files are necessary for C tools for binding
// needs glib still
// chafa build step also complained about libdevtool2 or something like that, but i ignored by using the flag --without-tools

// helpful reference about how linking object files and libraries works in general:
// https://stackoverflow.com/a/29728671

// to compile from C src:
//
// cc-rs crate, which handles compiles C on multiple platforms into static archives
// 75KB? https://crates.io/crates/cc
// more established
//
// or
//
// cargo-zigbuild, which uses zig as a great drop-in replacement for a C compiler
// 35KB? https://crates.io/crates/cargo-zigbuild/0.17.3
// new-age but promising

fn main() {
    // psuedocode goal for cargo install/build:
    //
    // match cfg feature/env::CHAFA_RUST_LINKING_TYPE {
    //     build_dynamic => build_dynamic().expect("chafa and glib could not successfully be linked dynamically")
    //     build_static => build_static().expect("chafa and glib could not be built from source")
    //     _ => build_dynamic().if_fails_then(build_static)
    //     or maybe _ => throw("must explicitly choose whether to build dynamic or static")
    // }

    // build_static();
    build_dynamic();
}

fn build_static() {
    // TODO: fix static build. see below:
    //
    // link chafa (and glib) statically from vendored source

    // println!("cargo:rustc-link-search=./vendor/glib/"); // didn't help me
    // env::set_var("LD_LIBRARY_PATH", "./vendor/glib"); // didn't help me
    // env::set_var("PKG_CONFIG_ALLOW_SYSTEM_CFLAGS", "1"); // didn't help me

    // libglib2.0-dev
    // "this package is needed co compile programs against libglib2.0-0,
    // as it onlly includes the header files and static libraries needed for compiling"
    //
    // more learning:
    // https://kornel.ski/rust-sys-crate#use-cc
    //
    // sometimes just listing all the .c files is enough to build with `cc`
    // example: https://github.com/kornelski/mozjpeg-sys/blob/384688f9c23e94ddeb353d414d45ede69768ec08/src/build.rs
    // but probs not with glib, because it's more complicated
    //
    // find . | grep '\.c$' | grep -v -e '/tests/ ' -e 'chafa/tools' -e 'glib/.gitlab-ci' -e 'glib/fuzzing'

    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // let source_c_files = std::fs::read_dir("vendor/glib")
    //     .unwrap()
    //     .filter_map(|entry| {
    //         let path = &entry.expect("couldn't unwrap entry inside vendor/").path();
    //         let pathstr = path.to_str().expect("couldn't convert path to string");
    //         match (
    //             path.is_file() &
    //             path.ends_with(".c") &
    //             !pathstr.contains("tests/") &
    //             !pathstr.contains("tools/") &
    //             !pathstr.contains("fuzzing/")
    //         ) {
    //             true => Some(String::from(pathstr)),
    //             false => None
    //         }
    //     });

    // println!("cargo:rustc-env=AR=ar");
    // println!("cargo:rustc-env=CC=cmd.exe /C zig cc");
    // println!("cargo:rustc-env=CC_ENABLE_DEBUG_OUTPUT=1");

    // cc::Build::new()
    //     // .include("vendor/")
    //     .files(source_c_files)
    //     // .file("vendor/glib/glib/glib.h")
    //     // .file("vendor/chafa/chafa/chafa.h")
    //     // .file("./vendor/glib/glib/glibconfig.h.in")
    //     .compile("chafa");
    // // maybe try manually compiling into objects,
    // // then manually linking/stuffing into archive?

    // println!("{}", out_dir);

    // seems to find all C files,
    // but can't find libchafa.a archive in the target outdir
    // supposed to assemble at get_out_dir()
    // https://github.com/rust-lang/cc-rs/blob/e6e29d873604a14912fa130f4c568bca5062d18a/src/lib.rs#L1277
    //
    // aside: a library archive is a bunch of objects in a bucket i guess
    // cc makes a library archive rather than command-line listing all object files
    // because the command-line may have character limits.
    //
    // also: GNU convention is to name the archive "lib{actualname}.a"

    // println!("cargo:rustc-link-search=./vendor/glib/"); // didn't help me
    // println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());

    // let bindings = bindgen::Builder::default()
    //     .header("wrapper.h")
    //     .clang_arg("-I./vendor/chafa/chafa/")
    //     .clang_arg("-I./vendor/glib/glib/")
    //     .generate()
    //     .expect("Unable to generate bindings");

    // let out_dir = env::var("OUT_DIR").unwrap();
    // bindings
    //     .write_to_file(PathBuf::from(out_dir).join("bindings.rs"))
    //     .expect("Couldn't write bindings")
}

fn build_dynamic() {
    // expecting glib and chafa already installed on system,
    // and pkg-config should find the libraries

    // STEPS TO BUILD A RUST LIB WITH C BINDINGS / WRAP A C LIBRARY
    //
    // 0) the C library needs to be installed, or built and installed.
    //    apt install chafa wasn't available on my machine, so i had to clone and build chafa.
    //    chafa build instructions were a bit confusing for me, as most C libraries are for me.
    //    i needed to install glib, libdevtools, and a couple other things.
    //    although the author said: "You could build it --without-tools,
    //    then you don't need any loaders. The deps will then be glib-2.0 and freetype."
    //    finally, chafa/autogen.sh, then make, then make install, worked
    //    note: my chafa version is 1.14.0, and my glib version is 2.56.0
    //
    // 1) a header file must exist in this rust project directory root.
    //    probs best simply call it wrapper.h, with contents:
    //    #include <chafa.h>
    //
    //    (https://rust-lang.github.io/rust-bindgen/tutorial-2.html)
    //
    // 2) in this build script, we need to tell the build tools where to find the libraries.
    //    there's a few ways to declare these library paths:
    //
    //    a) write the include/ path args by hand, eg:
    //       .clang_arg("-I/usr/local/include/chafa")
    //       ...
    //
    //    b) use the tool pkg-config to find these paths automatically.
    //       on the command-line, that looks like:
    //       pkg-config --cflags chafa
    //       which outputs the string:
    //       -I/usr/local/include/chafa -I/usr/local/lib...
    //
    //    c) I couldn't get this to work, but using either
    //       println!("cargo:rustc-link-search=/path/to/lib")
    //       or
    //       println!("cargo:rustc-link-lib=chafa")
    //       is supposed tell cargo about library locations
    //
    //    (https://rust-lang.github.io/rust-bindgen/tutorial-3.html)
    //
    // 3) create src/lib.rs with contents:
    //    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    //
    //    also adding some #![allow(...)] directives to surpress warnings about C-syntax.
    //
    //    then cargo build should work. bindgen will create a bindings.rs file somewhere deep in target/
    //
    // 4) verify that the layout, size, and alignment of FFI structs matches
    //    what bindgen thinks they should be, by running cargo test
    //
    //    (https://rust-lang.github.io/rust-bindgen/tutorial-4.html)
    //
    // 4.5) troubleshoot why some functions are an undefined reference?
    //      (error while loading shared libraries: no such file or directory libchafa.so.0)
    //      i guess the linking tools weren't finding /usr/local/lib/libchafa.so.0
    //
    //      EDIT - my solution was running the simple command `ldconfig`,
    //      which updates the cache for shared libraries.
    //      apparently it was not updated since i installed chafa.
    //
    // 5) create safe function wrappers around extern functions
    //    using Rust idioms and exposing a safe, high-level interface
    //
    //    (https://doc.rust-lang.org/nomicon/ffi.html)
    //    (https://medium.com/dwelo-r-d/using-c-libraries-in-rust-13961948c72a)
    //
    // 6) consider whitelisting only necessary functions 
    //    (or blacklisting unused functions)
    //    to reduce the size of the bindings file
    //
    //    (https://rust-lang.github.io/rust-bindgen/allowlisting.html)

    // didn't help:
    // println!("cargo:rustc-flags=-l chafa -L /usr/local/lib/chafa");
    // println!("cargo:rustc-link-lib=glib");
    // println!("cargo:rustc-link-search=/usr/local/lib");
    // env::set_var("LD_LIBRARY_PATH", "/usr/local/lib");
    // env::set_var("PKG_CONFIG_ALLOW_SYSTEM_CFLAGS", "1");

    // consider replacing pkg-config:
    // https://users.rust-lang.org/t/how-to-make-a-sys-crate/13767/8
    //
    // "For linking I went with simply outputting cargo:rustc-link-lib= from build.rs,
    // with ability to set the library path with an environment variable (no pkg-config)
    // and with default path on Windows set to where the library installer
    // installs the libraries by default."
    //
    let library = pkg_config::probe_library("chafa").expect("Unable to probe chafa library");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(
            library
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        )
        // aka:
        // .clang_arg("-I/usr/local/include/chafa")
        // .clang_arg("-I/usr/local/lib/chafa/include")
        // .clang_arg("-I/usr/include/glib-2.0")
        // .clang_arg("-I/usr/lib/x86_64-linux-gnu/glib-2.0/include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings")
}
