package com.planet.realtimerustface.pico;

import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.os.Message;

import java.io.IOException;

import io.github.planet0104.rustface.Area;
import io.github.planet0104.rustface.Pico;

public class Detector extends Thread{
    static final String TAG = com.planet.realtimerustface.Detector.class.getSimpleName();
    private Handler handler;
    private Handler callback;
    private int frameTime;
    private long nextTime = System.currentTimeMillis();
    private Pico pico;

    //每隔1秒中统计平均耗时
    private long nextCountTime = 0;
    private int totalTime = 0;
    private int totalCount = 0;
    private int fps = 0;

    public static class DetectInfo{
        public byte[] image;
        public int width;
        public int height;
        public int rotationDegrees;
    }

    public Detector(Context context, String assetsPath, Handler callback) throws IOException {
        this.frameTime = 50; //默认300ms检测一帧
        this.callback = callback;
        this.pico = new Pico(context, assetsPath);
        pico.setQThreshold(20.0f);
        pico.setMinSize(70);
        //pico.setNoUpdateMemory(true);
    }

    public void run() {
        Looper.prepare();
        handler = new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message msg) {
                DetectInfo info = (DetectInfo) msg.obj;
                long t = System.currentTimeMillis();
                Area[] areas = pico.findObjectsYUV420P(info.image, info.width, info.height, info.rotationDegrees);
                Message msg1 = Detector.this.callback.obtainMessage();
                msg1.obj = areas;
                int time = (int) (System.currentTimeMillis()-t);
                msg1.arg1 = fps;
                totalTime += time;
                totalCount += 1;
                if(nextCountTime == 0){
                    nextCountTime = System.currentTimeMillis()+1000;
                }else if(System.currentTimeMillis()>nextCountTime){
                    nextCountTime = System.currentTimeMillis()+1000;
                    fps = totalCount;
                    int avg = (int)(Math.ceil((double)totalTime/totalCount)*1.2);//扩大一定的时间
                    nextTime = System.currentTimeMillis()+avg;
                    totalTime = 0;
                    totalCount = 0;
                }
                Detector.this.callback.sendMessage(msg1);
                return false;
            }
        });
        Looper.loop();
    }

    public boolean readyForNext(){
        return System.currentTimeMillis()>=nextTime;
    }

    public void detect(DetectInfo info){
        nextTime = System.currentTimeMillis()+frameTime;
        if(handler == null){
            return;
        }
        Message msg = handler.obtainMessage();
        msg.obj = info;
        handler.sendMessage(msg);
    }
}