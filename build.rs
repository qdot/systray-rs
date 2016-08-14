fn main() {
    println!("cargo:rustc-link-lib=dylib={}", "rust-icon");
    println!("cargo:rustc-link-search=native={}", "c:\\Users\\qdot\\code\\git-projects\\systray-rs\\resources\\");
}
