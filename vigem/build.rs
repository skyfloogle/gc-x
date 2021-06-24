use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    cc::Build::new().file("ViGEmClient/src/ViGEmClient.cpp").include("ViGEmClient/include").compile("VigEmClient");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-IVigEmClient/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("^vigem_.+")
        .allowlist_type("^(VIGEM_|EVT_|XUSB).+")
        .blocklist_function("vigem_target_ds4_update_ex") // the struct gets marked as packed which is bad
        .blocklist_type(".?DS4_REPORT_EX.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings");
}
