package com.core.fy.kmp.core

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.kyant.backdrop.backdrops.LayerBackdrop
import com.kyant.backdrop.backdrops.layerBackdrop
import com.kyant.backdrop.backdrops.rememberLayerBackdrop
import com.core.fy.kmp.core.CoreRsPlatform
import com.core.fy.kmp.core.CoreSearch
import com.core.fy.kmp.core.GraphSegment
import com.core.fy.kmp.core.SearchResult
import com.core.fy.kmp.core.ZipEntryInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.pow

private enum class CoreTab(val title: String, val symbol: String) {
    Math("数学", "ƒ"), Graph("绘图", "∿"), Search("搜索", "⌕"), Binary("文件", "zip")
}

@Composable
fun CorePlaygroundContent() {
    var tabIndex by remember { mutableIntStateOf(0) }
    val backdrop = rememberLayerBackdrop()
    Box(Modifier.fillMaxSize()) {
        CoreBackdrop(backdrop)
        Column(
            Modifier.fillMaxSize().navigationBarsPadding().padding(horizontal = 16.dp)
        ) {
            Spacer(Modifier.height(42.dp))
            Text("Core RS", style = MaterialTheme.typography.headlineMedium)
            Text(
                "Rust Native Playground",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.65f),
            )
            Spacer(Modifier.height(16.dp))
            Box(Modifier.weight(1f)) {
                when (CoreTab.entries[tabIndex]) {
                    CoreTab.Math -> MathPanel(backdrop)
                    CoreTab.Graph -> GraphPanel(backdrop)
                    CoreTab.Search -> SearchPanel(backdrop)
                    CoreTab.Binary -> BinaryPanel(backdrop)
                }
            }
            Spacer(Modifier.height(10.dp))
            LiquidGlassPill(backdrop, Modifier.fillMaxWidth()) {
                CoreTab.entries.forEachIndexed { index, item ->
                    LiquidButton(
                        onClick = { tabIndex = index },
                        modifier = Modifier.weight(1f),
                    ) {
                        Text(
                            item.symbol + "  " + item.title,
                            color = if (index == tabIndex) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            Spacer(Modifier.height(8.dp))
        }
    }
}

@Composable
private fun CoreBackdrop(backdrop: LayerBackdrop) {
    Box(
        Modifier.fillMaxSize().layerBackdrop(backdrop).background(
            Brush.linearGradient(
                listOf(Color(0xFFF2F5FF), Color(0xFFE9F8F4), Color(0xFFF9F0FA))
            )
        )
    ) {
        Box(
            Modifier.size(260.dp)
                .background(Color(0xFF9BB8FF).copy(alpha = 0.25f), RoundedCornerShape(130.dp))
                .align(Alignment.TopEnd)
        )
        Box(
            Modifier.size(220.dp)
                .background(Color(0xFF86E0C0).copy(alpha = 0.20f), RoundedCornerShape(110.dp))
                .align(Alignment.BottomStart)
        )
    }
}

@Composable
private fun MathPanel(backdrop: LayerBackdrop) {
    var expression by remember { mutableStateOf("sqrt(9) + abs(-2) + pi") }
    var x by remember { mutableDoubleStateOf(2.0) }
    var result by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        item {
            LiquidGlassCard(backdrop, Modifier.fillMaxWidth()) {
                Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Expression", style = MaterialTheme.typography.titleLarge)
                    Text(
                        "lexer · parser · AST · constant / variable evaluation",
                        style = MaterialTheme.typography.bodySmall,
                    )
                    OutlinedTextField(
                        value = expression,
                        onValueChange = { expression = it },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        label = { Text("表达式") },
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        LiquidButton(onClick = {
                            scope.launch {
                                error = null
                                runCatching {
                                    withContext(Dispatchers.Default) { CoreRsPlatform.eval(expression) }
                                }.onSuccess { result = "constant = " + it }
                                    .onFailure { error = it.message }
                            }
                        }, backdrop = backdrop) { Text("常量求值") }
                        LiquidButton(onClick = {
                            scope.launch {
                                error = null
                                runCatching {
                                    withContext(Dispatchers.Default) { CoreRsPlatform.eval(expression, x) }
                                }.onSuccess { result = "f(" + x + ") = " + it }
                                    .onFailure { error = it.message }
                            }
                        }, backdrop = backdrop) { Text("f(x)") }
                    }
                    OutlinedTextField(
                        value = x.toString(),
                        onValueChange = { it.toDoubleOrNull()?.let { value -> x = value } },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        label = { Text("x") },
                    )
                    LiquidButton(onClick = {
                        runCatching { CoreRsPlatform.containsVariable(expression) }
                            .onSuccess { result = "contains variable x = " + it }
                            .onFailure { error = it.message }
                    }, backdrop = backdrop) { Text("检查是否包含 x") }
                    result?.let { Text(it, color = MaterialTheme.colorScheme.primary) }
                    error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        item {
            LiquidGlassCard(backdrop, Modifier.fillMaxWidth()) {
                Column(Modifier.padding(18.dp)) {
                    Text("Supported functions", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text(
                        "sin · cos · tan · asin · acos · atan · sqrt · abs · ln · log10 · exp · floor · ceil",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    Spacer(Modifier.height(6.dp))
                    Text(
                        "Operators: +  −  ×  ÷  %  ^  · implicit multiplication · pi · e",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        }
    }
}

@Composable
private fun GraphPanel(backdrop: LayerBackdrop) {
    var expression by remember { mutableStateOf("sin(x) + 0.2 * x") }
    var segments by remember { mutableStateOf<List<GraphSegment>>(emptyList()) }
    var minX by remember { mutableDoubleStateOf(-10.0) }
    var maxX by remember { mutableDoubleStateOf(10.0) }
    var minY by remember { mutableDoubleStateOf(-5.0) }
    var maxY by remember { mutableDoubleStateOf(5.0) }
    var canvasSize by remember { mutableStateOf(IntSize.Zero) }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    suspend fun sampleNow() {
        if (canvasSize.width < 2 || canvasSize.height < 2) return
        loading = true
        error = null
        runCatching {
            withContext(Dispatchers.Default) {
                CoreRsPlatform.sampleGraph(
                    expression, minX, maxX, minY, maxY,
                    pixelWidth = canvasSize.width,
                    pixelHeight = canvasSize.height,
                )
            }
        }.onSuccess { segments = it }
            .onFailure { error = it.message ?: "绘图失败" }
        loading = false
    }

    LaunchedEffect(canvasSize, expression, minX, maxX, minY, maxY) {
        sampleNow()
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = expression,
                    onValueChange = { expression = it },
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                    label = { Text("f(x)") },
                )
                Spacer(Modifier.width(8.dp))
                LiquidButton(onClick = {}, backdrop = backdrop, isInteractive = false) {
                    if (loading) CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                    else Text("绘制")
                }
            }
            error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            GraphCanvas(
                segments, minX, maxX, minY, maxY,
                Modifier.fillMaxWidth().weight(1f).onSizeChanged { canvasSize = it },
                onViewportChange = { dx, dy, scale ->
                    val width = maxX - minX
                    val height = maxY - minY
                    val nx = (width * scale).coerceIn(0.05, 200.0)
                    val ny = (height * scale).coerceIn(0.05, 200.0)
                    val cx = (minX + maxX) / 2 - dx * width / 500.0
                    val cy = (minY + maxY) / 2 + dy * height / 500.0
                    minX = cx - nx / 2
                    maxX = cx + nx / 2
                    minY = cy - ny / 2
                    maxY = cy + ny / 2
                },
            )
        }
    }
}

@Composable
private fun GraphCanvas(
    segments: List<GraphSegment>,
    minX: Double,
    maxX: Double,
    minY: Double,
    maxY: Double,
    modifier: Modifier,
    onViewportChange: (Double, Double, Double) -> Unit,
) {
    val primaryColor = MaterialTheme.colorScheme.primary

    Canvas(
        modifier.pointerInput(Unit) {
            detectTransformGestures { _, pan, zoom, _ ->
                onViewportChange(pan.x.toDouble(), pan.y.toDouble(), 1.0 / zoom.coerceIn(0.7f, 1.5f))
            }
        }
    ) {
        if (maxX <= minX || maxY <= minY) return@Canvas
        val sx = size.width / (maxX - minX).toFloat()
        val sy = size.height / (maxY - minY).toFloat()
        fun screenX(x: Double) = ((x - minX) * sx).toFloat()
        fun screenY(y: Double) = (size.height - (y - minY) * sy).toFloat()
        val step = niceStep((maxX - minX) / 10.0)
        var gx = floor(minX / step) * step
        while (gx <= maxX) {
            drawLine(Color.White.copy(alpha = 0.30f), Offset(screenX(gx), 0f), Offset(screenX(gx), size.height))
            gx += step
        }
        var gy = floor(minY / step) * step
        while (gy <= maxY) {
            drawLine(Color.White.copy(alpha = 0.30f), Offset(0f, screenY(gy)), Offset(size.width, screenY(gy)))
            gy += step
        }
        if (minX <= 0.0 && maxX >= 0.0)
            drawLine(Color.Black.copy(alpha = 0.45f), Offset(screenX(0.0), 0f), Offset(screenX(0.0), size.height), 2f)
        if (minY <= 0.0 && maxY >= 0.0)
            drawLine(Color.Black.copy(alpha = 0.45f), Offset(0f, screenY(0.0)), Offset(size.width, screenY(0.0)), 2f)
        segments.forEach { segment ->
            if (segment.points.size < 2) return@forEach
            val path = Path()
            segment.points.forEachIndexed { index, point ->
                val screen = Offset(screenX(point.x), screenY(point.y))
                if (index == 0) path.moveTo(screen.x, screen.y) else path.lineTo(screen.x, screen.y)
            }
            drawPath(path, primaryColor, style = Stroke(width = 3f, cap = StrokeCap.Round))
        }
    }
}

private fun niceStep(raw: Double): Double {
    val safe = raw.coerceAtLeast(1e-9)
    val exponent = floor(log10(safe))
    val fraction = safe / 10.0.pow(exponent)
    val nice = when {
        fraction < 1.5 -> 1.0
        fraction < 3.0 -> 2.0
        fraction < 7.0 -> 5.0
        else -> 10.0
    }
    return nice * 10.0.pow(exponent)
}

@Composable
private fun SearchPanel(backdrop: LayerBackdrop) {
    val engine = remember { CoreRsPlatform.createSearch() }
    DisposableEffect(Unit) { onDispose { engine.close() } }
    var query by remember { mutableStateOf("明月") }
    var text by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<List<SearchResult>>(emptyList()) }
    var message by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        listOf(
            "床前明月光",
            "明月几时有",
            "海上生明月",
            "举杯邀明月",
        ).forEachIndexed { index, value ->
            runCatching { engine.add(index.toLong() + 1L, value) }
        }
    }

    fun addDocument() {
        val id = System.nanoTime().coerceAtLeast(1L)
        runCatching {
            engine.add(id, text)
            text = ""
            message = "已添加，当前 " + engine.size() + " 条"
        }.onFailure { message = it.message }
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(Modifier.padding(18.dp)) {
            Text("SearchEngine", style = MaterialTheme.typography.titleLarge)
            Text("add · remove · clear · size · ranked search", style = MaterialTheme.typography.bodySmall)
            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    text, { text = it }, Modifier.weight(1f), singleLine = true,
                    label = { Text("新增文档") },
                )
                LiquidButton(onClick = ::addDocument, backdrop = backdrop, isInteractive = text.isNotBlank()) { Text("添加") }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    query, { query = it }, Modifier.weight(1f), singleLine = true,
                    label = { Text("搜索") },
                )
                LiquidButton(onClick = {
                    runCatching { engine.search(query, 20) }
                        .onSuccess { results = it }
                        .onFailure { message = it.message }
                }, backdrop = backdrop) { Text("搜索") }
                LiquidButton(onClick = {
                    engine.clear()
                    results = emptyList()
                    message = "已清空"
                }, backdrop = backdrop) { Text("Clear") }
            }
            Text("documents: " + engine.size(), style = MaterialTheme.typography.labelMedium)
            message?.let { Text(it, color = MaterialTheme.colorScheme.primary) }
            HorizontalDivider(Modifier.padding(vertical = 8.dp))
            if (results.isEmpty()) {
                Text("暂无搜索结果", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                LazyColumn(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    items(results, key = { it.id }) { item ->
                        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Column(Modifier.weight(1f)) {
                                Text(item.text)
                                Text(
                                    "id=" + item.id + "  score=" + "%.3f".format(item.score),
                                    style = MaterialTheme.typography.labelSmall,
                                )
                            }
                            LiquidButton(onClick = {
                                runCatching { engine.remove(item.id) }
                                    .onSuccess { results = engine.search(query) }
                                    .onFailure { message = it.message }
                            }, backdrop = backdrop) { Text("删除") }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun BinaryPanel(backdrop: LayerBackdrop) {
    val sampleZipPath = CoreRsPlatform.sampleZipPath
    val apkPath = CoreRsPlatform.apkPath
    var path by remember { mutableStateOf(sampleZipPath) }
    var entries by remember { mutableStateOf<List<ZipEntryInfo>>(emptyList()) }
    var containsName by remember { mutableStateOf("AndroidManifest.xml") }
    var containsResult by remember { mutableStateOf<Boolean?>(null) }
    var apkResult by remember { mutableStateOf<Boolean?>(null) }
    var apkEntries by remember { mutableStateOf<List<ZipEntryInfo>>(emptyList()) }
    var message by remember { mutableStateOf<String?>(null) }

    fun ensureSampleZip() {
        message = CoreRsPlatform.createSampleZip()
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item {
                Text("Binary / APK", style = MaterialTheme.typography.titleLarge)
                Text("ZipReader · ApkReader · entries · contains · isApk", style = MaterialTheme.typography.bodySmall)
            }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    LiquidButton(onClick = ::ensureSampleZip) { Text("生成示例 ZIP") }
                    LiquidButton(onClick = { path = sampleZip.absolutePath }, backdrop = backdrop) { Text("使用示例") }
                }
            }
            item {
                OutlinedTextField(
                    path, { path = it }, Modifier.fillMaxWidth(), singleLine = true,
                    label = { Text("ZIP 路径") },
                )
            }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    LiquidButton(onClick = {
                        runCatching { entries = CoreRsPlatform.zipEntries(path) }
                            .onFailure { message = it.message }
                    }, backdrop = backdrop) { Text("读取 entries") }
                    LiquidButton(onClick = {
                        runCatching { containsResult = CoreRsPlatform.zipContains(path, containsName) }
                            .onFailure { message = it.message }
                    }, backdrop = backdrop) { Text("contains") }
                }
            }
            item {
                OutlinedTextField(
                    containsName, { containsName = it }, Modifier.fillMaxWidth(), singleLine = true,
                    label = { Text("entry name") },
                )
                containsResult?.let { Text("contains = " + it) }
            }
            item {
                Text("Current app APK", style = MaterialTheme.typography.titleMedium)
                Text(apkPath, style = MaterialTheme.typography.labelSmall)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    LiquidButton(onClick = {
                        runCatching { apkResult = CoreRsPlatform.apkIsApk(apkPath) }
                            .onFailure { message = it.message }
                    }, backdrop = backdrop) { Text("isApk") }
                    LiquidButton(onClick = {
                        runCatching { apkEntries = CoreRsPlatform.apkEntries(apkPath) }
                            .onFailure { message = it.message }
                    }, backdrop = backdrop) { Text("APK entries") }
                }
                apkResult?.let { Text("isApk = " + it) }
            }
            message?.let { msg -> item { Text(msg, color = MaterialTheme.colorScheme.primary) } }
            item { Text("ZIP entries: " + entries.size) }
            items(entries.take(80), key = { it.name }) { entry ->
                Text(
                    (if (entry.isDirectory) "DIR " else "FILE ") + entry.name + "  " + entry.uncompressedSize + " B",
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            item { Text("APK entries: " + apkEntries.size) }
            items(apkEntries.take(80), key = { "apk:" + it.name }) { entry ->
                Text(
                    (if (entry.isDirectory) "DIR " else "FILE ") + entry.name,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}
