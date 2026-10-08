package com.core.fy.kmp.core

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.core.fy.kmp.components.LiquidButton
import com.kyant.backdrop.backdrops.LayerBackdrop
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private data class CalculatorHistoryItem(
    val expression: String,
    val result: String,
    val id: Long,
)

private sealed interface CalculatorKey {
    data class Text(val label: String, val insert: String = label) : CalculatorKey
    data object Clear : CalculatorKey
    data object Delete : CalculatorKey
    data object Equals : CalculatorKey
    data object ToggleSign : CalculatorKey
    data object Square : CalculatorKey
    data object Reciprocal : CalculatorKey
}

@Composable
internal fun CalculatorPanel(backdrop: LayerBackdrop) {
    var mode by remember { mutableStateOf(CalculatorMode.Basic) }
    var angleMode by remember { mutableStateOf(CalculatorAngleMode.Rad) }
    var expression by remember { mutableStateOf("") }
    var result by remember { mutableStateOf<CalculationResult?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var history by remember { mutableStateOf<List<CalculatorHistoryItem>>(emptyList()) }
    var showHistory by remember { mutableStateOf(false) }
    var evaluating by remember { mutableStateOf(false) }
    var memory by remember { mutableStateOf(0.0) }
    var justEvaluated by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun clearResult() { result = null; error = null }
    fun setExpression(value: String) { expression = value; clearResult(); justEvaluated = false }
    fun append(value: String) {
        val continuesResult = justEvaluated && value in listOf("+", "-", "*", "/", "^")
        if (continuesResult) expression = result?.value?.toDisplayNumber() ?: expression + value else if (justEvaluated) expression = ""
        expression += value
        clearResult()
        justEvaluated = false
    }

    fun evaluate() {
        if (expression.isBlank() || evaluating) return
        scope.launch {
            evaluating = true
            error = null
            runCatching {
                withContext(Dispatchers.Default) { CoreRsPlatform.calculate(autoCloseParentheses(expression), mode, angleMode) }
            }.onSuccess { value ->
                result = value
                val display = value.displayText()
                justEvaluated = true
                history = listOf(
                    CalculatorHistoryItem(expression, display, history.maxOfOrNull { it.id }?.plus(1) ?: 1),
                ) + history.filterNot { it.expression == expression }.take(49)
            }.onFailure { error = it.message ?: "计算失败" }
            evaluating = false
        }
    }

    LiquidGlassCard(backdrop, Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("计算器", style = MaterialTheme.typography.titleLarge)
                Spacer(Modifier.weight(1f))
                LiquidButton(onClick = { showHistory = !showHistory }, backdrop = backdrop) {
                    Text(if (showHistory) "键盘" else "历史")
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                CalculatorMode.entries.forEach { item ->
                    LiquidButton(
                        onClick = { mode = item; setExpression("") },
                        backdrop = backdrop,
                        tint = if (item == mode) MaterialTheme.colorScheme.primary else null,
                    ) { Text(item.title()) }
                }
            }

            if (showHistory) {
                HistoryView(
                    history = history,
                    backdrop = backdrop,
                    onSelect = { setExpression(it.expression); showHistory = false },
                    onClear = { history = emptyList() },
                    modifier = Modifier.weight(1f),
                )
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    if (mode == CalculatorMode.Scientific) {
                        LiquidButton(onClick = { angleMode = if (angleMode == CalculatorAngleMode.Rad) CalculatorAngleMode.Deg else CalculatorAngleMode.Rad }, backdrop = backdrop) { Text(if (angleMode == CalculatorAngleMode.Rad) "RAD" else "DEG") }
                    }
                }
                CalculatorDisplay(expression, result, error, evaluating, Modifier.weight(1f), onExpressionChange = ::setExpression)
                if (mode != CalculatorMode.Fraction) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        listOf("MC", "MR", "M+", "M-").forEach { action ->
                            LiquidButton(onClick = {
                                when (action) {
                                    "MC" -> memory = 0.0
                                    "MR" -> append(memory.toDisplayNumber())
                                    "M+" -> result?.let { memory += it.value }
                                    "M-" -> result?.let { memory -= it.value }
                                }
                            }, backdrop = backdrop, modifier = Modifier.weight(1f)) { Text(action) }
                        }
                    }
                }
                CalculatorKeyboard(calculatorKeys(mode), backdrop) { key ->
                    when (key) {
                        CalculatorKey.Clear -> setExpression("")
                        CalculatorKey.Delete -> if (expression.isNotEmpty()) setExpression(expression.dropLast(1))
                        CalculatorKey.Equals -> evaluate()
                        CalculatorKey.ToggleSign -> if (expression.isNotBlank()) setExpression("-(" + expression + ")")
                        CalculatorKey.Square -> if (expression.isNotBlank()) append("^2")
                        CalculatorKey.Reciprocal -> if (expression.isNotBlank()) setExpression("inv(" + expression + ")")
                        is CalculatorKey.Text -> append(key.insert)
                    }
                }
            }
        }
    }
}

