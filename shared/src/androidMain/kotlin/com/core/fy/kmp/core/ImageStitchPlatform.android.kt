package com.core.fy.kmp.core

import android.graphics.BitmapFactory
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import java.io.File

@Composable
actual fun ImageStitchPicker(onImagesSelected: (List<String>) -> Unit) {
    val context = LocalContext.current
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        val paths = uris.mapNotNull { uri ->
            runCatching {
                val extension = context.contentResolver.getType(uri)
                    ?.substringAfter('/', "img")
                    ?.filter { it.isLetterOrDigit() }
                    ?.take(8)
                    ?.ifBlank { "img" } ?: "img"
                val file = File.createTempFile("stitch_", ".$extension", context.cacheDir)
                context.contentResolver.openInputStream(uri)?.use { input ->
                    file.outputStream().use { output -> input.copyTo(output) }
                } ?: error("无法读取图片：$uri")
                file.absolutePath
            }.getOrNull()
        }
        if (paths.isNotEmpty()) onImagesSelected(paths)
    }
    androidx.compose.material3.TextButton(onClick = {
        launcher.launch(arrayOf("image/*"))
    }) { Text("打开系统相册") }
}

@Composable
actual fun ImageStitchPreview(path: String, modifier: Modifier) {
    val bitmap = remember(path) {
        runCatching {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeFile(path, bounds)
            var sampleSize = 1
            while (bounds.outWidth / sampleSize > 1200 || bounds.outHeight / sampleSize > 1600) {
                sampleSize *= 2
            }
            val options = BitmapFactory.Options().apply { inSampleSize = sampleSize }
            BitmapFactory.decodeFile(path, options)?.asImageBitmap()
        }.getOrNull()
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
actual fun defaultStitchOutputPath(): String {
    val context = LocalContext.current
    return remember(context) { File(context.cacheDir, "stitched.png").absolutePath }
}
