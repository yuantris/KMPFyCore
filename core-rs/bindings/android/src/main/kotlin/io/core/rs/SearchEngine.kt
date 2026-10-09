package io.core.rs

import org.json.JSONArray

data class SearchResult(
    val id: Long,
    val score: Float,
    val text: String,
)

class SearchEngine : AutoCloseable {
    private var handle: Long = CoreRsNative.nativeSearchCreate()

    fun add(id: Long, text: String) {
        checkOpen()
        require(id >= 0) { "id must be >= 0" }
        require(text.isNotBlank()) { "text must not be blank" }
        CoreRsNative.nativeSearchAdd(handle, id, text)
    }

    fun remove(id: Long): Boolean {
        checkOpen()
        require(id >= 0) { "id must be >= 0" }
        return CoreRsNative.nativeSearchRemove(handle, id)
    }

    fun clear() {
        checkOpen()
        CoreRsNative.nativeSearchClear(handle)
    }

    fun size(): Int {
        checkOpen()
        return CoreRsNative.nativeSearchSize(handle)
    }

    fun search(query: String, limit: Int = 20): List<SearchResult> {
        checkOpen()
        require(limit >= 0) { "limit must be >= 0" }
        val root = JSONArray(CoreRsNative.nativeSearch(handle, query, limit))
        return buildList(root.length()) {
            for (index in 0 until root.length()) {
                val item = root.optJSONObject(index) ?: continue
                add(
                    SearchResult(
                        id = item.optLong("id"),
                        score = item.optDouble("score").toFloat(),
                        text = item.optString("text"),
                    )
                )
            }
        }
    }

    override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeSearchDestroy(handle)
            handle = 0L
        }
    }

    private fun checkOpen() = check(handle != 0L) { "SearchEngine has been closed" }
}
