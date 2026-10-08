package io.core.rs

import java.io.File

internal object CoreRsNative {
    private const val LIBRARY_NAME = "core_rs_jvm"

    init {
        val explicitPath = System.getProperty("core.rs.native.path")
            ?.takeIf { it.isNotBlank() }
            ?.let(::File)

        if (explicitPath != null) {
            require(explicitPath.isFile) {
                "Core-RS native library does not exist: ${explicitPath.absolutePath}"
            }
            System.load(explicitPath.absolutePath)
        } else {
            System.loadLibrary(LIBRARY_NAME)
        }
    }

    @JvmStatic external fun nativeExpressionEval(expression: String): Double
    @JvmStatic external fun nativeExpressionEvalX(expression: String, x: Double): Double
    @JvmStatic external fun nativeExpressionContainsVariable(expression: String): Boolean

    @JvmStatic external fun nativeGraph(
        expression: String,
        minX: Double,
        maxX: Double,
        minY: Double,
        maxY: Double,
        samples: Int,
        pixelWidth: Int,
        pixelHeight: Int,
    ): String

    @JvmStatic external fun nativeSearchCreate(): Long
    @JvmStatic external fun nativeSearchDestroy(handle: Long)
    @JvmStatic external fun nativeSearchAdd(handle: Long, id: Long, text: String): Boolean
    @JvmStatic external fun nativeSearchRemove(handle: Long, id: Long): Boolean
    @JvmStatic external fun nativeSearchClear(handle: Long)
    @JvmStatic external fun nativeSearchSize(handle: Long): Int
    @JvmStatic external fun nativeSearch(handle: Long, query: String, limit: Int): String

    @JvmStatic external fun nativeZipCreate(path: String): Long
    @JvmStatic external fun nativeZipDestroy(handle: Long)
    @JvmStatic external fun nativeZipEntries(handle: Long): String
    @JvmStatic external fun nativeZipContains(handle: Long, name: String): Boolean

    @JvmStatic external fun nativeApkIsApk(path: String): Boolean
    @JvmStatic external fun nativeApkEntries(path: String): String
}
