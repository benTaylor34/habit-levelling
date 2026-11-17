package com.habitlevelling.app

import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // The app is dark-only, so always use light status/navigation bar icons.
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)

    // Edge-to-edge draws the app under the status bar, and the WebView doesn't
    // pass those insets on to CSS. Pad the content by the system bars instead,
    // and paint the padding in the app's background colour.
    window.decorView.setBackgroundColor(Color.parseColor("#0A0B0F"))
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      // While the on-screen keyboard is open, shrink the app above it so
      // forms (e.g. the log popup's "Log it" button) aren't hidden under it.
      val keyboard = insets.getInsets(WindowInsetsCompat.Type.ime())
      view.setPadding(bars.left, bars.top, bars.right, maxOf(bars.bottom, keyboard.bottom))
      WindowInsetsCompat.CONSUMED
    }

    // Back first asks the page (src/lib/back.ts) to close a popup or return to
    // Today. Only when there's nothing left to close does Android's normal Back
    // (leaving the app) happen.
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val webView = findWebView(window.decorView)
        if (webView == null) {
          exitViaSystemBack()
          return
        }
        webView.evaluateJavascript(
          "(window.__habitBack && window.__habitBack()) ? 'handled' : 'exit'"
        ) { result ->
          if (result?.contains("handled") != true) exitViaSystemBack()
        }
      }

      private fun exitViaSystemBack() {
        isEnabled = false
        onBackPressedDispatcher.onBackPressed()
        isEnabled = true
      }
    })
  }

  private fun findWebView(view: View): WebView? {
    if (view is WebView) return view
    if (view is ViewGroup) {
      for (i in 0 until view.childCount) {
        findWebView(view.getChildAt(i))?.let { return it }
      }
    }
    return null
  }
}
