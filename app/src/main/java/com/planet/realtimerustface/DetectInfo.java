package com.planet.realtimerustface;

import android.graphics.Bitmap;

public class DetectInfo{
    public Bitmap bitmap;
    public float scale;

    public DetectInfo(Bitmap bitmap, float scale) {
        this.bitmap = bitmap;
        this.scale = scale;
    }
}
