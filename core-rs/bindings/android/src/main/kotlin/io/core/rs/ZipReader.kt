package io.core.rs

import org.json.JSONArray

data class ZipEntryInfo(
    val name: String,
    val compressedSize: Long,
    val uncompressedSize: Long,
    val isDirectory: Boolean,
)

class ZipReader(path: String) : AutoCloseable {
    private var handle: Long = CoreRsNative.nativeZipCreate(path)

    init {
        check(handle != 0L) { "Unable to open ZIP: $path" }
    }

    fun entries(): List<ZipEntryInfo> {
        checkOpen()
        return parseEntries(CoreRsNative.nativeZipEntries(handle))
    }

    fun contains(name: String): Boolean {
        checkOpen()
        require(name.isNotBlank()) { "name must not be blank" }
        return CoreRsNative.nativeZipContains(handle, name)
    }

    override fun close() {
        if (handle != 0L) {
            CoreRsNative.nativeZipDestroy(handle)
            handle = 0L
        }
    }

    private fun checkOpen() = check(handle != 0L) { "ZipReader has been closed" }
}

class ApkReader(private val path: String) {
    fun isApk(): Boolean = CoreRsNative.nativeApkIsApk(path)

    fun entries(): List<ZipEntryInfo> = parseEntries(CoreRsNative.nativeApkEntries(path))
}

private fun parseEntries(json: String): List<ZipEntryInfo> {
    val root = JSONArray(json)
    return buildList(root.length()) {
        for (index in 0 until root.length()) {
            val item = root.optJSONObject(index) ?: continue
            add(
                ZipEntryInfo(
                    name = item.optString("name"),
                    compressedSize = item.optLong("compressed_size"),
                    uncompressedSize = item.optLong("uncompressed_size"),
                    isDirectory = item.optBoolean("is_dir"),
                )
            )
        }
    }
}
