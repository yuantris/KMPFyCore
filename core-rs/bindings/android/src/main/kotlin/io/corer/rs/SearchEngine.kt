package io.corer.rs

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class SearchResult(
    val id: Long,
    val score: Float,
    val text: String,
)

class SearchEngine : AutoCloseable {
    private var handle: Long = CoreRsNative.nativeSearchCreate()

    fun add(id: Long, text: String) {
        checkOpen()
        CoreRsNative.nativeSearchAdd(handle, id, text)
    }

    fun remove(id: Long): Boolean {
        checkOpen()
        return CoreRsNative.nativeSearchRemove(handle, id)
    }

    fun search(query: String, limit: Int = 20): List<SearchResult> {
        checkOpen()
        require(limit >= 0) { "limit must be >= 0" }
        return Json.decodeFromString(CoreRsNative.nativeSearch(handle, query, limit))
    }

    override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeSearchDestroy(handle)
            handle = 0L
        }
    }

    private fun checkOpen() = check(handle != 0L) { "SearchEngine has been closed" }
}
