use std::env;

fn main() {
    //println!("cargo:rerun-if-changed=wrapper.h"); // not really sure what file to do this for
    let mut build = cc::Build::new();
    build.file("ViGEmClient/src/ViGEmClient.cpp").include("ViGEmClient/include");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    if target_env == "gnu" {
        build.flag("-w");
        println!("cargo:rustc-link-lib=stdc++");
    }
    build.compile("VigEmClient");
}
