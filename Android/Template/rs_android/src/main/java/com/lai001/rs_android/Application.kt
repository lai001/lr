package com.lai001.rs_android

import android.content.Context
import android.view.MotionEvent
import android.view.Surface
import com.lai001.lib.lrjni.Application
import java.io.InputStream

class ApplicationException(message: String) : Exception(message)

class Application(context: Context, artifactName: String, surface: Surface) : java.io.Closeable {
    companion object {
        init {
            AutoLoadLibs
        }
    }

    private var nativeApplication: Long

    private var artifactInputStream: InputStream

    init {
        artifactInputStream = context.assets.open(artifactName)
        nativeApplication = Application.fromSurface(surface, artifactInputStream).takeIf {
            it != 0L
        } ?: run {
            throw ApplicationException("Failed to create application.")
        }
    }

    override fun close() {
        artifactInputStream.close()
        Application.drop(nativeApplication)
    }

    fun redraw() {
        Application.redraw(nativeApplication)
    }

    fun onTouchEvent(event: MotionEvent?): Boolean {
        return event?.let {
            return Application.onTouchEvent(nativeApplication, it)
        } ?: true
    }

    fun setEnvironment(env: Environment) {
        val address = env.nativeAddress ?: return
        Application.setEnvironment(nativeApplication, address)
    }

    fun surfaceChanged(format: Int, w: Int, h: Int) {
        Application.surfaceChanged(nativeApplication, format, w, h)
    }

    fun setNewSurface(surface: Surface) {
        Application.setNewSurface(nativeApplication, surface)
    }

    fun surfaceDestroyed(surface: Surface) {
        Application.surfaceDestroyed(nativeApplication, surface)
    }
}
