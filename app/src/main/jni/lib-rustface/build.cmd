cargo build --target armv7-linux-androideabi --release
cargo build --target aarch64-linux-android --release
cargo build --target i686-linux-android --release
copy target\armv7-linux-androideabi\release\librustface.so ..\..\jniLibs\armeabi-v7a\librustface.so
copy target\aarch64-linux-android\release\librustface.so ..\..\jniLibs\arm64-v8a\librustface.so
copy target\i686-linux-android\release\librustface.so ..\..\jniLibs\x86\librustface.so