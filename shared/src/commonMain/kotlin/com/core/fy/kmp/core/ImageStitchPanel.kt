package com.core.fy.kmp.core

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.kyant.backdrop.backdrops.LayerBackdrop
import com.core.fy.kmp.components.LiquidButton
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
    var outputPath by remember { mutableStateOf(defaultStitchOutputPath()) }
    var mode by remember { mutableStateOf(ImageStitchMode.Scroll) }
    var modeMenu by remember { mutableStateOf(false) }
    var autoFixedBars by remember { mutableStateOf(true) }
    var busy by remember { mutableStateOf(false) }
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
                    LiquidButton(
                        onClick = { ImageStitchPicker { selected ->
                            paths = (paths + selected).distinct()
                            result = null
                            message = "已添加 ${selected.size} 张图片"
                        } },
                        backdrop = backdrop,
                    ) { Text("选择图片") }
                    LiquidButton(
                        onClick = { paths = emptyList(); result = null; message = null },
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
                            Text("${index + 1}. ${path.substringAfterLast('/').substringAfterLast('\\\\')}", maxLines = 1)
                            Text(path, style = MaterialTheme.typography.labelSmall, maxLines = 1, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            TextButton(enabled = index > 0 && !busy, onClick = {
                                paths = paths.toMutableList().also { list ->
                                    val item = list.removeAt(index)
                                    list.add(index - 1, item)
                                }
                                result = null
                            }) { Text("↑") }
                            TextButton(enabled = index < paths.lastIndex && !busy, onClick = {
                                paths = paths.toMutableList().also { list ->
                                    val item = list.removeAt(index)
                                    list.add(index + 1, item)
                                }
                                result = null
                            }) { Text("↓") }
                        }
                        TextButton(enabled = !busy, onClick = {
                            paths = paths.toMutableList().also { it.removeAt(index) }
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
                        enabled = paths.size >= 2 && outputPath.isNotBlank() && !busy && mode != ImageStitchMode.Panorama,
                        onClick = {
                            busy = true
                            result = null
                            message = "正在匹配并拼接…"
                            val selectedPaths = paths.toList()
                            val selectedMode = mode
                            val selectedOutput = outputPath
                            val config = ImageStitchConfig(mode = selectedMode, autoDetectFixedBars = autoFixedBars)
                            scope.launch {
                                runCatching {
                                    withContext(Dispatchers.IO) {
                                        CoreRsPlatform.stitchFiles(selectedPaths, selectedOutput, config)
                                    }
                                }.onSuccess {
                                    result = it
                                    message = "拼接完成：${it.width} × ${it.height}，${it.imageCount} 张图片"
                                }.onFailure {
                                    message = it.message ?: "拼接失败"
                                }
                                busy = false
                            }
                        },
                    ) { Text(if (busy) "拼接中…" else "拼接并导出") }
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
