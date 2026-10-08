package io.core.rs

object Expression {
    fun eval(expression: String): Double {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionEval(expression)
    }

    fun eval(expression: String, x: Double): Double {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionEvalX(expression, x)
    }

    fun containsVariable(expression: String): Boolean {
        require(expression.isNotBlank()) { "expression must not be blank" }
        return CoreRsNative.nativeExpressionContainsVariable(expression)
    }
}
