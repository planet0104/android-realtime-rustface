package io.github.planet0104.rustface;

public class FaceInfo {
    public int x;
    public int y;
    public int width;
    public int height;
    public double score;

    public float xRatio;
    public float yRatio;
    public float widthRatio;
    public float heightRatio;

    public FaceInfo(int x, int y, int width, int height, double score,
                    float xRatio, float yRatio, float widthRatio, float heightRatio){
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
        this.score = score;
        this.xRatio = xRatio;
        this.yRatio = yRatio;
        this.widthRatio = widthRatio;
        this.heightRatio = heightRatio;
    }

    @Override
    public String toString() {
        return "FaceInfo{" +
                "x=" + x +
                ", y=" + y +
                ", width=" + width +
                ", height=" + height +
                ", score=" + score +
                '}';
    }
}
