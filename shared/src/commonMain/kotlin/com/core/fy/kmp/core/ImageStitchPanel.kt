package com.core.fy.kmp.core

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
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
                        calibrating = false
                        result = null
                        message = "已添加 ${selected.size} 张图片"
                    }
                    LiquidButton(
                        onClick = { paths = emptyList(); manualCrops = emptyList(); calibrating = false; result = null; message = null },
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
                    Text("手动校准裁切", style = MaterialTheme.typography.titleMedium)
                    Text(
                        "逐张调整顶部和底部裁切。滑块单位为原图像素；红框等固定导航区域可手动排除。调整后先确认，再生成最终长图。",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                itemsIndexed(paths, key = { index, path -> "crop:$index:$path" }) { index, path ->
                    val crop = manualCrops.getOrElse(index) { ImageStitchCrop() }
                    Surface(
                        Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.35f),
                        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
                    ) {
                        Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                ImageStitchPreview(path, Modifier.width(56.dp).height(90.dp).clip(RoundedCornerShape(6.dp)))
                                Column(Modifier.weight(1f)) {
                                    Text("第 ${index + 1} 张截图", style = MaterialTheme.typography.titleSmall)
                                    Text("顶部裁切 ${crop.top}px · 底部裁切 ${crop.bottom}px", style = MaterialTheme.typography.labelSmall)
                                }
                                TextButton(onClick = {
                                    manualCrops = manualCrops.toMutableList().also { it[index] = ImageStitchCrop() }
                                }, enabled = !busy) { Text("重置") }
                            }
                            Text("裁切顶部：${crop.top}px", style = MaterialTheme.typography.labelMedium)
                            Slider(
                                value = crop.top.toFloat(),
                                onValueChange = { value ->
                                    manualCrops = manualCrops.toMutableList().also {
                                        it[index] = crop.copy(top = value.toInt())
                                    }
                                    result = null
                                },
                                valueRange = 0f..600f,
                                steps = 59,
                                enabled = !busy,
                            )
                            Text("裁切底部：${crop.bottom}px", style = MaterialTheme.typography.labelMedium)
                            Slider(
                                value = crop.bottom.toFloat(),
                                onValueChange = { value ->
                                    manualCrops = manualCrops.toMutableList().also {
                                        it[index] = crop.copy(bottom = value.toInt())
                                    }
                                    result = null
                                },
                                valueRange = 0f..600f,
                                steps = 59,
                                enabled = !busy,
                            )
                        }
                    }
                }
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedButton(onClick = { calibrating = false }, enabled = !busy) { Text("返回") }
                        Button(
                            enabled = paths.size >= 2 && outputPath.isNotBlank() && !busy,
                            onClick = {
                                busy = true
                                result = null
                                message = "正在根据校准参数生成最终图片…"
                                val selectedPaths = paths.toList()
                                val selectedMode = mode
                                val selectedOutput = outputPath
                                val selectedCrops = manualCrops.toList()
                                val config = ImageStitchConfig(
                                    mode = selectedMode,
                                    autoDetectFixedBars = autoFixedBars,
                                    manualCrops = selectedCrops,
                                )
                                scope.launch {
                                    runCatching {
                                        withContext(Dispatchers.IO) {
                                            CoreRsPlatform.stitchFiles(selectedPaths, selectedOutput, config)
                                        }
                                    }.onSuccess {
                                        result = it
                                        calibrating = false
                                        message = "已确认并生成：${it.width} × ${it.height}"
                                    }.onFailure {
                                        message = it.message ?: "拼接失败"
                                    }
                                    busy = false
                                }
                            },
                        ) { Text(if (busy) "生成中…" else "确认并生成最终图片") }
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
                            calibrating = true
                            result = null
                            message = "请检查每张截图的裁切范围，确认后生成最终图片"
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
