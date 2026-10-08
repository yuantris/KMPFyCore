package io.corer.rs

object Expression {
    fun eval(expression: String): Double {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionEval(expression)
    }
}
