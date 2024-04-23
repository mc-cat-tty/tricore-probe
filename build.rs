

fn main() {
    #[cfg(target_os = "macos")] {
        println!("cargo:rustc-link-search=/usr/local/Cellar/libusb/1.0.27/lib");
        println!("cargo:rustc-link-lib=usb-1.0");
    }
}