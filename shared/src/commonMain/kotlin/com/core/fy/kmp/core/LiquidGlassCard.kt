package com.core.fy.kmp.core

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.kyant.backdrop.Backdrop
import com.kyant.backdrop.drawBackdrop
import com.kyant.backdrop.effects.blur
import com.kyant.backdrop.effects.lens
import com.kyant.backdrop.effects.vibrancy
import com.kyant.backdrop.highlight.Highlight

private val CoreCardShape = RoundedCornerShape(24.dp)

@Composable
fun LiquidGlassCard(
    backdrop: Backdrop,
    modifier: Modifier = Modifier,
    content: @Composable BoxScope.() -> Unit,
) {
    Box(
        modifier.drawBackdrop(
            backdrop = backdrop,
            shape = { CoreCardShape },
            highlight = { Highlight.Default },
            effects = {
                vibrancy()
                blur(8f.dp.toPx())
                lens(24f.dp.toPx(), 24f.dp.toPx())
            },
            onDrawSurface = { drawRect(Color.White.copy(alpha = 0.12f)) },
        ),
        content = content,
    )
}