@Composable
private fun CalculatorDisplay(
    expression: String,
    result: CalculationResult?,
    error: String?,
    evaluating: Boolean,
    modifier: Modifier,
    onExpressionChange: (String) -> Unit,
) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.Bottom, horizontalAlignment = Alignment.End) {
        OutlinedTextField(value = expression, onValueChange = onExpressionChange, modifier = Modifier.fillMaxWidth(), minLines = 2, maxLines = 4, placeholder = { Text("输入表达式") })
        Spacer(Modifier.height(8.dp))
        when {
            evaluating -> CircularProgressIndicator(Modifier.size(24.dp))
            error != null -> Text(error, color = MaterialTheme.colorScheme.error)
            result != null -> {
                if (result.type == "rational" && result.numerator != null && result.denominator != null) {
                    RationalDisplay(result.numerator, result.denominator)
                    Text("≈ " + result.value.toDisplayNumber(), style = MaterialTheme.typography.bodyMedium)
                } else {
                    Text(result.value.toDisplayNumber(), style = MaterialTheme.typography.displaySmall)
                }
            }
            else -> Text("0", style = MaterialTheme.typography.displaySmall)
        }
    }
}

@Composable
private fun RationalDisplay(numerator: Long, denominator: Long) {
    val negative = numerator < 0
    val absolute = kotlin.math.abs(numerator)
    val whole = absolute / denominator
    val remainder = absolute % denominator
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (negative) Text("−", style = MaterialTheme.typography.displaySmall)
        if (whole != 0L) Text(whole.toString(), style = MaterialTheme.typography.displaySmall)
        if (remainder != 0L) Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(remainder.toString(), style = MaterialTheme.typography.titleLarge)
            HorizontalDivider(Modifier.width(42.dp))
            Text(denominator.toString(), style = MaterialTheme.typography.titleLarge)
        } else if (whole == 0L) Text("0", style = MaterialTheme.typography.displaySmall)
    }
}

@Composable
private fun CalculatorKeyboard(
    keys: List<List<CalculatorKey>>,
    backdrop: LayerBackdrop,
    onKey: (CalculatorKey) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(7.dp)) {
        keys.forEach { row ->
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                row.forEach { key ->
                    LiquidButton(
                        onClick = { onKey(key) },
                        backdrop = backdrop,
                        modifier = Modifier.weight(1f),
                        tint = when (key) {
                            CalculatorKey.Equals -> MaterialTheme.colorScheme.primary
                            CalculatorKey.Clear, CalculatorKey.Delete -> MaterialTheme.colorScheme.secondaryContainer
                            else -> null
                        },
                    ) { Text(key.label()) }
                }
            }
        }
    }
}

