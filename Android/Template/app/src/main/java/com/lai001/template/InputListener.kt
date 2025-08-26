package com.lai001.template

import android.view.KeyEvent
import android.view.MotionEvent

interface InputListener {
    fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean?

    fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean?

    fun onTouchEvent(event: MotionEvent): Boolean?
}