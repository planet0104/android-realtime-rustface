export ANDROID_NDK="$HOME/android-ndk-r19b"
export ANDROID_TOOLCHAIN="$HOME/ndk-standalone-16-arm"
export PATH="$PATH:$ANDROID_TOOLCHAIN/bin"
# cargo build --target arm-linux-androideabi --release
cargo build --target armv7-linux-androideabi --release
cargo build --target aarch64-linux-android --release
cargo build --target i686-linux-android --release
# cp target/arm-linux-androideabi/debug/libwallpaper.so ../h8w-android/app/src/main/jniLibs/armeabi/libwallpaper.so
# cp target/arm-linux-androideabi/release/libwallpaper.so ../h8w-android/app/src/main/jniLibs/armeabi/libwallpaper.so