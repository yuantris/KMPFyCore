package io.core.rs

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class ZipEntryInfo(
    val name: String,
    val compressed_size: Long,
    val uncompressed_size: Long,
    val is_dir: Boolean,
)

class ZipReader(path: String) : AutoCloseable {
    private var handle: Long = CoreRsNative.nativeZipCreate(path)

    init { check(handle != 0L) { "Unable to open ZIP: $path" } }

    fun entries(): List<ZipEntryInfo> = run {
        checkOpen()
        Json.decodeFromString(CoreRsNative.nativeZipEntries(handle))
    }

    override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeZipDestroy(handle)
            handle = 0L
        }
    }

    private fun checkOpen() = check(handle != 0L) { "ZipReader has been closed" }
}
