package com.planet.realtimerustface.pico;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Rect;
import android.util.AttributeSet;
import android.util.Log;
import android.view.View;

import androidx.annotation.Nullable;

import java.util.Arrays;

import io.github.planet0104.rustface.Area;

public class DrawView extends View {
    static final String TAG = DrawView.class.getSimpleName();
    Area[] faces = null;
    int fps = 0;
    public DrawView(Context context) {
        super(context);
    }

    public DrawView(Context context, @Nullable AttributeSet attrs) {
        super(context, attrs);
    }

    public DrawView(Context context, @Nullable AttributeSet attrs, int defStyleAttr) {
        super(context, attrs, defStyleAttr);
    }

    public DrawView(Context context, @Nullable AttributeSet attrs, int defStyleAttr, int defStyleRes) {
        super(context, attrs, defStyleAttr, defStyleRes);
    }

    @Override
    public void draw(Canvas canvas) {
        Paint paint = new Paint();
        paint.setStrokeWidth(4);
        paint.setColor(Color.YELLOW);
        paint.setStyle(Paint.Style.STROKE);
        if(faces == null || faces.length==0){
            Log.d(TAG, "没有检测到人脸");
        }else{
            Log.d(TAG, "检测到人脸:"+ Arrays.toString(faces));
            for(Area face:faces){
                float x = (face.x/(float)face.imageWidth)*getMeasuredWidth();
                float y = (face.y/(float)face.imageHeight)*getMeasuredHeight();
                float radius = (face.radius/(float)face.imageHeight)*getMeasuredHeight();
                int rx = (int) (x-radius);
                int ry = (int) (y-radius);
                int width = (int) (radius*2);
                int height = (int) (radius*2);
                Rect rect = new Rect(rx, ry, rx+width, ry+height);
                rect.inset((int)(radius*0.2), (int)(radius*0.2));
                canvas.drawRect(rect,paint);
            }
        }
        paint.setTextSize(32);
        paint.setStyle(Paint.Style.FILL);
        paint.setStrokeWidth(1);
        canvas.drawText("fps:"+fps, 50, 50, paint);
        super.draw(canvas);
    }

    public void drawFaces(Area[] faces, int fps){
        this.faces = faces;
        this.fps = fps;
        this.invalidate();
    }

    public void clearFaces(){
        postDelayed(new Runnable() {
            @Override
            public void run() {
                DrawView.this.faces = new Area[]{};
                DrawView.this.fps = 0;
                DrawView.this.invalidate();
            }
        }, 500);
    }
}
