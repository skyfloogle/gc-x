use std::env;

fn main() {
    //println!("cargo:rerun-if-changed=wrapper.h"); // not really sure what file to do this for
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    if target_env == "gnu" {
        println!("cargo:rustc-link-lib=stdc++");
    }
    cc::Build::new().file("ViGEmClient/src/ViGEmClient.cpp").include("ViGEmClient/include").compile("VigEmClient");
}
