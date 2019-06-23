package io.github.planet0104.rustface;

public class Area {
    public float x;
    public float y;
    public float radius;
    public float score;
    public int imageWidth;
    public int imageHeight;

    public Area(float x, float y, float radius, float score, int imageWidth, int imageHeight) {
        this.x = x;
        this.y = y;
        this.radius = radius;
        this.score = score;
        this.imageWidth = imageWidth;
        this.imageHeight = imageHeight;
    }

    public FaceInfo asFaceInfo(){
        int x = (int)(this.x-this.radius);
        int y = (int)(this.y-this.radius);
        int width = (int)this.radius*2;
        int height = (int)this.radius*2;
        float score = this.score;
        float xRatio = (float) x/(float)this.imageWidth;
        float yRatio =  (float)y/(float)this.imageHeight;
        float widthRatio = (float)width/(float)this.imageWidth;
        float heightRatio = (float)height/(float)this.imageHeight;
        return new FaceInfo(x, y, width, height, score, xRatio, yRatio, widthRatio, heightRatio);
    }

    @Override
    public String toString() {
        return "Area{" +
                "x=" + x +
                ", y=" + y +
                ", radius=" + radius +
                ", score=" + score +
                ", imageWidth=" + imageWidth +
                ", imageHeight=" + imageHeight +
                '}';
    }
}
