package com.core.fy.kmp.core

import io.core.rs.CoreRsNative
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

actual object CoreRsPlatform {
    actual fun eval(expression: String): Double {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionEval(expression)
    }

    actual fun eval(expression: String, x: Double): Double {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionEvalX(expression, x)
    }

    actual fun calculate(expression: String, mode: CalculatorMode): CalculationResult {
        require(expression.isNotBlank()) { "expression must not be blank" }
        val root = JSONObject(CoreRsNative.nativeCalculate(expression, mode.ordinal))
        return CalculationResult(
            type = root.optString("type"),
            value = root.optDouble("value"),
            numerator = if (root.has("numerator")) root.optLong("numerator") else null,
            denominator = if (root.has("denominator")) root.optLong("denominator") else null,
        )
    }

    actual fun containsVariable(expression: String): Boolean {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionContainsVariable(expression)
    }

    actual fun sampleGraph(
        expression: String,
        minX: Double,
        maxX: Double,
        minY: Double,
        maxY: Double,
        samples: Int,
        pixelWidth: Int,
        pixelHeight: Int,
        mode: GraphMode,
        secondExpression: String,
        minT: Double,
        maxT: Double,
    ): List<GraphSegment> {
        require(expression.isNotBlank()) { "expression must not be blank" }

        val root = JSONArray(
            CoreRsNative.nativeGraphAdvanced(
                expression, secondExpression, mode.ordinal,
                minX, maxX, minY, maxY, minT, maxT,
                samples, pixelWidth, pixelHeight,
            )
        )

        return buildList(root.length()) {
            for (segmentIndex in 0 until root.length()) {
                val points = root.optJSONArray(segmentIndex) ?: continue
                add(
                    GraphSegment(
                        buildList(points.length()) {
                            for (pointIndex in 0 until points.length()) {
                                val point = points.optJSONArray(pointIndex) ?: continue
                                if (point.length() >= 2) {
                                    add(GraphPoint(point.optDouble(0), point.optDouble(1)))
                                }
                            }
                        }
                    )
                )
            }
        }
    }

    actual fun analyzeGraph(
        expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int
    ): GraphAnalysis {
        val root = JSONObject(CoreRsNative.nativeGraphAnalyze(expression, minX, maxX, minY, maxY, samples))
        fun points(name: String): List<GraphPoint> {
            val array = root.optJSONArray(name) ?: return emptyList()
            return buildList(array.length()) {
                for (i in 0 until array.length()) {
                    val item = array.optJSONObject(i) ?: continue
                    add(GraphPoint(item.optDouble("x"), item.optDouble("y")))
                }
            }
        }
        return GraphAnalysis(points("zeroes"), points("extrema"))
    }

    actual fun createSearch(): CoreSearch = CoreSearch()

    actual val sampleZipPath: String
        get() = File(System.getProperty("java.io.tmpdir"), "fycore/core-rs-sample.zip").absolutePath

    actual val apkPath: String
        get() = System.getProperty("core.rs.apk.path")
            ?.takeIf { it.isNotBlank() }
            ?: sampleZipPath

    actual fun createSampleZip(): String {
        val file = File(sampleZipPath)
        file.parentFile?.mkdirs()

        ZipOutputStream(file.outputStream()).use { out ->
            val entries = listOf(
                "AndroidManifest.xml" to "<manifest/>".encodeToByteArray(),
                "classes.dex" to byteArrayOf(0x64, 0x65, 0x78, 0x0A),
                "assets/core.txt" to "core-rs-jvm".encodeToByteArray(),
            )
            entries.forEach { (name, bytes) ->
                out.putNextEntry(ZipEntry(name))
                out.write(bytes)
                out.closeEntry()
            }
        }

        return "示例 ZIP 已生成：$sampleZipPath"
    }

    actual fun zipEntries(path: String): List<ZipEntryInfo> =
        ZipReaderNative(path).use { reader ->
            parseEntries(CoreRsNative.nativeZipEntries(reader.handle))
        }

    actual fun zipContains(path: String, name: String): Boolean {
        require(name.isNotBlank()) { "name must not be blank" }
        return ZipReaderNative(path).use { reader ->
            CoreRsNative.nativeZipContains(reader.handle, name)
        }
    }

    actual fun apkIsApk(path: String): Boolean {
        require(path.isNotBlank()) { "path must not be blank" }
        return CoreRsNative.nativeApkIsApk(path)
    }

    actual fun apkEntries(path: String): List<ZipEntryInfo> {
        require(path.isNotBlank()) { "path must not be blank" }
        return parseEntries(CoreRsNative.nativeApkEntries(path))
    }
}

actual class CoreSearch : AutoCloseable {
    private var handle: Long = CoreRsNative.nativeSearchCreate()

    actual fun add(id: Long, text: String) {
        checkOpen()
        require(id >= 0) { "id must be >= 0" }
        require(text.isNotBlank()) { "text must not be blank" }
        check(CoreRsNative.nativeSearchAdd(handle, id, text)) { "native search add failed" }
    }

    actual fun remove(id: Long): Boolean {
        checkOpen()
        require(id >= 0) { "id must be >= 0" }
        return CoreRsNative.nativeSearchRemove(handle, id)
    }

    actual fun clear() {
        checkOpen()
        CoreRsNative.nativeSearchClear(handle)
    }

    actual fun size(): Int {
        checkOpen()
        return CoreRsNative.nativeSearchSize(handle)
    }

    actual fun search(query: String, limit: Int): List<SearchResult> {
        checkOpen()
        require(limit >= 0) { "limit must be >= 0" }

        val root = JSONArray(CoreRsNative.nativeSearch(handle, query, limit))
        return buildList(root.length()) {
            for (index in 0 until root.length()) {
                val item = root.optJSONObject(index) ?: continue
                add(
                    SearchResult(
                        id = item.optLong("id"),
                        score = item.optDouble("score").toFloat(),
                        text = item.optString("text"),
                    )
                )
            }
        }
    }

    actual override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeSearchDestroy(handle)
            handle = 0L
        }
    }

    private fun checkOpen() {
        check(handle != 0L) { "CoreSearch has been closed" }
    }
}

private class ZipReaderNative(path: String) : AutoCloseable {
    var handle: Long = CoreRsNative.nativeZipCreate(path)
        private set

    init {
        check(handle != 0L) { "Unable to open ZIP: $path" }
    }

    override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeZipDestroy(handle)
            handle = 0L
        }
    }
}

private fun parseEntries(json: String): List<ZipEntryInfo> {
    val root = JSONArray(json)
    return buildList(root.length()) {
        for (index in 0 until root.length()) {
            val item: JSONObject = root.optJSONObject(index) ?: continue
            add(
                ZipEntryInfo(
                    name = item.optString("name"),
                    compressedSize = item.optLong("compressed_size"),
                    uncompressedSize = item.optLong("uncompressed_size"),
                    isDirectory = item.optBoolean("is_dir"),
                )
            )
        }
    }
}
