package com.core.fy.kmp

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import io.corer.rs.CoreGraph
import io.corer.rs.GraphSegment
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlin.math.max

@Composable
fun FunctionGraphScreen() {
    var expression by remember { mutableStateOf("sin(x)") }
    var segments by remember { mutableStateOf<List<GraphSegment>>(emptyList()) }
    var error by remember { mutableStateOf<String?>(null) }
    var minX by remember { mutableDoubleStateOf(-10.0) }
    var maxX by remember { mutableDoubleStateOf(10.0) }
    var minY by remember { mutableDoubleStateOf(-10.0) }
    var maxY by remember { mutableDoubleStateOf(10.0) }
    var loading by remember { mutableStateOf(false) }

    suspend fun sample() {
        loading = true
        error = null
        runCatching {
            withContext(Dispatchers.Default) {
                CoreGraph.sample(expression, minX, maxX, minY, maxY)
            }
        }.onSuccess { segments = it }.onFailure { error = it.message ?: "表达式错误" }
        loading = false
    }

    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("函数绘图", style = MaterialTheme.typography.headlineSmall)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedTextField(expression, { expression = it }, Modifier.weight(1f), singleLine = true, label = { Text("f(x)") })
            Button(onClick = { kotlinx.coroutines.MainScope().launch { sample() } }, enabled = !loading) { Text(if (loading) "绘制中" else "绘制") }
        }
        error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        GraphCanvas(
            segments = segments, minX = minX, maxX = maxX, minY = minY, maxY = maxY,
            onViewportChange = { dx, dy, scale ->
                val width = maxX - minX
                val height = maxY - minY
                val nx = width * scale
                val ny = height * scale
                val cx = (minX + maxX) / 2 - dx * width / 500.0
                val cy = (minY + maxY) / 2 + dy * height / 500.0
                minX = cx - nx / 2; maxX = cx + nx / 2
                minY = cy - ny / 2; maxY = cy + ny / 2
                kotlinx.coroutines.MainScope().launch { sample() }
            },
            modifier = Modifier.fillMaxWidth().height(0.dp).weight(1f)
        )
    }
}

@Composable
private fun GraphCanvas(
    segments: List<GraphSegment>, minX: Double, maxX: Double, minY: Double, maxY: Double,
    onViewportChange: (Double, Double, Double) -> Unit, modifier: Modifier
) {
    Canvas(modifier.pointerInput(Unit) {
        detectTransformGestures { _, pan, zoom, _ ->
            onViewportChange(pan.x, pan.y, 1.0 / zoom.coerceIn(0.6f, 1.6f))
        }
    }) {
        val sx = size.width / (maxX - minX).toFloat()
        val sy = size.height / (maxY - minY).toFloat()
        fun screenX(x: Double) = ((x - minX) * sx).toFloat()
        fun screenY(y: Double) = (size.height - (y - minY) * sy).toFloat()

        val gridStep = niceStep((maxX - minX) / 10.0)
        var gx = kotlin.math.floor(minX / gridStep) * gridStep
        while (gx <= maxX) {
            drawLine(MaterialTheme.colorScheme.outlineVariant, Offset(screenX(gx), 0f), Offset(screenX(gx), size.height))
            gx += gridStep
        }
        var gy = kotlin.math.floor(minY / gridStep) * gridStep
        while (gy <= maxY) {
            drawLine(MaterialTheme.colorScheme.outlineVariant, Offset(0f, screenY(gy)), Offset(size.width, screenY(gy)))
            gy += gridStep
        }
        if (minX <= 0 && maxX >= 0) drawLine(MaterialTheme.colorScheme.onSurface, Offset(screenX(0.0), 0f), Offset(screenX(0.0), size.height), strokeWidth = 2f)
        if (minY <= 0 && maxY >= 0) drawLine(MaterialTheme.colorScheme.onSurface, Offset(0f, screenY(0.0)), Offset(size.width, screenY(0.0)), strokeWidth = 2f)

        for (segment in segments) {
            if (segment.points.size < 2) continue
            val path = Path()
            segment.points.forEachIndexed { index, p ->
                val point = Offset(screenX(p.x), screenY(p.y))
                if (index == 0) path.moveTo(point.x, point.y) else path.lineTo(point.x, point.y)
            }
            drawPath(path, MaterialTheme.colorScheme.primary, style = Stroke(width = 3f, cap = StrokeCap.Round))
        }
    }
}

private fun niceStep(raw: Double): Double {
    val exponent = kotlin.math.floor(kotlin.math.log10(raw.coerceAtLeast(1e-9)))
    val fraction = raw / kotlin.math.pow(10.0, exponent)
    val nice = when { fraction < 1.5 -> 1.0; fraction < 3.0 -> 2.0; fraction < 7.0 -> 5.0; else -> 10.0 }
    return nice * kotlin.math.pow(10.0, exponent)
}
