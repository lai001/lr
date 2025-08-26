package com.lai001.template

import android.content.Context
import android.util.Log
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import com.lai001.rs_android.Application
import com.lai001.rs_android.Environment
import java.io.Closeable

sealed class SMsgType {
    class TouchEvent(val event: MotionEvent) : SMsgType()
    class KeyDown(val keyCode: Int, val event: KeyEvent) : SMsgType()
    class KeyUp(val keyCode: Int, val event: KeyEvent) : SMsgType()
    class SurfaceCreated(val surface: Surface) : SMsgType()
    class SurfaceDestroyed(val surface: Surface) : SMsgType()
    class SurfaceChanged(val format: Int, val w: Int, val h: Int) : SMsgType()
    class SetEnvironment(val env: Environment) : SMsgType()
    object Close : SMsgType()
}

class MTApp(private val context: Context) : Closeable, SurfaceHolder.Callback, InputListener {
    private val lock = Any()
    private var thread: Thread? = null
    private var messages: ArrayDeque<SMsgType> = ArrayDeque()

    init {
        val scaleFactor = context.resources.displayMetrics.density

        thread = Thread {
            var application: Application? = null

            while (true) {
                val message: SMsgType?
                synchronized(lock) {
                    message = messages.removeFirstOrNull()
                }

                when (message) {
                    is SMsgType.SurfaceCreated -> {
                        if (application == null) {
                            application =
                                Application(context, "main.rs", message.surface, scaleFactor)
                        } else {
                            application.setNewSurface(message.surface)
                        }
                    }

                    is SMsgType.SurfaceChanged -> {
                        application?.surfaceChanged(message.format, message.w, message.h)
                    }

                    is SMsgType.SurfaceDestroyed -> {
                        application?.surfaceDestroyed(message.surface)
                    }

                    is SMsgType.TouchEvent -> {
                        application?.onTouchEvent(message.event)
                    }

                    is SMsgType.SetEnvironment -> {
                        if (application == null) {
                            synchronized(lock) {
                                messages.addLast(message)
                            }
                        } else {
                            application.setEnvironment(message.env)
                        }
                    }

                    is SMsgType.Close -> {
                        application?.close()
                        application = null
                        break
                    }

                    is SMsgType.KeyDown -> {
                        application?.onKeyDown(message.keyCode, message.event)
                    }

                    is SMsgType.KeyUp -> {
                        application?.onKeyUp(message.keyCode, message.event)
                    }

                    null -> {}
                }
                application?.redraw()
                if (application == null) {
                    Thread.sleep(10)
                }
            }
        }
        thread?.start()
    }

    override fun close() {
        synchronized(lock) {
            messages.add(SMsgType.Close)
        }
        thread = null
    }

    private fun touchEvent(event: MotionEvent) {
        val crossThreadEvent = MotionEvent.obtain(event)
        synchronized(lock) {
            messages.add(SMsgType.TouchEvent(crossThreadEvent))
        }
    }

    private fun surfaceCreated(surface: Surface) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceCreated(surface))
        }
    }

    private fun surfaceDestroyed(surface: Surface) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceDestroyed(surface))
        }
    }

    private fun surfaceChanged(surface: Surface, format: Int, w: Int, h: Int) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceChanged(format, w, h))
        }
    }

    fun setEnvironment(env: Environment) {
        synchronized(lock) {
            messages.add(SMsgType.SetEnvironment(env))
        }
    }

    override fun surfaceCreated(p0: SurfaceHolder) {
        surfaceCreated(p0.surface)
    }

    override fun surfaceChanged(p0: SurfaceHolder, format: Int, w: Int, h: Int) {
        surfaceChanged(p0.surface, format, w, h)
    }

    override fun surfaceDestroyed(p0: SurfaceHolder) {
        surfaceDestroyed(p0.surface)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        synchronized(lock) {
            messages.add(SMsgType.KeyDown(keyCode, event))
        }
        return true
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        synchronized(lock) {
            messages.add(SMsgType.KeyUp(keyCode, event))
        }
        return true
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        touchEvent(event)
        return true
    }
}