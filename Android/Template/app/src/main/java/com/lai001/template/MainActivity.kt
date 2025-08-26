package com.lai001.template

import android.content.Context
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
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
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.lai001.rs_android.Environment
import com.lai001.template.ui.theme.TemplateTheme

const val TAG = "MainActivity"

class MySurfaceView(context: Context) : SurfaceView(context) {
    var inputListener: InputListener? = null

    init {
        isFocusable = true
        isFocusableInTouchMode = true
        requestFocus()
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (event.action == MotionEvent.ACTION_UP) {
            performClick()
        }
        return inputListener?.onTouchEvent(event) ?: super.onTouchEvent(event)
    }

    override fun performClick(): Boolean {
        return super.performClick()
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        return inputListener?.onKeyDown(keyCode, event) ?: super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        return inputListener?.onKeyUp(keyCode, event) ?: super.onKeyUp(keyCode, event)
    }
}

class MainActivityViewModel : ViewModel() {
    var mtApp: MTApp? = null
    var environment: Environment = Environment()

    fun init(context: Context) {
        if (mtApp == null) {
            mtApp = MTApp(context)
        }
        environment.statusBarHeight = getStatusBarHeight(context)
    }

    private fun getStatusBarHeight(context: Context): Int {
        var height = 0
        val resourceId =
            context.resources.getIdentifier("status_bar_height", "dimen", "android")
        if (resourceId > 0) {
            height = context.resources.getDimensionPixelSize(resourceId)
        }
        return height
    }
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val viewModel = ViewModelProvider(this)[MainActivityViewModel::class.java]
        viewModel.init(this)
        setContent {
            TemplateTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
                ) {
                    Greeting({ surfaceView ->
                        viewModel.mtApp?.let { app ->
                            app.setEnvironment(viewModel.environment)
                            surfaceView.holder.addCallback(app)
                            surfaceView.inputListener = app
                        }
                    })
                }
            }
        }
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