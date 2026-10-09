package com.core.fy.kmp.core

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.kyant.backdrop.backdrops.LayerBackdrop
import com.core.fy.kmp.BackdropDemoScaffold
import com.core.fy.kmp.components.LiquidBottomTab
import com.core.fy.kmp.components.LiquidBottomTabs
import com.core.fy.kmp.components.LiquidButton
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.coroutines.cancellation.CancellationException
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.pow

private enum class CoreTab(val title: String, val symbol: String) {
    Math("数学", "ƒ"), Graph("绘图", "∿"), Search("搜索", "⌕"), Binary("文件", "zip")
}

@Composable
fun CorePlaygroundContent() {
    var tabIndex by remember { mutableIntStateOf(0) }

    BackdropDemoScaffold(Modifier.fillMaxSize()) { backdrop ->
        Column(
            Modifier
                .fillMaxSize()
                .navigationBarsPadding()
                .padding(horizontal = 16.dp)
                .padding(top = 42.dp, bottom = 82.dp)
        ) {
            Text("Core RS", style = MaterialTheme.typography.headlineMedium)
            Text(
                "Rust Native Playground",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.65f),
            )
            Spacer(Modifier.height(16.dp))
            Box(Modifier.weight(1f).fillMaxWidth()) {
                when (CoreTab.entries[tabIndex]) {
                    CoreTab.Math -> MathPanel(backdrop)
                    CoreTab.Graph -> GraphPanel(backdrop)
                    CoreTab.Search -> SearchPanel(backdrop)
                    CoreTab.Binary -> BinaryPanel(backdrop)
                }
            }
            Spacer(Modifier.height(10.dp))
            LiquidBottomTabs(
                selectedTabIndex = { tabIndex },
                onTabSelected = { tabIndex = it },
                backdrop = backdrop,
                tabsCount = CoreTab.entries.size,
                modifier = Modifier.fillMaxWidth(),
            ) {
                CoreTab.entries.forEachIndexed { index, item ->
                    LiquidBottomTab(onClick = { tabIndex = index }) {
                        Text(item.symbol, style = MaterialTheme.typography.titleMedium)
                        Text(item.title, style = MaterialTheme.typography.labelSmall)
                    }
                }
            }
        }
    }
}

@Composable
private fun MathPanel(backdrop: LayerBackdrop) {
    CalculatorPanel(backdrop)
}

private data class Viewport(
    val minX: Double,
    val maxX: Double,
    val minY: Double,
    val maxY: Double,
) {
    val width: Double get() = maxX - minX
    val height: Double get() = maxY - minY

    fun transformed(
        panX: Double,
        panY: Double,
        scale: Double,
        anchorX: Double = 0.5,
        anchorY: Double = 0.5,
    ): Viewport {
        val oldWidth = width
        val oldHeight = height
        val newWidth = (oldWidth * scale).coerceIn(0.05, 200.0)
        val newHeight = (oldHeight * scale).coerceIn(0.05, 200.0)
        val anchorWorldX = minX + oldWidth * anchorX.coerceIn(0.0, 1.0)
        val anchorWorldY = maxY - oldHeight * anchorY.coerceIn(0.0, 1.0)
        val panWorldX = panX * oldWidth / 500.0
        val panWorldY = panY * oldHeight / 500.0
        val newMinX = anchorWorldX - newWidth * anchorX.coerceIn(0.0, 1.0) - panWorldX
        val newMaxY = anchorWorldY + newHeight * anchorY.coerceIn(0.0, 1.0) + panWorldY
        return Viewport(newMinX, newMinX + newWidth, newMaxY - newHeight, newMaxY)
    }
}

