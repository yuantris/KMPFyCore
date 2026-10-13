package com.core.fy.kmp.core

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import kotlin.math.roundToInt
import androidx.compose.ui.unit.dp
import com.core.fy.kmp.components.LiquidButton
import com.kyant.backdrop.backdrops.LayerBackdrop
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
expect fun ImageStitchPicker(onImagesSelected: (List<String>) -> Unit)

@Composable
expect fun ImageStitchPreview(path: String, modifier: Modifier = Modifier)

@Composable
expect fun defaultStitchOutputPath(): String

@Composable
fun ImageStitchPanel(backdrop: LayerBackdrop) {
    var paths by remember { mutableStateOf<List<String>>(emptyList()) }
    val defaultOutputPath = defaultStitchOutputPath()
    var outputPath by remember { mutableStateOf(defaultOutputPath) }
    var mode by remember { mutableStateOf(ImageStitchMode.Scroll) }
    var modeMenu by remember { mutableStateOf(false) }
    var autoFixedBars by remember { mutableStateOf(true) }
    var busy by remember { mutableStateOf(false) }
    var calibrating by remember { mutableStateOf(false) }
    var selectedSeam by remember { mutableStateOf<Int?>(null) }
    var movingLayer by remember { mutableStateOf("bottom") }
    var confirmedSeams by remember { mutableStateOf<Set<Int>>(emptySet()) }
    var manualCrops by remember { mutableStateOf<List<ImageStitchCrop>>(emptyList()) }
    var seamOffsets by remember { mutableStateOf<List<Int>>(emptyList()) }
    var imageOffsets by remember { mutableStateOf<List<Int>>(emptyList()) }
    var previewPath by remember { mutableStateOf<String?>(null) }
    var result by remember { mutableStateOf<ImageStitchResult?>(null) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item {
                Text("图片拼接", style = MaterialTheme.typography.titleLarge)
                Text(
                    "长截图匹配 · 固定栏检测 · 重复内容置信度校验",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    ImageStitchPicker { selected ->
                        paths = (paths + selected).distinct()
                        manualCrops = List((paths + selected).distinct().size) { ImageStitchCrop() }
                        seamOffsets = List((paths + selected).distinct().size - 1) { 0 }; imageOffsets = List((paths + selected).distinct().size) { 0 }
                        previewPath = null
                        calibrating = false
                        selectedSeam = null
                        confirmedSeams = emptySet()
                        result = null
                        message = "已添加 ${selected.size} 张图片"
                    }
                    LiquidButton(
                        onClick = { paths = emptyList(); manualCrops = emptyList(); seamOffsets = emptyList(); imageOffsets = emptyList(); previewPath = null; calibrating = false; selectedSeam = null; confirmedSeams = emptySet(); result = null; message = null },
                        backdrop = backdrop,
                    ) { Text("清空") }
                    Spacer(Modifier.weight(1f))
                    Text("${paths.size} 张", style = MaterialTheme.typography.labelLarge)
                }
            }
            if (paths.isEmpty()) {
                item {
                    Surface(
                        Modifier.fillMaxWidth().height(100.dp),
                        shape = RoundedCornerShape(14.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.45f),
                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Text("选择两张或更多连续截图开始拼接")
                        }
                    }
                }
            } else {
                itemsIndexed(paths, key = { index, path -> "$index:$path" }) { index, path ->
                    Row(
                        Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        ImageStitchPreview(path, Modifier.size(56.dp).clip(RoundedCornerShape(8.dp)))
                        Column(Modifier.weight(1f)) {
                            Text("${index + 1}. ${path.substringAfterLast('/').substringAfterLast('\\')}", maxLines = 1)
                            Text(path, style = MaterialTheme.typography.labelSmall, maxLines = 1, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            TextButton(enabled = index > 0 && !busy, onClick = {
                                paths = paths.toMutableList().also { list ->
                                    val item = list.removeAt(index)
                                    list.add(index - 1, item)
                                }
                                manualCrops = List(paths.size) { ImageStitchCrop() }
                                imageOffsets = List(paths.size) { 0 }
                                seamOffsets = List((paths.size - 1).coerceAtLeast(0)) { 0 }
                                confirmedSeams = emptySet()
                                selectedSeam = null
                                result = null
                            }) { Text("↑") }
                            TextButton(enabled = index < paths.lastIndex && !busy, onClick = {
                                paths = paths.toMutableList().also { list ->
                                    val item = list.removeAt(index)
                                    list.add(index + 1, item)
                                }
                                manualCrops = List(paths.size) { ImageStitchCrop() }
                                result = null
                            }) { Text("↓") }
                        }
                        TextButton(enabled = !busy, onClick = {
                            paths = paths.toMutableList().also { it.removeAt(index) }
                            manualCrops = manualCrops.filterIndexed { i, _ -> i != index }
                            imageOffsets = List(paths.size) { 0 }
                            seamOffsets = List((paths.size - 1).coerceAtLeast(0)) { 0 }
                            confirmedSeams = emptySet()
                            selectedSeam = null
                            result = null
                        }) { Text("移除") }
                    }
                }
            }
            item {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("模式")
                    Box {
                        OutlinedButton(onClick = { modeMenu = true }, enabled = !busy) { Text(mode.name) }
                        DropdownMenu(expanded = modeMenu, onDismissRequest = { modeMenu = false }) {
                            listOf(ImageStitchMode.Scroll, ImageStitchMode.Vertical, ImageStitchMode.Horizontal, ImageStitchMode.Grid, ImageStitchMode.Auto).forEach {
                                DropdownMenuItem(text = { Text(it.name) }, onClick = { mode = it; modeMenu = false; result = null })
                            }
                        }
                    }
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = autoFixedBars, onCheckedChange = { autoFixedBars = it }, enabled = !busy)
                    Column {
                        Text("自动识别固定顶部/底部栏")
                        Text("识别状态栏、导航栏等固定区域，并从长图中剔除", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            if (calibrating) {
                item {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("校准拼接", style = MaterialTheme.typography.headlineSmall)
                        Text("点击长图中的接缝，再选择移动上方或下方图片。每条接缝单独确认。", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Surface(shape = RoundedCornerShape(50), color = MaterialTheme.colorScheme.secondaryContainer) {
                                Text("${confirmedSeams.size} / ${paths.size - 1} 已确认", Modifier.padding(horizontal = 12.dp, vertical = 7.dp), style = MaterialTheme.typography.labelLarge)
                            }
                            Spacer(Modifier.weight(1f))
                            TextButton(onClick = { selectedSeam = (0 until paths.size - 1).firstOrNull { it !in confirmedSeams } }) { Text("下一条接缝") }
                        }
                    }
                }
                item {
                    val preview = previewPath
                    val stitched = result
                    if (preview != null && stitched != null) {
                        BoxWithConstraints(Modifier.fillMaxWidth()) {
                            val imageHeight = (maxWidth * stitched.height.toFloat() / stitched.width.coerceAtLeast(1)).coerceAtMost(620.dp)
                            Box(Modifier.fillMaxWidth().height(imageHeight).clip(RoundedCornerShape(24.dp))) {
                                ImageStitchPreview(preview, Modifier.fillMaxSize())
                                val active = selectedSeam
                                if (active != null && active < stitched.segments.size - 1) {
                                    val seamY = (imageHeight * stitched.segments[active + 1].y.toFloat() / stitched.height.coerceAtLeast(1)).coerceIn(8.dp, imageHeight - 8.dp)
                                    val hintMotion = rememberInfiniteTransition(label = "seamDragHint")
                                    val hintShift by hintMotion.animateFloat(
                                        initialValue = 0f,
                                        targetValue = 7f,
                                        animationSpec = infiniteRepeatable(tween(620), repeatMode = RepeatMode.Reverse),
                                        label = "seamDragHintOffset",
                                    )
                                    Column(
                                        Modifier.align(Alignment.TopCenter).offset(y = (seamY - 58.dp + hintShift.dp).coerceAtLeast(0.dp))
                                            .clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.surface.copy(alpha = 0.94f))
                                            .padding(horizontal = 12.dp, vertical = 5.dp),
                                        horizontalAlignment = Alignment.CenterHorizontally,
                                    ) {
                                        Text("↑", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary)
                                        Text(if (movingLayer == "top") "拖动上图" else "上图", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurface)
                                    }
                                    Column(
                                        Modifier.align(Alignment.TopCenter).offset(y = (seamY + 12.dp - hintShift.dp).coerceAtMost(imageHeight - 50.dp))
                                            .clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.surface.copy(alpha = 0.94f))
                                            .padding(horizontal = 12.dp, vertical = 5.dp),
                                        horizontalAlignment = Alignment.CenterHorizontally,
                                    ) {
                                        Text(if (movingLayer == "bottom") "拖动下图" else "下图", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurface)
                                        Text("↓", style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary)
                                    }
                                    val topSelected = movingLayer == "top"
                                    val regionY = if (topSelected) 0.dp else seamY
                                    val regionHeight = (if (topSelected) seamY else imageHeight - seamY).coerceAtLeast(1.dp)
                                    Box(
                                        Modifier.fillMaxWidth().offset(y = regionY).height(regionHeight)
                                            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.055f))
                                            .pointerInput(active, movingLayer, busy, imageHeight, stitched.height) {
                                                var drag = 0f
                                                detectDragGestures(
                                                    onDragStart = { drag = 0f },
                                                    onDragEnd = {
                                                        val delta = (drag * stitched.height / imageHeight.toPx()).roundToInt()
                                                        if (delta != 0 && !busy) {
                                                            val next = imageOffsets.toMutableList().ifEmpty { MutableList(paths.size) { 0 } }
                                                            val targetImage = if (movingLayer == "top") active else active + 1
                                                            if (targetImage in next.indices) next[targetImage] = (next[targetImage] + delta).coerceIn(-600, 600)
                                                            imageOffsets = next
                                                            busy = true
                                                            scope.launch {
                                                                val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars, manualCrops = manualCrops, seamOffsets = seamOffsets, imageOffsets = next)
                                                                runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(paths, preview, config) } }
                                                                    .onSuccess { result = it; message = "接缝已更新，可以继续微调" }
                                                                    .onFailure { message = it.message ?: "预览更新失败" }
                                                                busy = false
                                                            }
                                                        }
                                                    },
                                                    onDrag = { change, amount -> change.consume(); drag += amount.y },
                                                )
                                            },
                                    )
                                }
                                stitched.segments.drop(1).forEachIndexed { seamIndex, segment ->
                                    val y = (imageHeight * segment.y.toFloat() / stitched.height.coerceAtLeast(1)).coerceIn(10.dp, imageHeight - 10.dp)
                                    val selected = selectedSeam == seamIndex
                                    val scale by animateFloatAsState(if (selected) 1f else 0.84f, animationSpec = spring(dampingRatio = 0.68f), label = "seamHandle")
                                    Column(
                                        Modifier.fillMaxWidth().offset(y = y - 24.dp).height(48.dp)
                                            .clickable(enabled = !busy) { selectedSeam = seamIndex },
                                        horizontalAlignment = Alignment.CenterHorizontally,
                                        verticalArrangement = Arrangement.Center,
                                    ) {
                                        Box(Modifier.fillMaxWidth().height(if (selected) 2.dp else 1.dp).background(if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.5f)))
                                        Row(
                                            Modifier.padding(top = 2.dp).graphicsLayer { scaleX = scale; scaleY = scale }
                                                .clip(RoundedCornerShape(50))
                                                .background(if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surface.copy(alpha = 0.96f))
                                                .padding(horizontal = 14.dp, vertical = 7.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                                        ) {
                                            Text(if (seamIndex in confirmedSeams) "✓" else "✣", style = MaterialTheme.typography.labelMedium, color = if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary)
                                            Text("接缝 ${seamIndex + 1}", style = MaterialTheme.typography.labelMedium, color = if (selected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface)
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        Surface(Modifier.fillMaxWidth().height(180.dp), shape = RoundedCornerShape(24.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Box(contentAlignment = Alignment.Center) { CircularProgressIndicator() }
                        }
                    }
                }
                item {
                    val active = selectedSeam
                    AnimatedVisibility(visible = active != null, enter = fadeIn() + slideInVertically { it / 3 }, exit = fadeOut() + slideOutVertically { it / 4 }) {
                        Column(
                            Modifier.fillMaxWidth().clip(RoundedCornerShape(24.dp)).background(MaterialTheme.colorScheme.surfaceContainerHigh).padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Column(Modifier.weight(1f)) {
                                    Text("接缝 ${(active ?: 0) + 1} · 精细调整", style = MaterialTheme.typography.titleMedium)
                                    Text("拖动哪张图，就只调整哪张图的位置；接缝裁切保持独立。", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                Text("↕", style = MaterialTheme.typography.headlineSmall, color = MaterialTheme.colorScheme.primary)
                            }
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                FilterChip(selected = movingLayer == "top", onClick = { movingLayer = "top" }, label = { Text("移动上图") }, modifier = Modifier.weight(1f))
                                FilterChip(selected = movingLayer == "bottom", onClick = { movingLayer = "bottom" }, label = { Text("移动下图") }, modifier = Modifier.weight(1f))
                            }
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text("拖动长图中接缝对应的一侧；松手后自动刷新预览。", Modifier.weight(1f), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                FilledIconButton(
                                    onClick = {
                                        val updatedConfirmed = active?.let { confirmedSeams + it } ?: confirmedSeams
                                        confirmedSeams = updatedConfirmed
                                        selectedSeam = (0 until paths.size - 1).firstOrNull { it !in updatedConfirmed }
                                    },
                                    enabled = !busy && active != null,
                                ) { Text("✓", style = MaterialTheme.typography.titleLarge) }
                            }
                        }
                    }
                }
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        OutlinedButton(onClick = { calibrating = false; selectedSeam = null }, enabled = !busy, modifier = Modifier.weight(1f)) { Text("退出校准") }
                        Button(
                            onClick = {
                                busy = true
                                val selectedPaths = paths.toList()
                                val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars, manualCrops = manualCrops, seamOffsets = seamOffsets, imageOffsets = imageOffsets)
                                scope.launch {
                                    runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(selectedPaths, outputPath, config) } }
                                        .onSuccess { result = it; calibrating = false; selectedSeam = null; message = "已导出长图 · ${it.width} × ${it.height}" }
                                        .onFailure { message = it.message ?: "导出失败" }
                                    busy = false
                                }
                            },
                            enabled = paths.size >= 2 && outputPath.isNotBlank() && !busy,
                            modifier = Modifier.weight(1.4f),
                        ) { Text("✓"); Spacer(Modifier.width(8.dp)); Text(if (busy) "正在生成…" else "确认并导出") }
                    }
                }
            }
            item {
                OutlinedTextField(
                    value = outputPath,
                    onValueChange = { outputPath = it },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = !busy,
                    label = { Text("导出文件路径") },
                    singleLine = true,
                )
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Button(
                        enabled = paths.size >= 2 && !busy && mode != ImageStitchMode.Panorama,
                        onClick = {
                            manualCrops = List(paths.size) { ImageStitchCrop() }
                            seamOffsets = List(paths.size - 1) { 0 }
                            imageOffsets = List(paths.size) { 0 }
                            calibrating = true
                            selectedSeam = 0
                            confirmedSeams = emptySet()
                            result = null
                            previewPath = "${outputPath}.preview.png"
                            message = "正在生成自动拼接预览…"
                            busy = true
                            val selectedPaths = paths.toList()
                            val previewOutput = "${outputPath}.preview.png"
                            val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars)
                            scope.launch {
                                runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(selectedPaths, previewOutput, config) } }
                                    .onSuccess { result = it; message = "预览已生成，点击接缝开始校准" }
                                    .onFailure { message = it.message ?: "自动拼接预览失败"; calibrating = false }
                                busy = false
                            }
                        },
                    ) { Text("自动拼接并校准") }
                    if (busy) {
                        CircularProgressIndicator(Modifier.size(22.dp), strokeWidth = 2.dp)
                    }
                }
                message?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary) }
            }
            result?.let { stitched ->
                item {
                    HorizontalDivider()
                    Text("拼接预览", style = MaterialTheme.typography.titleMedium)
                    Text("${stitched.width} × ${stitched.height}", style = MaterialTheme.typography.labelMedium)
                    ImageStitchPreview(outputPath, Modifier.fillMaxWidth().heightIn(max = 420.dp).clip(RoundedCornerShape(12.dp)))
                    Text("已写入：$outputPath", style = MaterialTheme.typography.labelSmall)
                    Text("匹配置信度：" + stitched.segments.drop(1).joinToString { "${(it.confidence * 100).toInt()}%" }, style = MaterialTheme.typography.labelSmall)
                }
            }
        }
    }
}
