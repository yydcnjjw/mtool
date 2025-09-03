package org.yydcnjjw.mtool.system

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.util.Log
import android.view.accessibility.AccessibilityEvent


class AccessibilityService : AccessibilityService() {

    companion object {
        private const val TAG = "mtool.AccessibilityService"
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        Log.d(TAG, "onServiceConnected")

        serviceInfo = AccessibilityServiceInfo().apply {
            packageNames = arrayOf("com.netease.cloudmusic")
            eventTypes = AccessibilityEvent.TYPES_ALL_MASK
            flags = AccessibilityServiceInfo.DEFAULT or
                    AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS or
                    AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            notificationTimeout = 100
        }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        Log.d(TAG, "$event")
        event ?: return

        when (event.eventType) {
            AccessibilityEvent.TYPE_WINDOWS_CHANGED -> {
                windows
            }
        }
    }

    override fun onInterrupt() {
        Log.d(TAG, "onInterrupt")
    }
}