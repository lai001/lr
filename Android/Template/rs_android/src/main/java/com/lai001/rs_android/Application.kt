package com.lai001.rs_android

import android.content.Context
import android.view.MotionEvent
import android.view.Surface
import java.io.InputStream

class ApplicationException(message: String) : Exception(message)

class Application(context: Context, artifact_name: String, surface: Surface) :
    java.io.Closeable {
    companion object {
        init {
            System.loadLibrary("rs_android")
        }
    }

    private var externalApplication: Long? = null

    private var artifact_input_stream: InputStream

    init {
        artifact_input_stream = context.assets.open(artifact_name)
        val externalApplication = Application_fromSurface(surface, artifact_input_stream)
        if (externalApplication != 0L) {
            this.externalApplication = externalApplication
        } else {
            throw ApplicationException("Failed to create application.")
        }
    }

    override fun close() {
        artifact_input_stream.close()
        externalApplication?.let { Application_drop(it) }
    }

    fun redraw() {
        externalApplication?.let { Application_redraw(it) }
    }

    fun onTouchEvent(event: MotionEvent?): Boolean {
        var ret = true
        event?.let {
            externalApplication?.let { nativeObject ->
                ret = Application_onTouchEvent(nativeObject, it)
            }
        }
        return ret
    }

    fun setEnvironment(env: Environment) {
        externalApplication?.let { Application_setEnvironment(it, env) }
    }

    fun surfaceChanged(format: Int, w: Int, h: Int) {
        externalApplication?.let {
            Application_surfaceChanged(
                it, format, w, h
            )
        }
    }

    fun setNewSurface(surface: Surface) {
        externalApplication?.let { Application_setNewSurface(it, surface) }
    }

    fun surfaceDestroyed(surface: Surface) {
        externalApplication?.let {
            Application_surfaceDestroyed(it, surface)
        }
    }

    private external fun Application_fromSurface(surface: Surface, inputStream: InputStream): Long

    private external fun Application_setNewSurface(
        application: Long, surface: Surface
    ): Boolean

    private external fun Application_redraw(application: Long)

    private external fun Application_drop(application: Long)

    private external fun Application_surfaceChanged(
        application: Long, format: Int, w: Int, h: Int
    )

    private external fun Application_onTouchEvent(
        application: Long, event: MotionEvent
    ): Boolean

    private external fun Application_setEnvironment(application: Long, env: Environment)

    private external fun Application_surfaceDestroyed(
        application: Long, surface: Surface
    )
}