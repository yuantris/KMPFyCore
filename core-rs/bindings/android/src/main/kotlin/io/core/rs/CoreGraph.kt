package io.core.rs

import org.json.JSONArray

data class GraphPoint(val x: Double, val y: Double)

data class GraphSegment(val points: List<GraphPoint>)

object CoreGraph {
    fun sample(expression: String, minX: Double, maxX: Double, minY: Double, maxY: Double, samples: Int = 1200): List<GraphSegment> {
        val root = JSONArray(CoreRsNative.nativeGraph(expression, minX, maxX, minY, maxY, samples))
        return buildList(root.length()) {
            for (i in 0 until root.length()) {
                val points = root.optJSONArray(i) ?: continue
                add(GraphSegment(buildList(points.length()) {
                    for (j in 0 until points.length()) {
                        val point = points.optJSONArray(j) ?: continue
                        add(GraphPoint(point.optDouble(0), point.optDouble(1)))
                    }
                }))
            }
        }
    }
}
