package com.planet.realtimerustface.pico;

import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.res.Resources;
import android.graphics.Bitmap;
import android.graphics.Matrix;
import android.graphics.SurfaceTexture;
import android.graphics.drawable.Drawable;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.util.Rational;
import android.util.Size;
import android.view.Surface;
import android.view.TextureView;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.appcompat.app.AppCompatActivity;
import androidx.camera.core.CameraX;
import androidx.camera.core.ImageAnalysis;
import androidx.camera.core.ImageAnalysisConfig;
import androidx.camera.core.ImageCapture;
import androidx.camera.core.ImageCaptureConfig;
import androidx.camera.core.ImageProxy;
import androidx.camera.core.Preview;
import androidx.camera.core.PreviewConfig;
import androidx.core.content.ContextCompat;

import com.planet.realtimerustface.ImageUtils;
import com.planet.realtimerustface.R;

import java.io.IOException;
import java.util.Arrays;

import io.github.planet0104.rustface.Area;
import io.github.planet0104.rustface.Pico;

public class PicoActivity extends AppCompatActivity{
    static{
        System.loadLibrary("rustface");
    }
    private String TAG = PicoActivity.class.getSimpleName();

    private int REQUEST_CODE_PERMISSIONS = 10; //arbitrary number, can be changed accordingly
    private final String[] REQUIRED_PERMISSIONS = new String[]{"android.permission.CAMERA"}; //array w/ permissions from manifest
    TextureView viewFinder;

    DrawView drawView;
    int detectCount = 0;

