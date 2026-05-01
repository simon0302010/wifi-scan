fn main() {
    #[cfg(target_os = "openbsd")]
    cc::Build::new()
        .file("src/sys/openbsd/lswifi.c")
        .compile("lswifi");
}