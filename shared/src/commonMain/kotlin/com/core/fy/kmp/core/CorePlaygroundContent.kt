package com.core.fy.kmp.core

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectTransformGestures
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
                                    withContext(Dispatchers.Default) {
                                        CoreRsPlatform.eval(expression)
                                    }
                                }.onSuccess { result = "constant = " + it }
                                    .onFailure { error = it.message }
                            }
                        }, backdrop = backdrop, tint = Color(0xFFFF8D28)) { Text("常量求值") }
                        LiquidButton(onClick = {
                            scope.launch {
                                error = null
                                runCatching {
                                    withContext(Dispatchers.Default) {
                                        CoreRsPlatform.eval(expression, x)
                                    }
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
                    }, backdrop = backdrop, tint = Color(0xFF0088FF)) { Text("检查是否包含 x") }
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

private data class Viewport(
    val minX: Double,
    val maxX: Double,
    val minY: Double,
    val maxY: Double,
) {
    val width: Double get() = maxX - minX
    val height: Double get() = maxY - minY

    fun transformed(dx: Double, dy: Double, scale: Double): Viewport {
        val w = width
        val h = height
        val nw = (w * scale).coerceIn(0.05, 200.0)
        val nh = (h * scale).coerceIn(0.05, 200.0)
        val cx = (minX + maxX) / 2 - dx * w / 500.0
        val cy = (minY + maxY) / 2 + dy * h / 500.0
        return Viewport(cx - nw / 2, cx + nw / 2, cy - nh / 2, cy + nh / 2)
    }
}

@Composable
private fun GraphPanel(backdrop: LayerBackdrop) {
    var expression by remember { mutableStateOf("sin(x) + 0.2 * x") }
    var segments by remember { mutableStateOf<List<GraphSegment>>(emptyList()) }
    var viewport by remember { mutableStateOf(Viewport(-10.0, 10.0, -5.0, 5.0)) }
    var canvasSize by remember { mutableStateOf(IntSize.Zero) }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var refreshKey by remember { mutableIntStateOf(0) }

    // 关键改动：
    // 1) viewport 不再是 LaunchedEffect 的 key——避免拖动时反复取消采样
    // 2) snapshotFlow 把 viewport 变成一个流
    // 3) conflate() 在采样忙时自动丢弃中间值，而不是取消进行中的采样
    // 4) collect() 顺序处理，每次采样都会跑完，跑完后立刻拿最新 viewport 再采
    LaunchedEffect(canvasSize, expression, refreshKey) {
        snapshotFlow { viewport }
            .conflate()
            .collect { vp ->
                if (canvasSize.width < 2 || canvasSize.height < 2) return@collect

                loading = true
                try {
                    val result = withContext(Dispatchers.Default) {
                        CoreRsPlatform.sampleGraph(
                            expression,
                            vp.minX, vp.maxX, vp.minY, vp.maxY,
                            pixelWidth = canvasSize.width,
                            pixelHeight = canvasSize.height,
                        )
                    }
                    segments = result
                    error = null
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Throwable) {
                    error = e.message ?: "绘图失败"
                } finally {
                    // 协程被取消时不要抢写 loading，避免与新协程竞争
                    if (currentCoroutineContext().isActive) {
                        loading = false
                    }
                }
            }
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(
            Modifier.padding(top = 14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Row(
                Modifier.padding(horizontal = 14.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    value = expression,
                    onValueChange = { expression = it },
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                    label = { Text("f(x)") },
                )
                Spacer(Modifier.width(8.dp))
                LiquidButton(
                    onClick = { refreshKey++ },
                    backdrop = backdrop,
                    isInteractive = false,
                    tint = Color(0xFFFF8D28)
                ) {
                    if (loading) {
                        CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                    } else {
                        Text("绘制")
                    }
                }
            }
            error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            GraphCanvas(
                segments = segments,
                viewport = viewport,
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .onSizeChanged { canvasSize = it },
                onViewportChange = { dx, dy, scale ->
                    viewport = viewport.transformed(dx, dy, scale)
                },
            )
        }
    }
}

@Composable
private fun GraphCanvas(
    segments: List<GraphSegment>,
    viewport: Viewport,
    modifier: Modifier,
    onViewportChange: (Double, Double, Double) -> Unit,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val currentOnViewportChange by rememberUpdatedState(onViewportChange)

    Canvas(
        modifier.pointerInput(Unit) {
            detectTransformGestures { _, pan, zoom, _ ->
                currentOnViewportChange(
                    pan.x.toDouble(),
                    pan.y.toDouble(),
                    1.0 / zoom.coerceIn(0.7f, 1.5f),
                )
            }
        }
    ) {
        val minX = viewport.minX
        val maxX = viewport.maxX
        val minY = viewport.minY
        val maxY = viewport.maxY
        if (maxX <= minX || maxY <= minY) return@Canvas

        val sx = size.width / (maxX - minX).toFloat()
        val sy = size.height / (maxY - minY).toFloat()

        fun screenX(x: Double) = ((x - minX) * sx).toFloat()
        fun screenY(y: Double) = (size.height - (y - minY) * sy).toFloat()

        clipRect {
            val step = niceStep((maxX - minX) / 10.0)

            var gx = floor(minX / step) * step
            while (gx <= maxX) {
                drawLine(
                    Color.White.copy(alpha = 0.30f),
                    Offset(screenX(gx), 0f),
                    Offset(screenX(gx), size.height)
                )
                gx += step
            }

            var gy = floor(minY / step) * step
            while (gy <= maxY) {
                drawLine(
                    Color.White.copy(alpha = 0.30f),
                    Offset(0f, screenY(gy)),
                    Offset(size.width, screenY(gy))
                )
                gy += step
            }

            if (minX <= 0.0 && maxX >= 0.0) {
                drawLine(
                    Color.Black.copy(alpha = 0.45f),
                    Offset(screenX(0.0), 0f),
                    Offset(screenX(0.0), size.height),
                    2f
                )
            }
            if (minY <= 0.0 && maxY >= 0.0) {
                drawLine(
                    Color.Black.copy(alpha = 0.45f),
                    Offset(0f, screenY(0.0)),
                    Offset(size.width, screenY(0.0)),
                    2f
                )
            }

            segments.forEach { segment ->
                if (segment.points.size < 2) return@forEach
                val path = Path()
                segment.points.forEachIndexed { index, point ->
                    val screen = Offset(screenX(point.x), screenY(point.y))
                    if (index == 0) {
                        path.moveTo(screen.x, screen.y)
                    } else {
                        path.lineTo(screen.x, screen.y)
                    }
                }
                drawPath(path, primaryColor, style = Stroke(width = 6f, cap = StrokeCap.Round))
            }
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
