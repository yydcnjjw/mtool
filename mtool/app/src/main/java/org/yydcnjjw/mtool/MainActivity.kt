package org.yydcnjjw.mtool

import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.util.Log
import androidx.activity.result.contract.ActivityResultContracts.StartActivityForResult
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    companion object {
        private const val TAG = "mtool.MainActivity"

    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")

        startNotificationListener()
    }

    override fun onDestroy() {
        super.onDestroy()
    }

    fun isNotificationListenerEnabled(): Boolean {
        val flat = Settings.Secure.getString(
            contentResolver,
            "enabled_notification_listeners"
        )
        return flat.contains(packageName)
    }

    fun startNotificationListener() {
        if (isNotificationListenerEnabled()) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val intent = Intent(this, NotificationListener::class.java)
                startForegroundService(intent)
            }
            return
        }
        val launcher = registerForActivityResult(
            StartActivityForResult(),
        ) { it ->
            {
                startNotificationListener()
            }
        }
        launcher.launch(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS))
    }
}