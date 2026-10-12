package com.core.fy.kmp.core

import android.content.Context
import io.core.rs.ApkReader
import io.core.rs.CoreGraph
import io.core.rs.CoreRsNative
import io.core.rs.Expression
import org.json.JSONArray
import org.json.JSONObject
import io.core.rs.SearchEngine
import io.core.rs.ZipReader
import java.io.File
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

private object CoreRsContext {
    lateinit var context: Context
}
actual object CoreRsPlatform {
    actual fun eval(expression: String) = Expression.eval(expression)
    actual fun eval(expression: String, x: Double) = Expression.eval(expression, x)
    actual fun containsVariable(expression: String) = Expression.containsVariable(expression)
    actual fun calculate(expression: String, mode: CalculatorMode, angleMode: CalculatorAngleMode): CalculationResult {
        require(expression.isNotBlank()) { "expression must not be blank" }
        val root = JSONObject(CoreRsNative.nativeCalculate(expression, mode.ordinal, angleMode.ordinal))
        return CalculationResult(
            type = root.optString("type"),
            value = root.optDouble("value"),
            numerator = if (root.has("numerator")) root.optLong("numerator") else null,
            denominator = if (root.has("denominator")) root.optLong("denominator") else null,
        )
    }
    actual fun sampleGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int, pixelWidth: Int, pixelHeight: Int, mode: GraphMode, secondExpression: String, minT: Double, maxT: Double): List<GraphSegment> =
        CoreRsNative.nativeGraphAdvanced(expression, secondExpression, mode.ordinal, minX, maxX, minY, maxY, minT, maxT, samples, pixelWidth, pixelHeight).let { json -> JSONArray(json).let { root -> buildList(root.length()) { for (i in 0 until root.length()) { val points = root.optJSONArray(i) ?: continue; add(GraphSegment(buildList(points.length()) { for (j in 0 until points.length()) { val p = points.optJSONArray(j) ?: continue; if (p.length() >= 2) add(GraphPoint(p.optDouble(0), p.optDouble(1))) } })) } } } }
    actual fun analyzeGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int): GraphAnalysis {
        val root = JSONObject(CoreRsNative.nativeGraphAnalyze(expression, minX, maxX, minY, maxY, samples))
        fun points(name: String) = root.optJSONArray(name)?.let { a -> buildList(a.length()) { for (i in 0 until a.length()) { val p = a.optJSONObject(i) ?: continue; add(GraphPoint(p.optDouble("x"), p.optDouble("y"))) } } } ?: emptyList()
        return GraphAnalysis(points("zeroes"), points("extrema"))
    }
    actual fun stitchFiles(
        inputPaths: List<String>,
        outputPath: String,
        config: ImageStitchConfig,
    ): ImageStitchResult {
        require(inputPaths.isNotEmpty()) { "at least one input image is required" }
        require(outputPath.isNotBlank()) { "outputPath must not be blank" }
        require(config.background.size == 4 && config.background.all { it in 0..255 }) {
            "background must contain four RGBA values in 0..255"
        }
        val configJson = JSONObject()
            .put("mode", config.mode.name.lowercase())
            .put("max_width", config.maxWidth)
            .put("max_height", config.maxHeight)
            .put("min_overlap_ratio", config.minOverlapRatio.toDouble())
            .put("max_overlap_ratio", config.maxOverlapRatio.toDouble())
            .put("min_confidence", config.minConfidence.toDouble())
            .put("blend", config.blend)
            .put("output_format", config.outputFormat)
            .put("columns", config.columns)
            .put("spacing", config.spacing)
            .put("background", JSONArray().apply { config.background.forEach { put(it) } })
            .put("max_output_pixels", config.maxOutputPixels)
        val pathsJson = JSONArray().apply { inputPaths.forEach { put(it) } }.toString()
        val root = JSONObject(CoreRsNative.nativeStitch(pathsJson, outputPath, configJson.toString()))
        val segmentsJson = if (root.has("segments") && !root.isNull("segments")) {
            root.getJSONArray("segments")
        } else {
            JSONArray()
        }
        val segments = buildList(segmentsJson.length()) {
            for (index in 0 until segmentsJson.length()) {
                val item = if (segmentsJson.isNull(index)) continue else segmentsJson.getJSONObject(index)
                add(
                    ImageStitchSegment(
                        imageIndex = item.getInt("image_index"),
                        x = item.getLong("x"),
                        y = item.getLong("y"),
                        cropX = item.getInt("crop_x"),
                        cropY = item.getInt("crop_y"),
                        cropWidth = item.getInt("crop_width"),
                        cropHeight = item.getInt("crop_height"),
                        confidence = item.getDouble("confidence").toFloat(),
                    )
                )
            }
        }
        val mode = ImageStitchMode.entries.firstOrNull {
            it.name.equals(root.getString("mode"), ignoreCase = true)
        } ?: config.mode
        return ImageStitchResult(
            width = root.getInt("width"),
            height = root.getInt("height"),
            imageCount = root.getInt("image_count"),
            mode = mode,
            segments = segments,
        )
    }

    actual fun createSearch(): CoreSearch = CoreSearch()
    actual val sampleZipPath: String get() = File(CoreRsContext.context.cacheDir, "core-rs-sample.zip").absolutePath
    actual val apkPath: String get() = CoreRsContext.context.applicationInfo.sourceDir
    actual fun createSampleZip(): String {
        ZipOutputStream(File(sampleZipPath).outputStream()).use { out ->
            listOf("AndroidManifest.xml" to "<manifest/>".encodeToByteArray(), "classes.dex" to byteArrayOf(0x64,0x65,0x78,0x0A), "assets/core.txt" to "core-rs".encodeToByteArray()).forEach { (name, bytes) ->
                out.putNextEntry(ZipEntry(name)); out.write(bytes); out.closeEntry()
            }
        }
        return "示例 ZIP 已生成"
    }
    actual fun zipEntries(path: String) = ZipReader(path).use { it.entries().map { e -> ZipEntryInfo(e.name,e.compressedSize,e.uncompressedSize,e.isDirectory) } }
    actual fun zipContains(path: String, name: String) = ZipReader(path).use { it.contains(name) }
    actual fun apkIsApk(path: String) = ApkReader(path).isApk()
    actual fun apkEntries(path: String) = ApkReader(path).entries().map { ZipEntryInfo(it.name,it.compressedSize,it.uncompressedSize,it.isDirectory) }
}
actual class CoreSearch : AutoCloseable {
    private val delegate = SearchEngine()
    actual fun add(id: Long, text: String) = delegate.add(id, text)
    actual fun remove(id: Long) = delegate.remove(id)
    actual fun clear() = delegate.clear()
    actual fun size() = delegate.size()
    actual fun search(query: String, limit: Int) = delegate.search(query, limit).map { SearchResult(it.id,it.score,it.text) }
    actual override fun close() = delegate.close()
}
fun installCoreRsContext(context: Context) { CoreRsContext.context = context.applicationContext }
