package com.core.fy.kmp.core

import androidx.compose.runtime.Composable

data class GraphPoint(val x: Double, val y: Double)
data class GraphSegment(val points: List<GraphPoint>)
data class SearchResult(val id: Long, val score: Float, val text: String)
data class ZipEntryInfo(val name: String, val compressedSize: Long, val uncompressedSize: Long, val isDirectory: Boolean)

expect object CoreRsPlatform {
    fun eval(expression: String): Double
    fun eval(expression: String, x: Double): Double
    fun containsVariable(expression: String): Boolean
    fun sampleGraph(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int = 1200, pixelWidth: Int = 1200, pixelHeight: Int = 800): List<GraphSegment>
    fun createSearch(): CoreSearch
    val sampleZipPath: String
    val apkPath: String
    fun createSampleZip(): String
    fun zipEntries(path: String): List<ZipEntryInfo>
    fun zipContains(path: String, name: String): Boolean
    fun apkIsApk(path: String): Boolean
    fun apkEntries(path: String): List<ZipEntryInfo>
}
expect class CoreSearch : AutoCloseable {
    fun add(id: Long, text: String)
    fun remove(id: Long): Boolean
    fun clear()
    fun size(): Int
    fun search(query: String, limit: Int = 20): List<SearchResult>
    override fun close()
}
