package com.core.fy.kmp.core

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
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
    var manualCrops by remember { mutableStateOf<List<ImageStitchCrop>>(emptyList()) }
    var seamOffsets by remember { mutableStateOf<List<Int>>(emptyList()) }
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
                        seamOffsets = List((paths + selected).distinct().size - 1) { 0 }
                        previewPath = null
                        calibrating = false
                        result = null
                        message = "已添加 ${selected.size} 张图片"
                    }
                    LiquidButton(
                        onClick = { paths = emptyList(); manualCrops = emptyList(); seamOffsets = emptyList(); previewPath = null; calibrating = false; result = null; message = null },
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
                    HorizontalDivider()
                    Text("接缝校准", style = MaterialTheme.typography.titleMedium)
                    Text("先查看自动拼接的连续长图。点击接缝，再微调接缝上方或下方的截图；每次调整都会重新生成预览。", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                item {
                    val preview = previewPath
                    if (preview != null) {
                        ImageStitchPreview(preview, Modifier.fillMaxWidth().heightIn(max = 680.dp).clip(RoundedCornerShape(12.dp)))
                    } else {
                        Surface(Modifier.fillMaxWidth().height(180.dp), shape = RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Box(contentAlignment = Alignment.Center) { Text("正在生成自动拼接预览…") }
                        }
                    }
                }
                itemsIndexed(seamOffsets, key = { index, _ -> "seam:$index" }) { index, offset ->
                    var selected by remember { mutableStateOf(index == 0) }
                    Surface(
                        Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        color = if (selected) MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.45f) else MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.25f),
                        border = BorderStroke(1.dp, if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant),
                    ) {
                        Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                TextButton(onClick = { selected = !selected }) {
                                    Text(if (selected) "● 接缝 ${index + 1} 已选中" else "○ 选择接缝 ${index + 1}")
                                }
                                Spacer(Modifier.weight(1f))
                                Text("偏移 ${offset}px", style = MaterialTheme.typography.labelMedium)
                            }
                            Text("${index + 1} ↔ ${index + 2}：${paths[index].substringAfterLast('/').substringAfterLast('\\')} / ${paths[index + 1].substringAfterLast('/').substringAfterLast('\\')}", style = MaterialTheme.typography.labelSmall)
                            if (selected) {
                                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                    OutlinedButton(enabled = !busy, onClick = {
                                        seamOffsets = seamOffsets.toMutableList().also { it[index] = (it[index] - 4).coerceIn(-300, 300) }
                                        result = null
                                        busy = true
                                        scope.launch {
                                            val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars, manualCrops = manualCrops, seamOffsets = seamOffsets)
                                            runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(paths, previewPath ?: "${outputPath}.preview.png", config) } }
                                                .onSuccess { previewPath = "${outputPath}.preview.png"; message = "已更新接缝预览" }
                                                .onFailure { message = it.message ?: "预览生成失败" }
                                            busy = false
                                        }
                                    }) { Text("上方 ↑ 4px") }
                                    OutlinedButton(enabled = !busy, onClick = {
                                        seamOffsets = seamOffsets.toMutableList().also { it[index] = (it[index] + 4).coerceIn(-300, 300) }
                                        result = null
                                        busy = true
                                        scope.launch {
                                            val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars, manualCrops = manualCrops, seamOffsets = seamOffsets)
                                            runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(paths, previewPath ?: "${outputPath}.preview.png", config) } }
                                                .onSuccess { previewPath = "${outputPath}.preview.png"; message = "已更新接缝预览" }
                                                .onFailure { message = it.message ?: "预览生成失败" }
                                            busy = false
                                        }
                                    }) { Text("下方 ↓ 4px") }
                                    TextButton(onClick = { seamOffsets = seamOffsets.toMutableList().also { it[index] = 0 }; result = null }) { Text("重置") }
                                }
                            }
                        }
                    }
                }
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedButton(onClick = { calibrating = false }, enabled = !busy) { Text("返回") }
                        Button(enabled = paths.size >= 2 && outputPath.isNotBlank() && !busy, onClick = {
                            busy = true
                            val selectedPaths = paths.toList()
                            val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars, manualCrops = manualCrops, seamOffsets = seamOffsets)
                            scope.launch {
                                runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(selectedPaths, outputPath, config) } }
                                    .onSuccess { result = it; calibrating = false; message = "已生成最终图片：${it.width} × ${it.height}" }
                                    .onFailure { message = it.message ?: "最终拼接失败" }
                                busy = false
                            }
                        }) { Text(if (busy) "处理中…" else "确认并导出") }
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
                            calibrating = true
                            result = null
                            previewPath = "${outputPath}.preview.png"
                            message = "正在生成自动拼接预览…"
                            busy = true
                            val selectedPaths = paths.toList()
                            val previewOutput = "${outputPath}.preview.png"
                            val config = ImageStitchConfig(mode = mode, autoDetectFixedBars = autoFixedBars)
                            scope.launch {
                                runCatching { withContext(Dispatchers.IO) { CoreRsPlatform.stitchFiles(selectedPaths, previewOutput, config) } }
                                    .onSuccess { message = "自动拼接预览已生成，请选择接缝进行微调" }
                                    .onFailure { message = it.message ?: "自动拼接预览失败"; calibrating = false }
                                busy = false
                            }
                        },
                    ) { Text("调整裁切并预览") }
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
