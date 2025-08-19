package org.yydcnjjw.mtool.dioxus

import android.content.Intent
import android.content.res.AssetManager
import android.os.Bundle
import android.system.Os
import android.util.Log
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.activity.result.ActivityResult
import androidx.activity.result.ActivityResultCallback
import androidx.activity.result.contract.ActivityResultContracts
import org.yydcnjjw.mtool.dioxus.wry.WryActivity
import java.io.File
import java.io.FileOutputStream
import java.util.zip.ZipFile
import java.util.zip.ZipInputStream

class MainActivity : WryActivity() {
    companion object {
        private const val TAG = "mtool.MainActivity"

        init {
            System.loadLibrary("dioxusmain")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        tryInstallPython()
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")
    }


    override fun setContentView(view: View?) {
        val layout = FrameLayout(this)
        layout.fitsSystemWindows = true
        layout.addView(
            view, ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
        )
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

    fun tryInstallPython() {
        val pythonHome = filesDir.resolve("python")

        Os.setenv("HTTP_PROXY", "http://10.30.183.179:8188", true)
        Os.setenv("HTTPS_PROXY", "http://10.30.183.179:8188", true)

        Os.setenv("PYTHONPATH", "$pythonHome/python3.13:$pythonHome/python3.13/site-packages", true)
        Os.setenv("PYTHONPLATLIBDIR", "$pythonHome", true)
        copyAssetDirectory(assets, "python", pythonHome)

        val zipInputStream = ZipInputStream(assets.open("python313.zip"))
        unzip(zipInputStream, pythonHome)
        zipInputStream.close()
    }

    fun unzip(zipInputStream: ZipInputStream, destDir: File) {
        if (!destDir.exists()) {
            destDir.mkdirs()
        }

        var zipEntry = zipInputStream.nextEntry
        while (zipEntry != null) {
            val file = destDir.resolve(zipEntry.name)
            if (zipEntry.isDirectory) {
                file.mkdirs()
            } else {
                file.parentFile?.mkdirs()
                val out = file.outputStream()
                zipInputStream.copyTo(out)
                out.close()
            }
            zipEntry = zipInputStream.nextEntry
        }
    }

    fun copyAssetDirectory(assetManager: AssetManager, source: String, dest: File) {
        if (!dest.exists()) {
            dest.mkdirs()
        }

        val files = assetManager.list(source)
        if (files != null) {
            for (filename in files) {
                val path = "$source/$filename"
                runCatching {
                    assetManager.open(path)
                }.onSuccess {
                    val out = FileOutputStream(dest.resolve(filename))
                    it.copyTo(out)
                    it.close()
                    out.close()
                }.onFailure {
                    copyAssetDirectory(assetManager, path, dest.resolve(filename))
                }
            }
        }
    }
}
