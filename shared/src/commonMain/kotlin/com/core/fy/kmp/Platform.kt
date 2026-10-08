package com.core.fy.kmp

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform