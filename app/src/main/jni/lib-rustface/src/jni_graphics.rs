#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use jni;
use jni::objects::{JObject, JValue};
use jni::sys::{jobject, JNIEnv};
use std::os::raw::{c_int, c_uint, c_void};
//Bitmap作为

pub const ANDROID_BITMAP_FORMAT_NONE: i32 = 0;
pub const ANDROID_BITMAP_FORMAT_RGBA_8888: i32 = 1;
pub const ANDROID_BITMAP_FORMAT_RGB_565: i32 = 4;
pub const ANDROID_BITMAP_FORMAT_RGBA_4444: i32 = 7;
pub const ANDROID_BITMAP_FORMAT_A_8: i32 = 8;

pub fn get_format_name(format: i32) -> String {
    String::from(match format {
        ANDROID_BITMAP_FORMAT_NONE => "None",
        ANDROID_BITMAP_FORMAT_RGBA_8888 => "RGBA_8888",
        ANDROID_BITMAP_FORMAT_RGB_565 => "RGB_565",
        ANDROID_BITMAP_FORMAT_RGBA_4444 => "RGBA_4444",
        ANDROID_BITMAP_FORMAT_A_8 => "FORMAT_A_8",
        _ => "未知格式",
    })
}

#[repr(C)]
#[derive(Debug)]
pub struct AndroidBitmapInfo {
    pub width: c_uint,
    pub height: c_uint,
    pub stride: c_uint,
    pub format: c_int,
    pub flags: c_uint, // 0 for now
}

#[link(name = "jnigraphics", kind="dylib")]
#[allow(non_snake_case)]
extern "C" {
    ///给定一个java位图对象，为它填写AndroidBitmap结构。如果调用失败，将忽略info参数
    pub fn AndroidBitmap_getInfo(
        env: *mut JNIEnv,
        jbitmap: jobject,
        info: *mut AndroidBitmapInfo,
    ) -> c_int;

    ///给定一个java位图对象，尝试锁定像素地址。 锁定将确保像素的内存在unlockPixels调用之前不会移动，并确保如果像素先前已被清除，则它们将被恢复。
    ///如果此调用成功，则必须通过调用AndroidBitmap_unlockPixels来平衡，之后不再使用像素的地址。
    ///如果成功，* addrPtr将被设置为像素地址。 如果调用失败，将忽略addrPtr。
    pub fn AndroidBitmap_lockPixels(
        env: *mut JNIEnv,
        jbitmap: jobject,
        addrPtr: *mut *mut c_void,
    ) -> c_int;

    ///调用此方法可以平衡对AndroidBitmap_lockPixels的成功调用
    pub fn AndroidBitmap_unlockPixels(env: *mut JNIEnv, jbitmap: jobject) -> c_int;
}

pub fn unlock_bitmap(env: &jni::JNIEnv, bitmap: jobject) {
    let _ret = unsafe { AndroidBitmap_unlockPixels(env.get_native_interface(), bitmap) };
    // trace!("AndroidBitmap_unlockPixels:{}", ret);
}

//锁定bitmap
pub fn lock_bitmap<'a, F>(env: &jni::JNIEnv, bitmap: &JObject, mut render: F) -> Result<(), String>
where
    F: FnMut(&AndroidBitmapInfo, &mut [u8]) -> Result<(), String>,
{
    let mut info = AndroidBitmapInfo {
        width: 0,
        height: 0,
        stride: 0,
        format: 0,
        flags: 0,
    };
    let jenv = env.get_native_interface();
    let jbitmap = bitmap.into_inner();
    let ret = unsafe { AndroidBitmap_getInfo(jenv, jbitmap, &mut info) };
    if ret < 0 {
        return Result::Err(format!("AndroidBitmap_getInfo调用失败! {}", ret));
    }

    let bpp = match info.format {
        ANDROID_BITMAP_FORMAT_NONE => 0,
        ANDROID_BITMAP_FORMAT_RGBA_8888 => 4,
        ANDROID_BITMAP_FORMAT_RGB_565 => 2,
        ANDROID_BITMAP_FORMAT_RGBA_4444 => 2,
        ANDROID_BITMAP_FORMAT_A_8 => 1,
        _ => 0,
    };

    if bpp == 0 {
        return Result::Err(format!("不支持的位图格式: {}", info.format));
    }

    let mut pixels = 0 as *mut c_void;
    let ret = unsafe { AndroidBitmap_lockPixels(jenv, jbitmap, &mut pixels) };
    if ret < 0 {
        return Result::Err(format!("AndroidBitmap_lockPixels! {}", ret));
    }
    let pixels = unsafe {
        ::std::slice::from_raw_parts_mut(pixels as *mut u8, (info.stride * info.height) as usize)
    };

    render(&info, pixels)?;
    if unsafe { AndroidBitmap_unlockPixels(jenv, jbitmap) } != 0 {
        // error!("ANativeWindow_unlockAndPost 调用失败!");
    }
    Ok(())
}

//创建bitmap对象
pub fn create_java_bitmap_argb888<'a>(
    env: &'a jni::JNIEnv,
    width: i32,
    height: i32,
) -> Result<JObject<'a>, jni::errors::Error> {
    let config = env.call_static_method(
        "android/graphics/Bitmap$Config",
        "nativeToConfig",
        "(I)Landroid/graphics/Bitmap$Config;",
        &[JValue::from(5)],
    )?;
    let bitmap = env.call_static_method(
        "android/graphics/Bitmap",
        "createBitmap",
        "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;",
        &[JValue::from(width), JValue::from(height), config],
    )?;
    Ok(bitmap.l()?)
}

/// 创建bitmap对象
pub fn create_java_bitmap_form_colors<'a>(
    env: &'a jni::JNIEnv,
    colors: jni::sys::jintArray,
    offset: i32,
    stride: i32,
    width: i32,
    height: i32,
) -> Result<JObject<'a>, jni::errors::Error> {
    let config = env.call_static_method(
        "android/graphics/Bitmap$Config",
        "nativeToConfig",
        "(I)Landroid/graphics/Bitmap$Config;",
        &[JValue::from(5)],
    )?;
    let bitmap = env.call_static_method(
        "android/graphics/Bitmap",
        "createBitmap",
        "([IIIIILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;",
        &[
            JValue::from(JObject::from(colors)),
            JValue::from(offset),
            JValue::from(stride),
            JValue::from(width),
            JValue::from(height),
            config,
        ],
    )?;
    Ok(bitmap.l()?)
}

/// 创建Randroid.graphics.Rect
pub fn new_rect<'a>(
    env: &'a jni::JNIEnv,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) -> Result<JObject<'a>, jni::errors::Error> {
    env.new_object(
        "android/graphics/Rect",
        "(IIII)V",
        &[
            JValue::from(left),
            JValue::from(top),
            JValue::from(right),
            JValue::from(bottom),
        ],
    )
}

/// 创建Randroid.graphics.RectF
pub fn new_rectf<'a>(
    env: &'a jni::JNIEnv,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) -> Result<JObject<'a>, jni::errors::Error> {
    env.new_object(
        "android/graphics/RectF",
        "(FFFF)V",
        &[
            JValue::from(left),
            JValue::from(top),
            JValue::from(right),
            JValue::from(bottom),
        ],
    )
}
