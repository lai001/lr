package com.lai001.lib.lrjni;

import android.view.MotionEvent;
import android.view.Surface;

import java.io.InputStream;

public class Application {
    public static native long fromSurface(Surface surface, float scaleFactor, InputStream inputStream);

    public static native boolean setNewSurface(long application, Surface surface);

    public static native void redraw(long application);

    public static native void drop(long application);

    public static native void surfaceChanged(long application, int format, int w, int h);

    public static native boolean onTouchEvent(long application, MotionEvent event);

    public static native void setEnvironment(long application, long environment);

    public static native void surfaceDestroyed(long application, Surface surface);
}