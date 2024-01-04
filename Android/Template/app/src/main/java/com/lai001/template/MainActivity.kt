package com.lai001.template

import android.content.Context
import android.os.Bundle
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.ViewGroup
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.viewinterop.AndroidView
import com.lai001.rs_android.Application
import com.lai001.rs_android.Environment
import com.lai001.template.ui.theme.TemplateTheme
import java.io.Closeable

const val TAG = "MainActivity"

class MySurfaceView(context: android.content.Context) : SurfaceView(context) {

    var touchEvent: ((MotionEvent?) -> Boolean)? = null

    override fun onTouchEvent(event: MotionEvent?): Boolean {
        if (event?.action == MotionEvent.ACTION_DOWN) {
            performClick()
        }
        return if (touchEvent != null) {
            touchEvent!!.invoke(event)
        } else {
            super.onTouchEvent(event)
        }
    }

    override fun performClick(): Boolean {
        return super.performClick()
    }
}

class MainActivity : ComponentActivity() {
    val enviroment: Environment = Environment()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enviroment.statusBarHeight = getStatusBarHeight()

        setContent {
            TemplateTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
                ) {
                    Greeting({ surfaceView ->
                        val callback = MySurfaceCallback()
                        callback.mtApp.context = this
                        callback.setEnviroment(enviroment)
                        surfaceView.holder.addCallback(callback)
                        surfaceView.touchEvent = { motionEvent ->
                            callback.onTouchEvent(motionEvent)
                        }
                    })
                }
            }
        }
    }

    fun getStatusBarHeight(): Int {
        var height = 0
        val resourceId =
            applicationContext.resources.getIdentifier("status_bar_height", "dimen", "android")
        if (resourceId > 0) {
            height = applicationContext.resources.getDimensionPixelSize(resourceId)
        }
        return height
    }
}

sealed class SMsgType {
    class TouchEvent(val event: MotionEvent) : SMsgType()
    class SurfaceCreated(val surface: Surface) : SMsgType()
    class SurfaceDestroyed(val surface: Surface) : SMsgType()
    class SurfaceChanged(val format: Int, val w: Int, val h: Int) : SMsgType()
    class SetEnvironment(val env: Environment) : SMsgType()
    class Close : SMsgType()
}

class MTApp : Closeable {
    private val lock = Any()
    private var thread: Thread? = null
    private var messages: ArrayDeque<SMsgType> = ArrayDeque()
     var context: Context? = null

    init {
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
                            try {
                                application =
                                    context?.let { Application(it, "test.rs", message.surface) }
                            } catch (e: Exception) {

                            }
                        } else {
                            application.setNewSurface(message.surface)
                        }
                    }

                    is SMsgType.SurfaceChanged -> {
                        application?.surfaceChanged(message.format, message.w, message.h)
                    }

                    is SMsgType.SurfaceDestroyed -> {
                        application?.surfaceDestroyed(message.surface)
                        break
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

                    null -> {}
                }
                application?.redraw()
                Thread.sleep(16)
            }
        }
        thread?.start()
    }

    override fun close() {
        synchronized(lock) {
            messages.add(SMsgType.Close())
        }
        thread = null
    }

    fun touchEvent(event: MotionEvent) {
        synchronized(lock) {
            messages.add(SMsgType.TouchEvent(event))
        }
    }

    fun surfaceCreated(surface: Surface) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceCreated(surface))
        }
    }

    fun surfaceDestroyed(surface: Surface) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceDestroyed(surface))
        }
    }

    fun surfaceChanged(format: Int, w: Int, h: Int) {
        synchronized(lock) {
            messages.add(SMsgType.SurfaceChanged(format, w, h))
        }
    }

    fun setEnviroment(env: Environment) {
        synchronized(lock) {
            messages.add(SMsgType.SetEnvironment(env))
        }
    }
}

class MySurfaceCallback : SurfaceHolder.Callback {
    var mtApp = MTApp()

    override fun surfaceCreated(p0: SurfaceHolder) {
        mtApp.surfaceCreated(p0.surface)
    }

    override fun surfaceChanged(p0: SurfaceHolder, format: Int, w: Int, h: Int) {
        mtApp.surfaceChanged(format, w, h)
    }

    override fun surfaceDestroyed(p0: SurfaceHolder) {
        mtApp.surfaceDestroyed(p0.surface)
    }

    fun onTouchEvent(event: MotionEvent?): Boolean {
        event?.let {
            mtApp.touchEvent(it)
        }
        return true
    }

    fun setEnviroment(env: Environment) {
        mtApp.setEnviroment(env)
    }
}

@Composable
fun Greeting(
    surfaceViewCreated: (MySurfaceView) -> Unit, modifier: Modifier = Modifier
) {
    AndroidView(factory = { context ->
        val surfaceView = MySurfaceView(context).apply {
            layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT
            )
        }
        surfaceViewCreated(surfaceView)
        surfaceView
    }, modifier = modifier)
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    TemplateTheme {
        Greeting({

        })
    }
}