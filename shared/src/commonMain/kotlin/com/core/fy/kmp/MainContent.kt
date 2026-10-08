package com.core.fy.kmp

import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import com.core.fy.kmp.destinations.AdaptiveLuminanceGlassContent
import com.core.fy.kmp.destinations.BottomTabsContent
import com.core.fy.kmp.destinations.ButtonsContent
import com.core.fy.kmp.destinations.ControlCenterContent
import com.core.fy.kmp.destinations.DialogContent
import com.core.fy.kmp.destinations.GlassPlaygroundContent
import com.core.fy.kmp.destinations.HomeContent
import com.core.fy.kmp.destinations.LazyScrollContainerContent
import com.core.fy.kmp.destinations.LockScreenContent
import com.core.fy.kmp.destinations.MagnifierContent
import com.core.fy.kmp.destinations.ProgressiveBlurContent
import com.core.fy.kmp.destinations.ScrollContainerContent
import com.core.fy.kmp.destinations.SliderContent
import com.core.fy.kmp.destinations.ToggleContent
import com.core.fy.kmp.utils.BackHandler

@Composable
fun MainContent() {
    val isLightTheme = !isSystemInDarkTheme()

    CompositionLocalProvider(
        LocalIndication provides ripple(color = if (isLightTheme) Color.Black else Color.White)
    ) {
        var destination by rememberSaveable { mutableStateOf(CatalogDestination.Home) }

        when (destination) {
            CatalogDestination.Home -> HomeContent(onNavigate = { destination = it })

            CatalogDestination.Buttons -> ButtonsContent()
            CatalogDestination.Toggle -> ToggleContent()
            CatalogDestination.Slider -> SliderContent()
            CatalogDestination.BottomTabs -> BottomTabsContent()
            CatalogDestination.Dialog -> DialogContent()

            CatalogDestination.LockScreen -> LockScreenContent()
            CatalogDestination.ControlCenter -> ControlCenterContent()
            CatalogDestination.Magnifier -> MagnifierContent()

            CatalogDestination.GlassPlayground -> GlassPlaygroundContent()
            CatalogDestination.AdaptiveLuminanceGlass -> AdaptiveLuminanceGlassContent()
            CatalogDestination.ProgressiveBlur -> ProgressiveBlurContent()
            CatalogDestination.ScrollContainer -> ScrollContainerContent()
            CatalogDestination.LazyScrollContainer -> LazyScrollContainerContent()
        }

        BackHandler(destination != CatalogDestination.Home) {
            destination = CatalogDestination.Home
        }
    }
}
