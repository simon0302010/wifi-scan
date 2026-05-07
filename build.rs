fn main() {
    #[cfg(target_os = "openbsd")]
    {
        cc::Build::new()
            .file("src/sys/openbsd/lswifi.c")
            .compile("lswifi");

        println!("cargo:rerun-if-changed=src/sys/openbsd/lswifi.c");
    }

    #[cfg(target_os = "freebsd")]
    {
        cc::Build::new()
            .file("src/sys/freebsd/lswifi.c")
            .compile("lswifi");

        println!("cargo:rustc-link-lib=80211");
        println!("cargo:rerun-if-changed=src/sys/freebsd/lswifi.c");
    }

    #[cfg(target_os = "netbsd")]
    {
        cc::Build::new()
            .file("src/sys/netbsd/lswifi.c")
            .compile("lswifi");

        println!("cargo:rerun-if-changed=src/sys/netbsd/lswifi.c");
    }
}
