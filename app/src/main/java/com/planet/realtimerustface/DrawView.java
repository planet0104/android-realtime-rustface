package com.planet.realtimerustface;

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

import io.github.planet0104.rustface.FaceInfo;

public class DrawView extends View {
    static final String TAG = DrawView.class.getSimpleName();
    FaceInfo[] faces = null;
    int timeUsed = 0;
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
            for(FaceInfo face:faces){
                int x = (int)(face.xRatio * getMeasuredWidth());
                int y = (int)(face.yRatio * getMeasuredHeight());
                int width = (int)(face.widthRatio * getMeasuredWidth());
                int height = (int)(face.heightRatio * getMeasuredHeight());
                canvas.drawRect(new Rect(x, y, x+width, y+height),paint);
//                canvas.drawRect(new Rect(face.x, face.y, face.x+face.width, face.y+face.height),paint);
            }
        }
        paint.setTextSize(32);
        paint.setStyle(Paint.Style.FILL);
        paint.setStrokeWidth(1);
        canvas.drawText(timeUsed+"ms", 50, 50, paint);
        super.draw(canvas);
    }

    public void drawFaces(FaceInfo[] faces, int timeUsed){
        this.faces = faces;
        this.timeUsed = timeUsed;
        this.invalidate();
    }

    public void clearFaces(){
        this.faces = new FaceInfo[]{};
        this.timeUsed = 0;
        this.invalidate();
    }
}
