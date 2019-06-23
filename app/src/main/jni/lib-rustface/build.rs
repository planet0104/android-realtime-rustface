fn main() {
    //设置windows环境变量
    // AR_armv7-linux-androideabi = C:\android-ndk-r20\toolchains\llvm\prebuilt\windows-x86_64\bin\arm-linux-androideabi-ar
    // CC_armv7-linux-androideabi = C:\android-ndk-r20\toolchains\llvm\prebuilt\windows-x86_64\bin\armv7a-linux-androideabi17-clang.cmd

    // AR_aarch64-linux-android = C:/android-ndk-r20/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android-ar
    // CC_aarch64-linux-android = C:/android-ndk-r20/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android21-clang.cmd

    // AR_i686-linux-android = C:/android-ndk-r20/toolchains/llvm/prebuilt/windows-x86_64/bin/i686-linux-android-ar
    // CC_i686-linux-android = C:/android-ndk-r20/toolchains/llvm/prebuilt/windows-x86_64/bin/i686-linux-android17-clang.cmd

    cc::Build::new()
        .file("pico/rnt/picornt.c")
        .include("pico/rnt/")
        .compile("pico");
}
