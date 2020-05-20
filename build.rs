use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // We need access to several of the #defines in fontconfig.h, so generate bindings for them here
    // The rest of the autogenerated bindings aren't suitable, so we use the servo-fontconfig crate
    let mut builder_main = bindgen::Builder::default();
    builder_main = builder_main.header("src/headers/fontconfig.h");

    if cfg!(feature = "Xinerama") {
	println!("cargo:rustc-link-lib=Xinerama");
	builder_main = builder_main.header("src/headers/xinerama.h");
    }

    builder_main.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_main")
        .write_to_file(out_path.join("bindings_main.rs"))
        .expect("Couldn't write bindings_main!");


    // Additionally, the x11 crate doesn't null terminate its strings for some
    //   strange reason, so a bit of extra work is required
    bindgen::Builder::default()
	.header("src/headers/xlib.h")
	.ignore_functions() // strip out unused and warning-prone functions
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings_xlib")
        .write_to_file(out_path.join("bindings_xlib.rs"))
        .expect("Couldn't write bindings_xlib!");
    
    
    // bindings depend on files in the headers directory,
    //   so make sure they are tracked for rebuild on edit
    for e in WalkDir::new("src/headers").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() {
	    let name = e.path().to_str().unwrap();
	    if name.as_bytes()[name.len()-1] != '~' as u8 { // ignore editor files
		println!("cargo:rerun-if-changed={}", e.path().display());
	    }
	}
    }

    // and link libs
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xft");


    
    
    // That's all the dmenu stuff. stest.c also need compiled
    // This is just done with standard cc tools. No rust here.
    cc::Build::new()
	.file("src/stest/stest.c")
	.flag("-pedantic") // -pedantic, opt_level, etc handled automatically
	.cargo_metadata(false) // don't link -- not a part of the rust code
	.compile("stest");
    
    Command::new("cc") // compile into standalone lib
	.args(&["-o", &format!("target/{}/stest", env::var("PROFILE").unwrap())
		, &(env::var("OUT_DIR").unwrap() + "/libstest.a")])
        .output()
	.expect("Could not link libstest.a");


}