    CameraX.LensFacing facing = CameraX.LensFacing.FRONT;
    boolean enableDetect = true;
    private Detector picoDetector;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_pico);
        viewFinder = findViewById(R.id.view_finder);

        //View root = findViewById(R.id.root);
        View root = viewFinder;
        root.setOutlineProvider(new TextureVideoViewOutlineProvider(20));
        root.setClipToOutline(true);
        Resources res = getResources();
        Drawable drawable = res.getDrawable(android.R.color.transparent);
        getWindow().setBackgroundDrawable(drawable);

        drawView = findViewById(R.id.drawView);

        if(allPermissionsGranted()){
            startCamera();
        }else{
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                requestPermissions(REQUIRED_PERMISSIONS, REQUEST_CODE_PERMISSIONS);
            }
        }
        detectCount = 0;
    }

    private void startCamera() {
        if(picoDetector == null){
            try {
                picoDetector = new Detector(this,"facefinder",  new Handler(new Handler.Callback() {
                    @Override
                    public boolean handleMessage(Message msg) {
                        Area[] faces = (Area[]) msg.obj;
                        //1~2秒中准备时间 = 16
                        if(faces.length>0){
                            detectCount += 1;
                            if(detectCount>=20){
                                Toast.makeText(PicoActivity.this, "检测到人脸！！", Toast.LENGTH_LONG).show();
                                //在onActivityResult中返回人脸的数据和图片
                                Intent i = new Intent();
                                i.putExtra("faces", faces);
                                i.putExtra("bitmap", viewFinder.getBitmap());
                                setResult(RESULT_OK, new Intent());
                                finish();
                            }
                        }else{
                            detectCount = 0;
                        }
                        drawView.drawFaces(faces, msg.arg1);
                        return false;
                    }
                }));
                picoDetector.start();
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
        //make sure there isn't another camera instance running before starting
        CameraX.unbindAll();

        /* start preview */
//        int aspRatioW = viewFinder.getWidth(); //get width of screen
//        int aspRatioH = viewFinder.getHeight(); //get height

        int aspRatioW = 480;
        int aspRatioH = 640;
        Log.d(TAG, "预览大小:"+aspRatioW+"x"+aspRatioH);
        Rational asp = new Rational (aspRatioW, aspRatioH); //aspect ratio
        Size screen = new Size(aspRatioW, aspRatioH); //size of the screen
        try{
            CameraManager cameraManager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
            CameraCharacteristics cameraCharacteristics = cameraManager.getCameraCharacteristics("1");
            StreamConfigurationMap streamConfigurationMap = cameraCharacteristics.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
            Size[] sizes = streamConfigurationMap.getOutputSizes(SurfaceTexture.class);
            Log.d(TAG, "sizes="+ Arrays.toString(sizes));
        }catch (Exception e){
            e.printStackTrace();
        }

        //config obj for preview/viewfinder thingy.
        PreviewConfig pConfig = new PreviewConfig.Builder().setTargetAspectRatio(asp)
                .setLensFacing(facing).setTargetResolution(screen).build();
        Preview preview = new Preview(pConfig); //lets build it

        preview.setOnPreviewOutputUpdateListener(
                new Preview.OnPreviewOutputUpdateListener() {
                    //to update the surface texture we  have to destroy it first then re-add it
                    @Override
                    public void onUpdated(Preview.PreviewOutput output){
                        ViewGroup parent = (ViewGroup) viewFinder.getParent();
                        parent.removeView(viewFinder);
                        parent.addView(viewFinder, 0);

                        viewFinder.setSurfaceTexture(output.getSurfaceTexture());
                        updateTransform();
                    }
                });

        /* image capture */


        //config obj, selected capture mode
        ImageCaptureConfig imgCConfig = new ImageCaptureConfig.Builder().setCaptureMode(ImageCapture.CaptureMode.MIN_LATENCY)
                .setLensFacing(facing)
                .setTargetRotation(getWindowManager().getDefaultDisplay().getRotation()).build();
        final ImageCapture imgCap = new ImageCapture(imgCConfig);

        /* image analyser */

        ImageAnalysisConfig imgAConfig = new ImageAnalysisConfig.Builder()
                .setImageReaderMode(ImageAnalysis.ImageReaderMode.ACQUIRE_LATEST_IMAGE)
                .setLensFacing(facing)
                .build();
        ImageAnalysis analysis = new ImageAnalysis(imgAConfig);

        analysis.setAnalyzer(
                new ImageAnalysis.Analyzer(){
                    @Override
                    public void analyze(ImageProxy image, int rotationDegrees){
                        //y'all can add code to analyse stuff here idek go wild.
                        //ImageFormat
                        Image rawImage = image.getImage();
                        if(enableDetect && rawImage!=null && picoDetector.readyForNext()){
                            Detector.DetectInfo info = new Detector.DetectInfo();
                            info.image = ImageUtils.imageToByteBuffer(rawImage).array();
                            info.width = rawImage.getWidth();
                            info.height = rawImage.getHeight();
                            info.rotationDegrees = rotationDegrees;
                            picoDetector.detect(info);
                        }
                    }
                });

        //bind to lifecycle:
        CameraX.bindToLifecycle(this, analysis, imgCap, preview);
    }

    private void updateTransform(){
        //compensates the changes in orientation for the viewfinder, bc the rest of the layout stays in portrait mode.
        //methinks :thonk:
        Matrix mx = new Matrix();
        float w = viewFinder.getMeasuredWidth();
        float h = viewFinder.getMeasuredHeight();

        float cX = w / 2f; //calc centre of the viewfinder
        float cY = h / 2f;

        int rotationDgr;
        int rotation = (int)viewFinder.getRotation(); //cast to int bc switches don't like floats

        switch(rotation){ //correct output to account for display rotation
            case Surface.ROTATION_0:
                rotationDgr = 0;
                break;
            case Surface.ROTATION_90:
                rotationDgr = 90;
                break;
            case Surface.ROTATION_180:
                rotationDgr = 180;
                break;
            case Surface.ROTATION_270:
                rotationDgr = 270;
                break;
            default:
                return;
        }

        mx.postRotate((float)rotationDgr, cX, cY);
        //apply transformations to textureview
        viewFinder.setTransform(mx);
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions, @NonNull int[] grantResults) {
        //start camera when permissions have been granted otherwise exit app
        if(requestCode == REQUEST_CODE_PERMISSIONS){
            if(allPermissionsGranted()){
                startCamera();
            } else{
                Toast.makeText(this, "Permissions not granted by the user.", Toast.LENGTH_SHORT).show();
                finish();
            }
        }
    }

    private boolean allPermissionsGranted(){
        //check if req permissions have been granted
        for(String permission : REQUIRED_PERMISSIONS){
            if(ContextCompat.checkSelfPermission(this, permission) != PackageManager.PERMISSION_GRANTED){
                return false;
            }
        }
        return true;
    }

    public void changeFront(View view) {
        if(facing != CameraX.LensFacing.FRONT){
            facing = CameraX.LensFacing.FRONT;
            startCamera();
            drawView.clearFaces();
        }
    }

    public void changeBack(View view) {
        if(facing != CameraX.LensFacing.BACK){
            facing = CameraX.LensFacing.BACK;
            startCamera();
            drawView.clearFaces();
        }
    }

    public void stopDetect(View view) {
        enableDetect = false;
        drawView.clearFaces();
    }

    public void startDetect(View view) {
        enableDetect = true;
    }

    public void capture(View view) {
        ImageView img = findViewById(R.id.image);
        img.setImageBitmap(viewFinder.getBitmap());
    }
}