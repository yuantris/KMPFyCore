package com.core.fy.kmp.utils

import kotlinx.coroutines.delay

actual suspend fun awaitFrame() {
    delay(1000L / 60L)
}
