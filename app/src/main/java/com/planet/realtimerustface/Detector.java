package com.planet.realtimerustface;

import android.content.Context;
import android.graphics.Bitmap;
import android.os.Handler;
import android.os.Looper;
import android.os.Message;
import android.util.Log;

import java.io.IOException;

import io.github.planet0104.rustface.FaceInfo;
import io.github.planet0104.rustface.RustFace;

class DetectInfo{
    Bitmap bitmap;
    float scale;

    DetectInfo(Bitmap bitmap, float scale) {
        this.bitmap = bitmap;
        this.scale = scale;
    }
}

public class Detector extends Thread{
    static final String TAG = Detector.class.getSimpleName();
    private Handler handler;
    private Handler callback;
    private int frameTime;
    private long nextTime = System.currentTimeMillis();
    private String dataFilePath;
    private Context context;

    public Detector(Context context, String assetsPath, int fps, Handler callback) {
        this.frameTime = (int)(1000.0 / (float)fps);
        this.callback = callback;
        this.context = context;
        this.dataFilePath = assetsPath;
    }

    public void run() {
        //初始化
        try {
            RustFace.createFromAssets(context, dataFilePath);
            RustFace.setMinFaceSize(20);
            RustFace.setScoreThresh(2.0);
            RustFace.setPyramidScaleFactor(0.8f);
            RustFace.setSlideWindowStep(4, 4);
        } catch (IOException e) {
            Log.e(TAG, "RustFace创建失败:"+e.getMessage());
            e.printStackTrace();
            return;
        }
        Log.d(TAG, "RustFace创建成功, 启动Looper");
        Looper.prepare();
        Log.d(TAG, "Looper.prepare OK.");
        handler = new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message msg) {
                DetectInfo info = (DetectInfo) msg.obj;
                long t = System.currentTimeMillis();
                FaceInfo[] faces = RustFace.detect(info.bitmap, info.scale);
                Message msg1 = Detector.this.callback.obtainMessage();
                msg1.obj = faces;
                msg1.arg1 = (int) (System.currentTimeMillis()-t);
                Detector.this.callback.sendMessage(msg1);
                return false;
            }
        });
        Log.d(TAG, "启动loop:");
        Looper.loop();
    }

    public boolean readyForNext(){
        return System.currentTimeMillis()>=nextTime;
    }

    public void detect(Bitmap bitmap, float scale){
        nextTime = System.currentTimeMillis()+frameTime;
        if(handler == null){
            return;
        }
        Message msg = handler.obtainMessage();
        msg.obj = new DetectInfo(bitmap, scale);
        handler.sendMessage(msg);
    }
}