fn main() {
    //println!("cargo:rerun-if-changed=wrapper.h"); // not really sure what file to do this for
    cc::Build::new().file("ViGEmClient/src/ViGEmClient.cpp").include("ViGEmClient/include").compile("VigEmClient");
}
