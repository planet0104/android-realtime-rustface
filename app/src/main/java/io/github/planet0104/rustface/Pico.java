package io.github.planet0104.rustface;

import android.content.Context;
import android.graphics.Bitmap;

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

    public Area[] findObjects(Bitmap bitmap, float scale){
        return findObjects(this.key, bitmap, scale);
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

    public void setStrideFactor(float strideFactor){
        setParameter(this.key, "stridefactor", strideFactor+"");
    }

    public void setQThreshold(float threshold){
        setParameter(this.key, "qthreshold", threshold+"");
    }

    private static native void setParameter(String pico_key, String key, String val);
    private static native Area[] findObjects(String key, Bitmap bitmap, float scale);
    private static native String create(byte[] data);
    private static native void remove(String key);
}
