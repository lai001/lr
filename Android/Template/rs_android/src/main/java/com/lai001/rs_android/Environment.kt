package com.lai001.rs_android

import com.lai001.lib.lrjni.Environment

class Environment {
    companion object {
        init {
            AutoLoadLibs
        }
    }

    var statusBarHeight: Int = 0
        set(value) {
            nativeAddress?.let { Environment.setStatusBarHeight(it, value) }
        }

    var nativeAddress: Long? = null
        private set

    init {
        nativeAddress = Environment.newEnvironment().takeIf {
            it != 0L
        }
    }

    protected fun finalize() {
        nativeAddress?.let { Environment.drop(it) }
        nativeAddress = null
    }
}
