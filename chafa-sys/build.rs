use pkg_config;

// important design decisions for a Rust bindings crate:
// https://kornel.ski/rust-sys-crate
// https://internals.rust-lang.org/t/statically-linked-c-c-libraries/17175
//
// see also:
// https://patchwork.kernel.org/project/qemu-devel/patch/20210907121943.3498701-14-marcandre.lureau@redhat.com/#24435093
// https://users.rust-lang.org/t/help-creating-a-sys-crate/12624/3
//
// - Cargo.toml [features] dynamic = [] and static = []
// - don't set a default, because it's too hard to unset defaults, esp since this crate will
// probably be a dependency of a dependency of a dependency
// - also support env vars so top-level project can choose linking type without needing to touch a Cargo.toml
// - do i run bindgen once and commit it, or run bindgen on every build? i say once,
// because chafa ABI is relatively stable
// and bindgen has a lot of dependencies, so it would be nice to make it optional
// for consumers of this crate, by having the output already committed to the repo
// and i don't think there are many platform-dependent chafa bindings, but i should check
//
// tbd:
// - for static builds, do i vendor chafa/glib and build from scratch? if so, do i use cc or some other build system?
// (i had trouble building glib; i only know how to link static in a dockerfile with prebuilt glib)
// (but would be nice in future to build from source, esp on multiple platforms)
// - how to handle upgrades / version changes? for incompatible versions, making a new crate for each
// version might be good; or manually versioning functions;
// but chafa seems stable, so i don't think i need to worry about versions
// - does my app logic work on multiple platforms? e.g. CARGO_CFG_TARGET_POINTER_WIDTH
// - crates.io, and if so, co-owner?
// - verify correctness of rust bindings against potentially platform-dependent C headers: https://crates.io/crates/ctest

fn main() {
    let is_dynamic = cfg!(feature = "link-dynamic");
    let is_static = cfg!(feature = "link-static");
    // let is_static_build = cfg!(feature = "build-and-link-static-from-source");

    // make feature flags mutually exclusive
    match (is_dynamic, is_static) {
        (true, false) => link_dynamic(),
        (false, true) => link_static(),
        (false, false) => {
            panic!("chafa-sys build requires either `--features link-dynamic` or `--features link-static`.");
        }
        (true, true) => {
            panic!("chafa-sys build must choose either dynamic or static linking, not both.");
        }
    }
}

fn link_static() {
    // this rustified pkg_config command is like:
    //
    //   pkg-config --libs --cflags --static chafa
    //
    // which returns on alpine container:
    //   -I/usr/include/chafa -I/usr/lib/chafa/include -I/usr/include/glib-2.0 -I/usr/lib/glib-2.0/include -lchafa -lm -lglib-2.0 -lintl -latomic -lm -pthread -lpcre2-8
    //
    // compare to no --static:
    //
    //   -I/usr/include/chafa -I/usr/lib/chafa/include -I/usr/include/glib-2.0 -I/usr/lib/glib-2.0/include -lchafa -lglib-2.0 -lintl
    //
    // and compare to my arch machine, no --static:
    //   -I/usr/include/chafa -I/usr/lib/chafa/include -I/usr/include/glib-2.0 -I/usr/lib/glib-2.0/include -I/usr/include/sysprof-6 -pthread -lchafa -lglib-2.0
    //
    // note that pkg_config --static could return with no errors and compile successfully,
    // but produce a dynamically linked executable.
    // this is unfortunate.
    // you should verify that the resulting exectuable is statically linked by running `ldd`.
    //
    // i also recommend knowing exactly where every dependency is located
    // in your build environment, and whether they are static .a libraries or dynamic .so libraries.
    // this could be enforced by abandoning pkg_config in favor of something like:
    // ```
    // match std::env::var("CHAFA_SYS_LIBRARY_PATH") {
    //     Ok(library_path) => {
    //         println!("cargo:rustc-link-search={}", library_path); // like gcc -L
    //         println!("cargo:rustc-link-lib=static=chafa"); // like gcc -l
    //         println!("cargo:rustc-link-lib=static=glib-2.0");
    //     }
    //     Err(std::env::VarError::NotPresent) => {}
    //     Err(e) => {}
    // };
    // ```
    //
    pkg_config::Config::new()
        .statik(true)
        .probe("chafa")
        .expect("pkg-config could not find chafa");
}

// fn _build_and_link_static_from_source() {
//     todo!("build chafa+glib from source in this build.rs");
// }

fn link_dynamic() {
    pkg_config::probe_library("chafa").expect("pkg-config could not find chafa");
}

#[cfg(feature = "bindgen")]
fn _create_bindings() {
    // TODO: replace this with a cli command
    // this function was run once manually, and the output bindings.rs was copied to src/

    let library = pkg_config::probe_library("chafa").expect("pkg-config could not find chafa");
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
        .expect("bindgen was unable to generate chafa bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("unable to write chafa bindings to file");
}
