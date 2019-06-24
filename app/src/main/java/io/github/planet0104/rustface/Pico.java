package io.github.planet0104.rustface;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.util.Log;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;

public class Pico {
    private String key;

    public Pico(byte[] data) {
        this.key = create(data);
    }

    public Pico(Context context, String assetsPath) throws IOException {
        InputStream is = context.getAssets().open(assetsPath);
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        byte[] buffer = new byte[0xFFFF];
        for (int len = is.read(buffer); len != -1; len = is.read(buffer)) {
            os.write(buffer, 0, len);
        }
        this.key = create(os.toByteArray());
    }

    @Override
    protected void finalize() throws Throwable {
        remove(this.key);
        super.finalize();
    }

    public Area[] findObjects(Bitmap bitmap, int rotationDegrees){
        return findObjects(this.key, bitmap, rotationDegrees);
    }

    public Area[] findObjectsYUV420P(byte[] data, int width, int height, int rotationDegrees){
        return findObjectsYUV420P(this.key, data, width, height, rotationDegrees);
    }

    public void setMinSize(int minSize){
        setParameter(this.key, "minsize", minSize+"");
    }

    public void setMaxSize(int maxSize){
        setParameter(this.key, "maxsize", maxSize+"");
    }

    public void setScaleFactor(float scaleFactor){
        setParameter(this.key, "scalefactor", scaleFactor+"");
    }

    public Bitmap getLastImage(){
        byte[] bytes = getLastImage(this.key);
        return BitmapFactory.decodeStream(new ByteArrayInputStream(bytes));
    }

    public void setStrideFactor(float strideFactor){
        setParameter(this.key, "stridefactor", strideFactor+"");
    }

    public void setQThreshold(float threshold){
        setParameter(this.key, "qthreshold", threshold+"");
    }

    public void setNoUpdateMemory(boolean noUpdateMemory){
        setParameter(this.key, "noupdatememory", noUpdateMemory?"1":"0");
    }

    private static native byte[] getLastImage(String pico_key);
    private static native void setParameter(String pico_key, String key, String val);
    private static native Area[] findObjects(String key, Bitmap bitmap, int rotationDegrees);
    private static native Area[] findObjectsYUV420P(String key, byte[] data, int width, int height, int rotationDegrees);
    private static native String create(byte[] data);
    private static native void remove(String key);
}
