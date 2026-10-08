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
    actual fun calculate(expression: String, mode: CalculatorMode): CalculationResult {
        require(expression.isNotBlank()) { "expression must not be blank" }
        val value = when (mode) {
            CalculatorMode.Basic, CalculatorMode.Scientific, CalculatorMode.Fraction -> Expression.eval(expression)
        }
        return CalculationResult("real", value)
    }
    actual fun sampleGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int, pixelWidth: Int, pixelHeight: Int, mode: GraphMode, secondExpression: String, minT: Double, maxT: Double): List<GraphSegment> =
        CoreRsNative.nativeGraphAdvanced(expression, secondExpression, mode.ordinal, minX, maxX, minY, maxY, minT, maxT, samples, pixelWidth, pixelHeight).let { json -> JSONArray(json).let { root -> buildList(root.length()) { for (i in 0 until root.length()) { val points = root.optJSONArray(i) ?: continue; add(GraphSegment(buildList(points.length()) { for (j in 0 until points.length()) { val p = points.optJSONArray(j) ?: continue; if (p.length() >= 2) add(GraphPoint(p.optDouble(0), p.optDouble(1))) } })) } } } }
    actual fun analyzeGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int): GraphAnalysis {
        val root = JSONObject(CoreRsNative.nativeGraphAnalyze(expression, minX, maxX, minY, maxY, samples))
        fun points(name: String) = root.optJSONArray(name)?.let { a -> buildList(a.length()) { for (i in 0 until a.length()) { val p = a.optJSONObject(i) ?: continue; add(GraphPoint(p.optDouble("x"), p.optDouble("y"))) } } } ?: emptyList()
        return GraphAnalysis(points("zeroes"), points("extrema"))
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
