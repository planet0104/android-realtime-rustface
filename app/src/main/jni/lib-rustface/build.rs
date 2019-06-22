fn main() {
    cc::Build::new()
        .file("pico/rnt/picornt.c")
        .include("pico/rnt/")
        .compile("pico");
}
