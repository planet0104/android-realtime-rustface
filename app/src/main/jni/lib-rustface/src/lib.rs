// #[macro_use]
// extern crate log;
// extern crate android_logger;
mod jni_graphics;

use image::{ConvertBuffer, GrayImage, ImageBuffer, Rgba};
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jbyteArray, jdouble, jfloat, jint};
use jni::{JNIEnv, JavaVM};
use rustface::{Detector, ImageData};
use std::cell::RefCell;
// use std::time::Instant;

thread_local! {
    pub static DETECTOR: RefCell<Option<Box<Detector>>> = RefCell::new(None);
}

//JNI加载完成
#[no_mangle]
pub extern "C" fn JNI_OnLoad(_jvm: JavaVM, _reserved: *mut std::ffi::c_void) -> jint {
    // android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));
    // info!("JNI_OnLoad.");
    jni::sys::JNI_VERSION_1_6
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_create(
    env: JNIEnv,
    _class: JClass,
    file_data: jbyteArray,
) {
    match || -> Result<(), String> {
        let data = env
            .convert_byte_array(file_data)
            .map_err(|err| format!("数据读取失败: {:?}", err))?;
        let model = rustface::model::read_model(data.to_vec())
            .map_err(|err| format!("创建失败: {:?}", err))?;
        DETECTOR.with(|detector| {
            // detector.set_min_face_size(20);
            // detector.set_score_thresh(2.0);//默认:2.0, 阈值越小检测次数越多 0.95
            // detector.set_pyramid_scale_factor(0.8);
            // detector.set_slide_window_step(4, 4);
            *detector.borrow_mut() = Some(rustface::create_detector_with_model(model));
        });
        Ok(())
    }() {
        Ok(_) => (),
        Err(err) => {
            let _ = env.throw_new("java/lang/Exception", format!("创建失败:{:?}", err));
        }
    };
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_setMinFaceSize(
    env: JNIEnv,
    _class: JClass,
    min_face_size: jint,
) {
    DETECTOR.with(|detector| {
        match detector.borrow_mut().as_mut() {
            Some(detector) => detector.set_min_face_size(min_face_size as u32),
            None => {
                let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            }
        };
    });
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_setScoreThresh(
    env: JNIEnv,
    _class: JClass,
    score_thresh: jdouble,
) {
    DETECTOR.with(|detector| {
        match detector.borrow_mut().as_mut() {
            Some(detector) => detector.set_score_thresh(score_thresh),
            None => {
                let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            }
        };
    });
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_setPyramidScaleFactor(
    env: JNIEnv,
    _class: JClass,
    pyramid_scale_factor: jfloat,
) {
    DETECTOR.with(|detector| {
        match detector.borrow_mut().as_mut() {
            Some(detector) => detector.set_pyramid_scale_factor(pyramid_scale_factor),
            None => {
                let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            }
        };
    });
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_setSlideWindowStep(
    env: JNIEnv,
    _class: JClass,
    step_x: jint,
    step_y: jint,
) {
    DETECTOR.with(|detector| {
        match detector.borrow_mut().as_mut() {
            Some(detector) => detector.set_slide_window_step(step_x as u32, step_y as u32),
            None => {
                let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            }
        };
    });
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_setWindowSize(
    env: JNIEnv,
    _class: JClass,
    wnd_size: jint,
) {
    DETECTOR.with(|detector| {
        match detector.borrow_mut().as_mut() {
            Some(detector) => detector.set_window_size(wnd_size as u32),
            None => {
                let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            }
        };
    });
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_RustFace_detect(
    env: JNIEnv,
    _class: JClass,
    bitmap: JObject,
    mut scale: jfloat,
) -> jni::sys::jobject {
    let created = DETECTOR.with(|detector| match detector.borrow_mut().as_mut() {
        Some(_) => true,
        None => {
            let _ = env.throw_new("java/lang/Exception", "请先调用RustFace.create()创建");
            false
        }
    });
    if !created {
        return JObject::null().into_inner();
    }

    // info!("detect face...");
    let mje = |err| format!("识别失败 {:?}", err);
    let mut rects = None;
    // let mut t = Instant::now();
    let result = (|| -> Result<(), String> {
        jni_graphics::lock_bitmap(&env, &bitmap, |info, pixels| {
            //只支持argb888格式
            if info.format != jni_graphics::ANDROID_BITMAP_FORMAT_RGBA_8888 {
                Err("图片格式只支持RGBA_8888!".to_string())
            } else {
                // info!("lock_bitmap耗时:{}ms", t.elapsed().as_millis());
                // t = Instant::now();
                //创建
                let image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> =
                    ImageBuffer::from_raw(info.width, info.height, pixels.to_vec());
                // info!("image_from_raw耗时:{}ms", t.elapsed().as_millis());
                // t = Instant::now();
                if let Some(image) = image {
                    //灰度
                    let mut image: GrayImage = image.convert();
                    // info!("灰度耗时:{}ms", t.elapsed().as_millis());
                    // t = Instant::now();
                    //缩放
                    if scale < 1.0 {
                        let (width, height) = (
                            (info.width as f32 * scale) as u32,
                            (info.height as f32 * scale) as u32,
                        );
                        image = image::imageops::resize(
                            &image,
                            width,
                            height,
                            image::FilterType::Nearest,
                        );
                    } else {
                        scale = 1.0;
                    }
                    // info!("缩放耗时:{}ms", t.elapsed().as_millis());
                    // t = Instant::now();

                    //识别
                    let mut image_data =
                        ImageData::new(image.as_ptr(), image.width(), image.height());
                    // info!("image_data耗时:{}ms", t.elapsed().as_millis());
                    // t = Instant::now();
                    let faces = DETECTOR.with(|detector| {
                        detector
                            .borrow_mut()
                            .as_mut()
                            .unwrap()
                            .detect(&mut image_data)
                    });
                    // info!("detect耗时:{}ms", t.elapsed().as_millis());

                    //创建对象数组
                    let mut arr = vec![];
                    for face_info in faces {
                        let bbox = face_info.bbox();
                        let (x, y, width, height) = {
                            (
                                (bbox.x() as f32 / scale) as i32,
                                (bbox.y() as f32 / scale) as i32,
                                (bbox.width() as f32 / scale) as i32,
                                (bbox.height() as f32 / scale) as i32,
                            )
                        };
                        arr.push(
                            env.new_object(
                                "io/github/planet0104/rustface/FaceInfo",
                                "(IIIIDFFFF)V",
                                &[
                                    JValue::from(x),
                                    JValue::from(y),
                                    JValue::from(width),
                                    JValue::from(height),
                                    JValue::from(face_info.score()),
                                    JValue::from(x as f32 / info.width as f32),
                                    JValue::from(y as f32 / info.height as f32),
                                    JValue::from(width as f32 / info.width as f32),
                                    JValue::from(height as f32 / info.height as f32),
                                ],
                            )
                            .map_err(mje)?,
                        );
                    }

                    let face_info_array = env
                        .new_object_array(
                            arr.len() as i32,
                            "io/github/planet0104/rustface/FaceInfo",
                            JObject::null(),
                        )
                        .map_err(mje)?;
                    for (i, r) in arr.iter().enumerate() {
                        env.set_object_array_element(face_info_array, i as i32, JObject::from(*r))
                            .map_err(mje)?;
                    }

                    rects = Some(face_info_array);
                    Ok(())
                } else {
                    Err("图片读取失败，请查格式!".to_string())
                }
            }
        })?;
        Ok(())
    })();

    if result.is_err() {
        let err = result.err();
        // error!("{:?}", &err);
        let _ = env.throw_new("java/lang/Exception", format!("{:?}", err));
        JObject::null().into_inner()
    } else {
        rects.unwrap()
    }
}