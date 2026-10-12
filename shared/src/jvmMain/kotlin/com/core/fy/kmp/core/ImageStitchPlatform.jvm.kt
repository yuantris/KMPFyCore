package com.core.fy.kmp.core

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toComposeImageBitmap
import androidx.compose.ui.layout.ContentScale
import java.io.File
import javax.imageio.ImageIO
import javax.swing.JFileChooser
import javax.swing.filechooser.FileNameExtensionFilter

@Composable
actual fun ImageStitchPicker(onImagesSelected: (List<String>) -> Unit) {
    androidx.compose.material3.Button(onClick = {
        val chooser = JFileChooser().apply {
            dialogTitle = "选择要拼接的图片"
            isMultiSelectionEnabled = true
            fileFilter = FileNameExtensionFilter("图片文件", "png", "jpg", "jpeg", "webp", "bmp")
        }
        if (chooser.showOpenDialog(null) == JFileChooser.APPROVE_OPTION) {
            val selected = chooser.selectedFiles.map { it.absolutePath }
            if (selected.isNotEmpty()) onImagesSelected(selected)
        }
    }) { Text("选择图片") }
}

@Composable
actual fun ImageStitchPreview(path: String, modifier: Modifier) {
    val bitmap = remember(path) {
        runCatching { ImageIO.read(File(path))?.toComposeImageBitmap() }.getOrNull()
    }
    if (bitmap != null) {
        Image(bitmap, contentDescription = path, modifier = modifier, contentScale = ContentScale.Fit)
    } else {
        Box(modifier, contentAlignment = Alignment.Center) {
            Text("预览不可用", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
actual fun defaultStitchOutputPath(): String =
    remember { File(System.getProperty("java.io.tmpdir"), "fycore/stitched.png").absolutePath }
