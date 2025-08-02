package org.yydcnjjw.mtool.dioxus

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultCallback
import androidx.activity.result.contract.ActivityResultContracts
import org.yydcnjjw.mtool.dioxus.wry.WryActivity


class MainActivity : WryActivity() {
    companion object {
        private const val TAG = "mtool.MainActivity"

        init {
            System.loadLibrary("dioxusmain")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")
    }

    override fun setContentView(view: View?) {
        val layout = FrameLayout(this)
        layout.fitsSystemWindows = true
        layout.addView(view, ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.MATCH_PARENT
        ))
        super.setContentView(layout)
    }

    override fun onDestroy() {
        super.onDestroy()
    }

    val startForActivityResultLauncher =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { it ->
            {
                startForActivityResultLauncherCallbacks.remove(it.data)?.onActivityResult(it)
            }
        }
    val startForActivityResultLauncherCallbacks =
        mutableMapOf<Intent, ActivityResultCallback<ActivityResult>>()

    fun launchForActivityResult(
        intent: Intent,
        callback: ActivityResultCallback<ActivityResult>,
    ) {
        startForActivityResultLauncherCallbacks[intent] = callback
        startForActivityResultLauncher.launch(intent)
    }
}