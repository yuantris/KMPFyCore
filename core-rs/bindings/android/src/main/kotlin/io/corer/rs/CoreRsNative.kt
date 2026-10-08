package io.corer.rs

internal object CoreRsNative {
    init { System.loadLibrary("core_rs_android") }

    @JvmStatic external fun nativeExpressionEval(expression: String): Double
    @JvmStatic external fun nativeGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int): String
    @JvmStatic external fun nativeSearchCreate(): Long
    @JvmStatic external fun nativeSearchDestroy(handle: Long)
    @JvmStatic external fun nativeSearchAdd(handle: Long, id: Long, text: String): Boolean
    @JvmStatic external fun nativeSearchRemove(handle: Long, id: Long): Boolean
    @JvmStatic external fun nativeSearch(handle: Long, query: String, limit: Int): String
    @JvmStatic external fun nativeZipCreate(path: String): Long
    @JvmStatic external fun nativeZipDestroy(handle: Long)
    @JvmStatic external fun nativeZipEntries(handle: Long): String
    @JvmStatic external fun nativeApkIsApk(path: String): Boolean
}
