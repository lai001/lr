package com.lai001.lib.lrjni;

public class Environment {
    public static native long newEnvironment();

    public static native void drop(long environment);

    public static native void setStatusBarHeight(long environment, int statusBarHeight);
}