@Composable
private fun GraphPanel(backdrop: LayerBackdrop) {
    data class FunctionItem(val id: Long, val expression: String, val secondExpression: String = "", val enabled: Boolean = true)
    var functions by remember { mutableStateOf(listOf(FunctionItem(1, "sin(x) + 0.2*x"), FunctionItem(2, "0.5*cos(2*x)"))) }
    var nextId by remember { mutableLongStateOf(3) }; var selectedId by remember { mutableLongStateOf(1) }
    var mode by remember { mutableStateOf(GraphMode.Cartesian) }; var modeMenu by remember { mutableStateOf(false) }
    var viewport by remember { mutableStateOf(Viewport(-10.0, 10.0, -6.0, 6.0)) }
    var segments by remember { mutableStateOf<Map<Long, List<GraphSegment>>>(emptyMap()) }
    var analysis by remember { mutableStateOf<GraphAnalysis?>(null) }; var cursor by remember { mutableStateOf<GraphPoint?>(null) }
    var cursorValue by remember { mutableStateOf<Double?>(null) }; var canvasSize by remember { mutableStateOf(IntSize.Zero) }
    var loading by remember { mutableStateOf(false) }; var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val selected = functions.firstOrNull { it.id == selectedId }

    LaunchedEffect(canvasSize, viewport, functions, mode) {
        if (canvasSize.width < 2 || canvasSize.height < 2) return@LaunchedEffect
        delay(60)
        loading = true
        error = null
        runCatching {
            withContext(Dispatchers.Default) {
                functions.filter { it.enabled }.associate { item ->
                    item.id to CoreRsPlatform.sampleGraph(
                        item.expression,
                        viewport.minX, viewport.maxX, viewport.minY, viewport.maxY,
                        1600, canvasSize.width, canvasSize.height,
                        mode, item.secondExpression, -10.0, 10.0,
                    )
                }
            }
        }.onSuccess { segments = it }
            .onFailure { error = it.message }
        loading = false
    }
    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(Modifier.padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("函数绘图器", style = MaterialTheme.typography.titleLarge)
                LiquidButton(onClick = { modeMenu = true }, backdrop = backdrop) { Text(mode.name) }
                DropdownMenu(expanded = modeMenu, onDismissRequest = { modeMenu = false }) { GraphMode.entries.forEach { candidate -> DropdownMenuItem(text = { Text(candidate.name) }, onClick = { mode = candidate; modeMenu = false }) } }
                Spacer(Modifier.weight(1f))
                LiquidButton(onClick = { viewport = Viewport(-10.0, 10.0, -6.0, 6.0); cursor = null; analysis = null }, backdrop = backdrop) { Text("重置") }
                LiquidButton(onClick = { selected?.let { item -> scope.launch { runCatching { withContext(Dispatchers.Default) { CoreRsPlatform.analyzeGraph(item.expression, viewport.minX, viewport.maxX, viewport.minY, viewport.maxY, 2400) } }.onSuccess { analysis = it }.onFailure { error = it.message } } } }, backdrop = backdrop) { Text("分析") }
            }
            Row(Modifier.padding(horizontal = 14.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                functions.forEach { item -> LiquidButton(onClick = { selectedId = item.id }, backdrop = backdrop) { Text(if (item.enabled) "● ${item.expression}" else "○ ${item.expression}") } }
                LiquidButton(onClick = { val item = FunctionItem(nextId++, if (mode == GraphMode.Parametric) "cos(t)" else "x", if (mode == GraphMode.Parametric) "sin(t)" else ""); functions = functions + item; selectedId = item.id }, backdrop = backdrop) { Text("+ 函数") }
            }
            selected?.let { item -> Column(Modifier.padding(horizontal = 14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                OutlinedTextField(item.expression, { v -> functions = functions.map { if (it.id == item.id) it.copy(expression = v) else it } }, Modifier.fillMaxWidth(), singleLine = true, label = { Text(if (mode == GraphMode.Polar) "r(t)" else if (mode == GraphMode.Parametric) "x(t)" else "y=f(x)") })
                if (mode == GraphMode.Parametric) OutlinedTextField(item.secondExpression, { v -> functions = functions.map { if (it.id == item.id) it.copy(secondExpression = v) else it } }, Modifier.fillMaxWidth(), singleLine = true, label = { Text("y(t)") })
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    LiquidButton(onClick = { functions = functions.map { if (it.id == item.id) it.copy(enabled = !it.enabled) else it } }, backdrop = backdrop) { Text(if (item.enabled) "隐藏" else "显示") }
                    LiquidButton(onClick = { functions = functions.filterNot { it.id == item.id }; selectedId = functions.firstOrNull()?.id ?: 0 }, backdrop = backdrop) { Text("删除") }
                    if (loading) CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                }
            } }
            error?.let { Text(it, Modifier.padding(horizontal = 14.dp), color = MaterialTheme.colorScheme.error) }
            Box(Modifier.fillMaxWidth().weight(1f).padding(horizontal = 8.dp).onSizeChanged { canvasSize = it }) {
                GraphCanvas(
                    segments, viewport, cursor, analysis, Modifier.fillMaxSize(),
                    { centroid, dx, dy, scale ->
                        val anchorX = if (canvasSize.width > 0) centroid.x.toDouble() / canvasSize.width else 0.5
                        val anchorY = if (canvasSize.height > 0) centroid.y.toDouble() / canvasSize.height else 0.5
                        viewport = viewport.transformed(dx, dy, scale, anchorX, anchorY)
                    },
                    { point ->
                        cursor = point
                        cursorValue = selected?.let { if (mode == GraphMode.Cartesian) CoreRsPlatform.eval(it.expression, point.x) else null }
                    },
                )
            }
            cursor?.let { Text("x=" + it.x + "   y=" + it.y + (cursorValue?.let { v -> "   f(x)=" + v } ?: ""), Modifier.padding(horizontal = 14.dp), style = MaterialTheme.typography.labelMedium) }
            analysis?.let { Text("零点: " + it.zeroes.take(8).joinToString { p -> p.x.toString() } + "    极值: " + it.extrema.take(8).joinToString { p -> "(" + p.x + ", " + p.y + ")" }, Modifier.padding(horizontal = 14.dp), style = MaterialTheme.typography.labelSmall) }
        }
    }
}