@Composable
private fun HistoryView(
    history: List<CalculatorHistoryItem>,
    backdrop: LayerBackdrop,
    onSelect: (CalculatorHistoryItem) -> Unit,
    onClear: () -> Unit,
    modifier: Modifier,
) {
    Column(modifier) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("计算历史", style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.weight(1f))
            LiquidButton(onClick = onClear, backdrop = backdrop) { Text("清空") }
        }
        Spacer(Modifier.height(8.dp))
        if (history.isEmpty()) {
            Text("暂无历史记录", color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                items(history, key = { it.id }) { item ->
                    LiquidGlassCard(backdrop, Modifier.fillMaxWidth()) {
                        Column(Modifier.fillMaxWidth().padding(12.dp)) {
                            Text(item.expression)
                            Text(item.result, style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.primary)
                            LiquidButton(onClick = { onSelect(item) }, backdrop = backdrop) { Text("重新编辑") }
                        }
                    }
                }
            }
        }
    }
}

private fun calculatorKeys(mode: CalculatorMode): List<List<CalculatorKey>> {
    fun t(label: String, insert: String = label) = CalculatorKey.Text(label, insert)
    val basic = listOf(
        listOf(CalculatorKey.Clear, CalculatorKey.Delete, t("%"), t("÷", "/")),
        listOf(t("7"), t("8"), t("9"), t("×", "*")),
        listOf(t("4"), t("5"), t("6"), t("−", "-")),
        listOf(t("1"), t("2"), t("3"), t("+")),
        listOf(t("0"), t("."), CalculatorKey.ToggleSign, CalculatorKey.Equals),
    )
    if (mode == CalculatorMode.Basic) return basic
    if (mode == CalculatorMode.Fraction) {
        return listOf(
            listOf(CalculatorKey.Clear, CalculatorKey.Delete, t("÷", "/"), t("×", "*")),
            listOf(t("7"), t("8"), t("9"), t("−", "-")),
            listOf(t("4"), t("5"), t("6"), t("+")),
            listOf(t("1"), t("2"), t("3"), t("/")),
            listOf(t("0"), t("("), t(")"), CalculatorKey.ToggleSign),
            listOf(t("1/2", "1/2"), t("1/3", "1/3"), t("1/4", "1/4"), CalculatorKey.Equals),
        )
    }
    return listOf(
        listOf(t("sin("), t("cos("), t("tan("), t("√", "sqrt("), t("xʸ", "^")),
        listOf(CalculatorKey.Square, CalculatorKey.Reciprocal, t("!"), t("ln("), t("log(")),
        listOf(t("asin("), t("acos("), t("atan("), t("ln("), t("log(")),
        listOf(t("abs("), t("exp("), t("floor("), t("ceil("), t("π", "pi")),
        listOf(t("e"), t("("), t(")"), t("%"), t("÷", "/")),
        listOf(t("7"), t("8"), t("9"), t("×", "*"), t("−", "-")),
        listOf(t("4"), t("5"), t("6"), t("+"), CalculatorKey.ToggleSign),
        listOf(t("1"), t("2"), t("3"), t("."), CalculatorKey.Equals),
        listOf(t("0"), CalculatorKey.Clear, CalculatorKey.Delete, t(")"), t("(")),
    )
}

private fun CalculatorKey.label(): String = when (this) {
    CalculatorKey.Clear -> "AC"
    CalculatorKey.Delete -> "DEL"
    CalculatorKey.Equals -> "="
    CalculatorKey.ToggleSign -> "±"
    CalculatorKey.Square -> "x²"
    CalculatorKey.Reciprocal -> "1/x"
    is CalculatorKey.Text -> label
}

private fun autoCloseParentheses(input: String): String {
    var balance = 0
    input.forEach { if (it == '(') balance++ else if (it == ')' && balance > 0) balance-- }
    return input + ")".repeat(balance)
}

private fun CalculationResult.displayText(): String =
    if (type == "rational" && numerator != null && denominator != null) {
        numerator.toString() + "/" + denominator
    } else value.toDisplayNumber()

private fun Double.toDisplayNumber(): String =
    if (isFinite() && this % 1.0 == 0.0) toLong().toString() else toString()
