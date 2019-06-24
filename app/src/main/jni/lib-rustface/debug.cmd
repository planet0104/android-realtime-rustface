del ..\..\jniLibs\arm64-v8a\librustface.so
del ..\..\jniLibs\armeabi-v7a\librustface.so
del ..\..\jniLibs\x86\librustface.so
REM cargo build --target armv7-linux-androideabi --release
cargo build --target aarch64-linux-android
REM cargo build --target i686-linux-android --release
REM copy target\armv7-linux-androideabi\release\librustface.so ..\..\jniLibs\armeabi-v7a\librustface.so
copy target\aarch64-linux-android\debug\librustface.so ..\..\jniLibs\arm64-v8a\librustface.so
REM copy target\i686-linux-android\release\librustface.so ..\..\jniLibs\x86\librustface.so