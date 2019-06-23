package io.github.planet0104.rustface;

import android.content.Context;
import android.graphics.Bitmap;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * 基于rustface库的人脸检测
 */
public class RustFace {
    /**
     * 检测人脸
     * @param bitmap 图片
     * @param scale 缩放比例(比例越小识别速度越快)
     * @return FaceInfo数组
     */
    public static native FaceInfo[] detect(Bitmap bitmap, float scale);

    /**
     * 检测人脸(不缩放)
     * @param bitmap 图片数据
     * @return FaceInfo数组
     */
    public static FaceInfo[] detect(Bitmap bitmap){
        return detect(bitmap, 1.0f);
    }
    public static native void setMinFaceSize(int min_face_size);
    public static native void setScoreThresh(double score_thresh);
    public static native void setPyramidScaleFactor(float pyramid_scale_factor);
    public static native void setSlideWindowStep(int step_x, int step_y);
    public static native void setWindowSize(int wnd_size);

    /**
     * 初始化 RustFace
     * @param context Context
     * @param fileName "seeta_fd_frontal_v1.0.bin"
     * @throws IOException
     */
    public static void createFromAssets(Context context, String fileName) throws IOException {
        InputStream is = context.getAssets().open(fileName);
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        byte[] buffer = new byte[0xFFFF];
        for (int len = is.read(buffer); len != -1; len = is.read(buffer)) {
            os.write(buffer, 0, len);
        }
        RustFace.create(os.toByteArray());
    }

    /**
     *  初始化
     * @param data 文件seeta_fd_frontal_v1.0.bin
     */
    public static native void create(byte[] data);
}