@Composable
private fun GraphCanvas(
    segmentsByFunction: Map<Long, List<GraphSegment>>,
    viewport: Viewport,
    cursor: GraphPoint?,
    analysis: GraphAnalysis?,
    modifier: Modifier,
    onViewportChange: (Offset, Double, Double, Double) -> Unit,
    onCursor: (GraphPoint) -> Unit,
) {
    val currentPan by rememberUpdatedState(onViewportChange)
    val currentCursor by rememberUpdatedState(onCursor)
    val primary = MaterialTheme.colorScheme.primary
    val errorColor = MaterialTheme.colorScheme.error

    Box(
        modifier
            .pointerInput(Unit) {
                detectTransformGestures { centroid, pan, zoom, _ ->
                    currentPan(
                        centroid,
                        pan.x.toDouble(),
                        pan.y.toDouble(),
                        1.0 / zoom.coerceIn(0.7f, 1.5f),
                    )
                }
            }
            .pointerInput(viewport) {
                detectTapGestures(
                    onLongPress = { point ->
                        if (size.width > 0 && size.height > 0) {
                            currentCursor(
                                GraphPoint(
                                    viewport.minX + point.x / size.width * viewport.width,
                                    viewport.maxY - point.y / size.height * viewport.height,
                                ),
                            )
                        }
                    },
                    onTap = { point ->
                        if (size.width > 0 && size.height > 0) {
                            currentCursor(
                                GraphPoint(
                                    viewport.minX + point.x / size.width * viewport.width,
                                    viewport.maxY - point.y / size.height * viewport.height,
                                ),
                            )
                        }
                    },
                )
            },
    ) {
        Canvas(Modifier.fillMaxSize()) {
            fun screenX(x: Double): Float =
                ((x - viewport.minX) / viewport.width * size.width).toFloat()

            fun screenY(y: Double): Float =
                ((viewport.maxY - y) / viewport.height * size.height).toFloat()

            val step = niceStep(viewport.width / 10.0)
            var gridX = floor(viewport.minX / step) * step
            while (gridX <= viewport.maxX) {
                drawLine(
                    primary.copy(alpha = 0.10f),
                    Offset(screenX(gridX), 0f),
                    Offset(screenX(gridX), size.height),
                )
                gridX += step
            }

            var gridY = floor(viewport.minY / step) * step
            while (gridY <= viewport.maxY) {
                drawLine(
                    primary.copy(alpha = 0.10f),
                    Offset(0f, screenY(gridY)),
                    Offset(size.width, screenY(gridY)),
                )
                gridY += step
            }

            if (viewport.minX <= 0.0 && viewport.maxX >= 0.0) {
                drawLine(
                    primary.copy(alpha = 0.35f),
                    Offset(screenX(0.0), 0f),
                    Offset(screenX(0.0), size.height),
                    strokeWidth = 2f,
                )
            }
            if (viewport.minY <= 0.0 && viewport.maxY >= 0.0) {
                drawLine(
                    primary.copy(alpha = 0.35f),
                    Offset(0f, screenY(0.0)),
                    Offset(size.width, screenY(0.0)),
                    strokeWidth = 2f,
                )
            }

            segmentsByFunction.values.forEachIndexed { index, segments ->
                val color = primary.copy(
                    alpha = (1f - index * 0.12f).coerceAtLeast(0.35f),
                )
                segments.forEach { segment ->
                    if (segment.points.size >= 2) {
                        val path = Path()
                        segment.points.forEachIndexed { pointIndex, point ->
                            val x = screenX(point.x)
                            val y = screenY(point.y)
                            if (pointIndex == 0) path.moveTo(x, y) else path.lineTo(x, y)
                        }
                        drawPath(
                            path,
                            color,
                            style = Stroke(width = 4f, cap = StrokeCap.Round),
                        )
                    }
                }
            }

            analysis?.zeroes?.forEach { point ->
                drawCircle(primary, 5f, Offset(screenX(point.x), screenY(point.y)))
            }
            analysis?.extrema?.forEach { point ->
                drawCircle(errorColor, 5f, Offset(screenX(point.x), screenY(point.y)))
            }
            cursor?.let { point ->
                drawLine(
                    primary.copy(alpha = 0.45f),
                    Offset(screenX(point.x), 0f),
                    Offset(screenX(point.x), size.height),
                    strokeWidth = 2f,
                )
                drawLine(
                    primary.copy(alpha = 0.45f),
                    Offset(0f, screenY(point.y)),
                    Offset(size.width, screenY(point.y)),
                    strokeWidth = 2f,
                )
                drawCircle(primary, 6f, Offset(screenX(point.x), screenY(point.y)))
            }
        }

        Row(
            Modifier.fillMaxWidth().padding(6.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(viewport.minX.toString(), style = MaterialTheme.typography.labelSmall)
            Text("x", style = MaterialTheme.typography.labelSmall)
            Text(viewport.maxX.toString(), style = MaterialTheme.typography.labelSmall)
        }
        Column(
            Modifier.align(Alignment.CenterStart).padding(6.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(viewport.maxY.toString(), style = MaterialTheme.typography.labelSmall)
            Text("y", style = MaterialTheme.typography.labelSmall)
            Text(viewport.minY.toString(), style = MaterialTheme.typography.labelSmall)
        }
    }
}

private fun niceStep(rawStep: Double): Double {
    if (!rawStep.isFinite() || rawStep <= 0.0) return 1.0
    val magnitude = 10.0.pow(floor(log10(rawStep)))
    val normalized = rawStep / magnitude
    val niceNormalized = when {
        normalized <= 1.0 -> 1.0
        normalized <= 2.0 -> 2.0
        normalized <= 5.0 -> 5.0
        else -> 10.0
    }
    return niceNormalized * magnitude
}

@Composable
private fun SearchPanel(backdrop: LayerBackdrop) {
    val engine = remember { CoreRsPlatform.createSearch() }
    DisposableEffect(Unit) { onDispose { engine.close() } }
    var query by remember { mutableStateOf("明月") }
    var text by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<List<SearchResult>>(emptyList()) }
    var message by remember { mutableStateOf<String?>(null) }
    var nextId by remember { mutableLongStateOf(100L) }

    fun seedSamples() {
        listOf(
            "床前明月光",
            "明月几时有",
            "海上生明月",
            "举杯邀明月",
        ).forEachIndexed { index, value ->
            runCatching { engine.add(index.toLong() + 1L, value) }
        }
    }

    LaunchedEffect(Unit) { seedSamples() }

    fun addDocument() {
        val id = nextId++
        runCatching {
            engine.add(id, text)
            text = ""
            message = "已添加，当前 " + engine.size() + " 条"
        }.onFailure { message = it.message }
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(Modifier.padding(18.dp)) {
            Text("SearchEngine", style = MaterialTheme.typography.titleLarge)
            Text(
                "add · remove · clear · size · ranked search",
                style = MaterialTheme.typography.bodySmall
            )
            Spacer(Modifier.height(12.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    text, { text = it }, Modifier.weight(1f), singleLine = true,
                    label = { Text("新增文档") },
                )
                LiquidButton(
                    onClick = ::addDocument,
                    backdrop = backdrop,
                    tint = Color(0xFFFF8D28),
                    isInteractive = text.isNotBlank()
                ) { Text("添加") }
            }
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    query, { query = it }, Modifier.weight(1f), singleLine = true,
                    label = { Text("搜索") },
                )
                LiquidButton(
                    onClick = {
                        runCatching { engine.search(query, 20) }
                            .onSuccess { results = it }
                            .onFailure { message = it.message }
                    },
                    backdrop = backdrop, tint = Color(0xFFFF8D28),
                ) { Text("搜索") }
                LiquidButton(onClick = {
                    engine.clear()
                    seedSamples()
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
                        Row(
                            Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(Modifier.weight(1f)) {
                                Text(item.text)
                                Text(
                                    "id=" + item.id + "  score=" + item.score.toString(),
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
                Text(
                    "ZipReader · ApkReader · entries · contains · isApk",
                    style = MaterialTheme.typography.bodySmall
                )
            }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    LiquidButton(
                        onClick = ::ensureSampleZip,
                        backdrop = backdrop
                    ) { Text("生成示例 ZIP") }
                    LiquidButton(
                        onClick = { path = sampleZipPath },
                        backdrop = backdrop
                    ) { Text("使用示例") }
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
                        runCatching {
                            containsResult = CoreRsPlatform.zipContains(path, containsName)
                        }
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
