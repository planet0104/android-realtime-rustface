package com.planet.realtimerustface.pico;

import android.content.Context;
import android.graphics.Bitmap;
import android.os.Handler;
import android.os.Looper;
import android.os.Message;

import com.planet.realtimerustface.DetectInfo;

import java.io.IOException;

import io.github.planet0104.rustface.Area;
import io.github.planet0104.rustface.Pico;

public class Detector extends Thread{
    static final String TAG = com.planet.realtimerustface.Detector.class.getSimpleName();
    private Handler handler;
    private Handler callback;
    private int frameTime;
    private long nextTime = System.currentTimeMillis();
    private Context context;
    private Pico pico;

    public Detector(Context context, String assetsPath, int fps, Handler callback) throws IOException {
        this.frameTime = (int)(1000.0 / (float)fps);
        this.callback = callback;
        this.context = context;
        this.pico = new Pico(context, assetsPath);
    }

    public void run() {
        Looper.prepare();
        handler = new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message msg) {
                DetectInfo info = (DetectInfo) msg.obj;
                long t = System.currentTimeMillis();
                Area[] areas = pico.findObjects(info.bitmap, info.scale);
                Message msg1 = Detector.this.callback.obtainMessage();
                msg1.obj = areas;
                msg1.arg1 = (int) (System.currentTimeMillis()-t);
                Detector.this.callback.sendMessage(msg1);
                return false;
            }
        });
        Looper.loop();
    }

    public boolean readyForNext(){
        return System.currentTimeMillis()>=nextTime;
    }

    public void detect1(Bitmap bitmap, float scale){
        nextTime = System.currentTimeMillis()+frameTime;
        if(handler == null){
            return;
        }
        Message msg = handler.obtainMessage();
        msg.obj = new DetectInfo(bitmap, scale);
        handler.sendMessage(msg);
    }
}