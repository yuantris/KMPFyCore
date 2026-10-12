package com.core.fy.kmp.core

import androidx.compose.runtime.Composable

data class GraphPoint(val x: Double, val y: Double)
data class GraphSegment(val points: List<GraphPoint>)

enum class GraphMode { Cartesian, Polar, Parametric }
enum class CalculatorMode { Basic, Scientific, Fraction }
enum class CalculatorAngleMode { Rad, Deg }
data class CalculationResult(val type: String, val value: Double, val numerator: Long? = null, val denominator: Long? = null)

data class GraphAnalysis(
    val zeroes: List<GraphPoint>,
    val extrema: List<GraphPoint>,
)
data class SearchResult(val id: Long, val score: Float, val text: String)
data class ZipEntryInfo(val name: String, val compressedSize: Long, val uncompressedSize: Long, val isDirectory: Boolean)

enum class ImageStitchMode { Scroll, Panorama, Vertical, Horizontal, Grid, Auto }

data class ImageStitchConfig(
    val mode: ImageStitchMode = ImageStitchMode.Scroll,
    val maxWidth: Int = 0,
    val maxHeight: Int = 0,
    val minOverlapRatio: Float = 0.10f,
    val maxOverlapRatio: Float = 0.85f,
    val minConfidence: Float = 0.45f,
    val blend: Boolean = true,
    val outputFormat: String = "png",
    val columns: Int = 0,
    val spacing: Int = 12,
    val background: List<Int> = listOf(255, 255, 255, 255),
    val maxOutputPixels: Long = 100_000_000L,
)

data class ImageStitchSegment(
    val imageIndex: Int,
    val x: Long,
    val y: Long,
    val cropX: Int,
    val cropY: Int,
    val cropWidth: Int,
    val cropHeight: Int,
    val confidence: Float,
)

data class ImageStitchResult(
    val width: Int,
    val height: Int,
    val imageCount: Int,
    val mode: ImageStitchMode,
    val segments: List<ImageStitchSegment>,
)

expect object CoreRsPlatform {
    fun eval(expression: String): Double
    fun eval(expression: String, x: Double): Double
    fun containsVariable(expression: String): Boolean
    fun calculate(expression: String, mode: CalculatorMode, angleMode: CalculatorAngleMode = CalculatorAngleMode.Rad): CalculationResult
    fun sampleGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int = 1200, pixelWidth: Int = 1200, pixelHeight: Int = 800, mode: GraphMode = GraphMode.Cartesian, secondExpression: String = "", minT: Double = -10.0, maxT: Double = 10.0): List<GraphSegment>
    fun analyzeGraph(
        expression: String,
        minX: Double,
        maxX: Double,
        minY: Double,
        maxY: Double,
        samples: Int = 1200,
    ): GraphAnalysis
    fun createSearch(): CoreSearch
    val sampleZipPath: String
    val apkPath: String
    fun createSampleZip(): String
    fun zipEntries(path: String): List<ZipEntryInfo>
    fun zipContains(path: String, name: String): Boolean
    fun apkIsApk(path: String): Boolean
    fun apkEntries(path: String): List<ZipEntryInfo>
    fun stitchFiles(
        inputPaths: List<String>,
        outputPath: String,
        config: ImageStitchConfig = ImageStitchConfig(),
    ): ImageStitchResult
}
expect class CoreSearch : AutoCloseable {
    fun add(id: Long, text: String)
    fun remove(id: Long): Boolean
    fun clear()
    fun size(): Int
    fun search(query: String, limit: Int = 20): List<SearchResult>
    override fun close()
}
