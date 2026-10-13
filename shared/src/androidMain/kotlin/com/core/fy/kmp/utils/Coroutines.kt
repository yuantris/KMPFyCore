package com.core.fy.kmp.utils

import kotlinx.coroutines.android.awaitFrame

actual suspend fun awaitFrame() {
    awaitFrame()
}
