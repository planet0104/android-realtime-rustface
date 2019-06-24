#[macro_use]
extern crate log;
extern crate android_logger;

#[macro_use]
extern crate lazy_static;

mod jni_graphics;
mod pico;

use image::{ConvertBuffer, GrayImage, RgbImage, ImageBuffer, Luma, Rgb, Rgba};
use jni::objects::{JClass, JString, JObject, JValue};
use jni::sys::{jbyteArray, jobject, jdouble, jfloat, jint};
use jni::{JNIEnv, JavaVM};
use rustface::{Detector, ImageData};
use std::cell::RefCell;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

thread_local! {
    pub static DETECTOR: RefCell<Option<Box<Detector>>> = RefCell::new(None);
}

lazy_static! {
    static ref PICOS: Arc<Mutex<HashMap<String, (pico::Pico, Option<(Vec<u8>, u32, u32)>)>>> = Arc::new(Mutex::new(HashMap::new()));
}

//JNI加载完成
#[no_mangle]
pub extern "C" fn JNI_OnLoad(_jvm: JavaVM, _reserved: *mut std::ffi::c_void) -> jint {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));
    info!("JNI_OnLoad.");
    jni::sys::JNI_VERSION_1_6
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_remove(
    env: JNIEnv,
    _class: JClass,
    key: JString,
){
    let mje = |err| format!("Pico创建失败: {:?}", err);
    match || -> Result<(), String> {
        let key:String = env.get_string(key).map_err(mje)?.into();
        let mut picos = PICOS.lock().map_err(|err| format!("Pico删除失败: {:?}", err))?;
        if picos.contains_key(&key){
            picos.remove(&key);
        }
        Ok(())
    }() {
        Ok(_) => (),
        Err(err) => {
            let _ = env.throw_new("java/lang/Exception", format!("Pico删除失败:{:?}", err));
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_create(
    env: JNIEnv,
    _class: JClass,
    file_data: jbyteArray,
) -> jobject {
    let mje = |err| format!("Pico创建失败: {:?}", err);
    match || -> Result<JObject, String> {
        let data = env
            .convert_byte_array(file_data)
            .map_err(mje)?;

        let digest = md5::compute(&data);
        
        let key = format!("{:x}", digest);
        let jkey = env.new_string(&key).map_err(mje)?;
        let mut picos = PICOS.lock().map_err(|err| format!("Pico创建失败: {:?}", err))?;
        if !picos.contains_key(&key){
            let pico = pico::Pico::new(data);
            picos.insert(key, (pico, None));
        }
        Ok(JObject::from(jkey))
    }() {
        Ok(obj) => obj.into_inner(),
        Err(err) => {
            let _ = env.throw_new("java/lang/Exception", format!("创建失败:{:?}", err));
            JObject::null().into_inner()
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_setParameter(
    env: JNIEnv,
    _class: JClass,
    pico_key: JString,
    key: JString,
    val: JString
) {
    let mje = |err| format!("Pico参数设置失败: {:?}", err);
    let mrie = |err| format!("Pico参数设置失败: {:?}", err);
    let mrfe = |err| format!("Pico参数设置失败: {:?}", err);
    match || -> Result<(), String> {
        let pico_key:String = env.get_string(pico_key).map_err(mje)?.into();
        let key:String = env.get_string(key).map_err(mje)?.into();
        let val:String = env.get_string(val).map_err(mje)?.into();
        let mut picos = PICOS.lock().map_err(|err| format!("Pico删除失败: {:?}", err))?;
        match picos.get_mut(&pico_key){
            Some((pico, _last_image)) => {
                match key.as_str(){
                    "noupdatememory" => pico.set_noupdatememory(val.parse::<i32>().map_err(mrie)?),
                    "minsize" => pico.set_minsize(val.parse::<i32>().map_err(mrie)?),
                    "maxsize" => pico.set_maxsize(val.parse::<i32>().map_err(mrie)?),
                    "scalefactor" => pico.set_scalefactor(val.parse::<f32>().map_err(mrfe)?),
                    "stridefactor" => pico.set_stridefactor(val.parse::<f32>().map_err(mrfe)?),
                    "angle" => pico.set_angle(val.parse::<f32>().map_err(mrfe)?),
                    "qthreshold" => pico.set_qthreshold(val.parse::<f32>().map_err(mrfe)?),
                    _ => return Err("Pico参数不存在".to_string())
                };
                Ok(())
            },
            None => {
                Err("Pico不存在，请先创建".to_string())
            }
        }
    }() {
        Ok(_) => (),
        Err(err) => {
            let _ = env.throw_new("java/lang/Exception", format!("Pico参数设置失败:{:?}", err));
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_findObjects(
    env: JNIEnv,
    _class: JClass,
    pico_key: JString,
    bitmap: JObject,
    rotation_degrees: jint,
) -> jobject {
    let mje = |err| format!("识别失败 {:?}", err);
    let mut rects = None;
    // let mut t = Instant::now();
    let result = (|| -> Result<(), String> {
        let pico_key:String = env.get_string(pico_key).map_err(mje)?.into();
        let mut picos = PICOS.lock().map_err(|err| format!("识别失败: {:?}", err))?;
        let pico = picos.get_mut(&pico_key);
        if pico.is_none(){
            return Err("Pico不存在，请先创建".to_string());
        }
        let (pico, _last_image) = pico.unwrap();

        jni_graphics::lock_bitmap(&env, &bitmap, |info, pixels| {
            //只支持argb888格式
            if info.format != jni_graphics::ANDROID_BITMAP_FORMAT_RGBA_8888 {
                Err("图片格式只支持RGBA_8888!".to_string())
            } else {
                //创建
                let image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> =
                    ImageBuffer::from_raw(info.width, info.height, pixels.to_vec());
                // info!("image_from_raw耗时:{}ms", t.elapsed().as_millis());
                // t = Instant::now();
                if let Some(rgba_image) = image {
                    //灰度
                    let mut image: GrayImage = rgba_image.convert();
                    // info!("rust:灰度耗时:{}ms", t.elapsed().as_millis());
                    // t = Instant::now();

                    //旋转
                    match rotation_degrees{
                        90 => {
                            image = image::imageops::rotate90(&image);
                        },
                        180 => {
                            image = image::imageops::rotate180(&image);
                        },
                        270 => {
                            image = image::imageops::rotate270(&image);
                            //镜像翻转
                            image = image::imageops::flip_horizontal(&image);
                        }
                        _ => ()
                    };
                    // info!("rust:旋转耗时:{}ms", t.elapsed().as_millis());
                    // t = Instant::now();

                    //识别
                    let (width, height) = (image.width(), image.height());
                    let raw_data = image.into_raw();
                    let areas = pico.find_objects(&raw_data, height as i32, width as i32, width as i32);
                    // *last_image = Some(rgb_image);
                    // info!("rust:find_objects耗时:{}ms", t.elapsed().as_millis());
                    
                    //创建对象数组
                    let mut arr = vec![];
                    for area in areas {
                        let (x, y, radius, score) = {
                            (
                                area.x,
                                area.y,
                                area.radius,
                                area.score
                            )
                        };
                        arr.push(
                            env.new_object(
                                "io/github/planet0104/rustface/Area",
                                "(FFFFII)V",
                                &[
                                    JValue::from(x),
                                    JValue::from(y),
                                    JValue::from(radius),
                                    JValue::from(score),
                                    JValue::from(width as i32),
                                    JValue::from(height as i32)
                                ],
                            )
                            .map_err(mje)?,
                        );
                    }

                    let face_info_array = env
                        .new_object_array(
                            arr.len() as i32,
                            "io/github/planet0104/rustface/Area",
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

#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_getLastImage(
    env: JNIEnv,
    _class: JClass,
    pico_key: JString
) -> jobject {
    let mje = |err| format!("图片获取失败 {:?}", err);
    let result = (|| -> Result<jbyteArray, String> {
        let pico_key:String = env.get_string(pico_key).map_err(mje)?.into();
        let mut picos = PICOS.lock().map_err(|err| format!("图片获取失败: {:?}", err))?;
        let pico = picos.get_mut(&pico_key);
        if pico.is_none(){
            return Ok(JObject::null().into_inner());
        }
        if let (_pico, Some((gray, width, height))) = pico.unwrap(){
            let mut jpeg_data = vec![];
            let mut encoder = image::jpeg::JPEGEncoder::new(&mut jpeg_data);
            match encoder.encode(&gray, *width, *height, <Luma<u8> as image::Pixel>::color_type()){
                Ok(_) => {
                    Ok(env.byte_array_from_slice(&jpeg_data).map_err(mje)?)
                }
                Err(err) => {
                    error!("getLastImage: {:?}", err);
                    return Ok(JObject::null().into_inner());
                }
            }
        }else{
            Ok(JObject::null().into_inner())
        }
    })();

    match result{
        Err(err) => {
            let _ = env.throw_new("java/lang/Exception", format!("{:?}", err));
            JObject::null().into_inner()
        }
        Ok(obj) => {
            obj
        }
    }
}

//在慢速手机上，renderScript解码yuv速度会很慢，可以使用cpu解码yuv图像，直接创建灰度图
#[no_mangle]
pub extern "C" fn Java_io_github_planet0104_rustface_Pico_findObjectsYUV420P(
    env: JNIEnv,
    _class: JClass,
    pico_key: JString,
    data: jbyteArray,
    width:jint,
    height:jint,
    rotation_degrees: jint,
) -> jobject {
    let mje = |err| format!("识别失败 {:?}", err);
    let mut rects = None;
    let result = (|| -> Result<(), String> {
        let pico_key:String = env.get_string(pico_key).map_err(mje)?.into();
        let mut picos = PICOS.lock().map_err(|err| format!("识别失败: {:?}", err))?;
        let pico = picos.get_mut(&pico_key);
        if pico.is_none(){
            return Err("Pico不存在，请先创建".to_string());
        }
        let (pico, last_image) = pico.unwrap();

        let data = env.convert_byte_array(data).map_err(mje)?;
        // info!("convert_byte_array耗时:{}ms", t.elapsed().as_millis()); t = Instant::now();
        let gray_data = decode_yuv420sp(&data, width, height);
        // info!("decode_yuv420sp耗时:{}ms", t.elapsed().as_millis()); t = Instant::now();

        //创建
        let image: Option<ImageBuffer<Luma<u8>, Vec<u8>>> =
            ImageBuffer::from_raw(width as u32, height as u32, gray_data);
        // info!("image_from_raw耗时:{}ms", t.elapsed().as_millis()); t = Instant::now();
        if let Some(mut image) = image {
            //灰度
            // let mut image: GrayImage = image;
            // info!("rust:灰度耗时:{}ms", t.elapsed().as_millis());
            // t = Instant::now();

            //旋转
            match rotation_degrees{
                90 => {
                    image = image::imageops::rotate90(&image);
                },
                180 => {
                    image = image::imageops::rotate180(&image);
                },
                270 => {
                    image = image::imageops::rotate270(&image);
                    //镜像翻转
                    image = image::imageops::flip_horizontal(&image);
                }
                _ => ()
            };
            // info!("rust:旋转耗时:{}ms", t.elapsed().as_millis());
            // t = Instant::now();

            //识别
            let (width, height) = (image.width(), image.height());
            // *last_image = Some(image.clone());
            let raw_data = image.into_raw();
            let areas = pico.find_objects(&raw_data, height as i32, width as i32, width as i32);
            *last_image = Some((raw_data, width, height));
            // info!("rust:find_objects耗时:{}ms", t.elapsed().as_millis());
            
            //创建对象数组
            let mut arr = vec![];
            for area in areas {
                let (x, y, radius, score) = {
                    (
                        area.x,
                        area.y,
                        area.radius,
                        area.score
                    )
                };
                arr.push(
                    env.new_object(
                        "io/github/planet0104/rustface/Area",
                        "(FFFFII)V",
                        &[
                            JValue::from(x),
                            JValue::from(y),
                            JValue::from(radius),
                            JValue::from(score),
                            JValue::from(width as i32),
                            JValue::from(height as i32)
                        ],
                    )
                    .map_err(mje)?,
                );
            }

            let face_info_array = env
                .new_object_array(
                    arr.len() as i32,
                    "io/github/planet0104/rustface/Area",
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
) -> jobject {
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

/// android: YUV420SP 转 rgb
pub fn decode_yuv420sp(data:&[u8], width:i32, height:i32) -> Vec<u8>{
    let frame_size = width * height;
    let mut yp = 0;
    // let mut rgb = vec![0; (width*height*3) as usize];
    let mut gray = vec![0; (width*height) as usize];
    // let mut pi = 0;
    for j in 0..height{
        let (mut uvp, mut u, mut v) = ((frame_size + (j >> 1) * width) as usize, 0, 0);
        for i in 0..width{
            let mut y = (0xff & data[yp] as i32) - 16;  
            if y < 0 { y = 0; }
            if i & 1 == 0{
                v = (0xff & data[uvp] as i32) - 128;
                uvp += 1;
                u = (0xff & data[uvp] as i32) - 128;  
                uvp += 1;
            }
            let y1192 = 1192 * y;  
            let mut r = y1192 + 1634 * v;
            let mut g = y1192 - 833 * v - 400 * u;
            let mut b = y1192 + 2066 * u;

            if r < 0 { r = 0; } else if r > 262143 { r = 262143; };
            if g < 0 { g = 0; } else if g > 262143 { g = 262143; }
            if b < 0 { b = 0;} else if b > 262143 { b = 262143; }

            //rgb[yp] = 0xff000000u32 as i32 | ((r << 6) & 0xff0000) | ((g >> 2) & 0xff00) | ((b >> 10) & 0xff);
            
            // rgb[pi] = (r>>10) as u8;
            // rgb[pi+1] = (g>>10) as u8;
            // rgb[pi+2] = (b >> 10) as u8;
            // pi += 3;

            let color = (2*r+7*g+b)/10;

            gray[yp] = (color >> 10) as u8;

            yp += 1;
        }
    }
    gray
}