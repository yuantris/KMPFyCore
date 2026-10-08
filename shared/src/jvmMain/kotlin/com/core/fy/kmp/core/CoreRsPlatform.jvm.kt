package com.core.fy.kmp.core

actual object CoreRsPlatform {
    actual fun eval(expression: String): Double = error("Core-RS JVM binding is not available yet")
    actual fun eval(expression: String, x: Double): Double = error("Core-RS JVM binding is not available yet")
    actual fun containsVariable(expression: String): Boolean = error("Core-RS JVM binding is not available yet")
    actual fun sampleGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int, pixelWidth: Int, pixelHeight: Int): List<GraphSegment> = error("Core-RS JVM binding is not available yet")
    actual fun createSearch(): CoreSearch = CoreSearch()
    actual val sampleZipPath: String get() = ""
    actual val apkPath: String get() = ""
    actual fun createSampleZip() = "Desktop 当前没有 JVM Core-RS binding"
    actual fun zipEntries(path: String): List<ZipEntryInfo> = error("Core-RS JVM binding is not available yet")
    actual fun zipContains(path: String, name: String): Boolean = error("Core-RS JVM binding is not available yet")
    actual fun apkIsApk(path: String): Boolean = error("Core-RS JVM binding is not available yet")
    actual fun apkEntries(path: String): List<ZipEntryInfo> = error("Core-RS JVM binding is not available yet")
}
actual class CoreSearch : AutoCloseable {
    actual fun add(id: Long, text: String) = error("Core-RS JVM binding is not available yet")
    actual fun remove(id: Long): Boolean = error("Core-RS JVM binding is not available yet")
    actual fun clear() = error("Core-RS JVM binding is not available yet")
    actual fun size(): Int = 0
    actual fun search(query: String, limit: Int): List<SearchResult> = emptyList()
    actual override fun close() = Unit
}
