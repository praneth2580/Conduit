package com.conduit.plugins

import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import com.getcapacitor.JSObject
import com.getcapacitor.Plugin
import com.getcapacitor.PluginCall
import com.getcapacitor.PluginMethod
import com.getcapacitor.annotation.CapacitorPlugin
import org.json.JSONArray
import org.json.JSONObject
import kotlin.concurrent.thread

@CapacitorPlugin(name = "Conduit")
class ConduitPlugin : Plugin() {

  private var audioThread: Thread? = null
  @Volatile private var capturing = false
  @Volatile private var pttActive = false

  @PluginMethod
  fun initialize(call: PluginCall) {
    val name = call.getString("name") ?: "android-node"
    val code = ConduitNative.nativeInit(name)
    if (code == 0) call.resolve() else call.reject("init failed: $code")
  }

  @PluginMethod
  fun joinNetwork(call: PluginCall) {
    val code = ConduitNative.nativeJoin()
    if (code == 0) call.resolve() else call.reject("join failed: $code")
  }

  @PluginMethod
  fun leaveNetwork(call: PluginCall) {
    stopAudioCaptureInternal()
    val code = ConduitNative.nativeLeave()
    if (code == 0) call.resolve() else call.reject("leave failed: $code")
  }

  @PluginMethod
  fun tick(call: PluginCall) {
    val json = ConduitNative.nativeTick()
    if (json.isEmpty()) {
      call.reject("tick failed")
      return
    }
    call.resolve(jsonToJSObject(json))
  }

  @PluginMethod
  fun setPushToTalk(call: PluginCall) {
    val active = call.getBoolean("active") ?: false
    pttActive = active
    val code = ConduitNative.nativeSetPtt(active)
    if (code == 0) call.resolve() else call.reject("setPushToTalk failed: $code")
  }

  @PluginMethod
  fun setVoiceMode(call: PluginCall) {
    val mode = call.getString("mode") ?: "ptt"
    val code = ConduitNative.nativeSetVoiceMode(mode)
    if (code == 0) call.resolve() else call.reject("setVoiceMode failed: $code")
  }

  @PluginMethod
  fun sendVoice(call: PluginCall) {
    val arr = call.getArray("samples") ?: run {
      call.reject("samples required")
      return
    }
    val samples = ShortArray(arr.length()) { i -> arr.getInt(i).toShort() }
    val code = ConduitNative.nativeSendVoice(samples)
    if (code == 0) call.resolve() else call.reject("sendVoice failed: $code")
  }

  @PluginMethod
  fun getDiagnostics(call: PluginCall) {
    call.resolve(jsonToJSObject(ConduitNative.nativeGetDiagnostics()))
  }

  @PluginMethod
  fun getPacketLog(call: PluginCall) {
    val ret = JSObject()
    ret.put("value", JSONArray(ConduitNative.nativeGetPacketLog()))
    call.resolve(ret)
  }

  @PluginMethod
  fun getEventLog(call: PluginCall) {
    val ret = JSObject()
    ret.put("value", JSONArray(ConduitNative.nativeGetEventLog()))
    call.resolve(ret)
  }

  @PluginMethod
  fun exportLogs(call: PluginCall) {
    val ret = JSObject()
    ret.put("value", ConduitNative.nativeExportLogs())
    call.resolve(ret)
  }

  @PluginMethod
  fun getVersion(call: PluginCall) {
    call.resolve(jsonToJSObject(ConduitNative.nativeVersion()))
  }

  @PluginMethod
  fun startAudioCapture(call: PluginCall) {
    if (capturing) {
      call.resolve()
      return
    }
    capturing = true
    audioThread = thread(start = true, name = "conduit-audio") {
      val sampleRate = 48000
      val frame = 960
      val minBuf = AudioRecord.getMinBufferSize(
        sampleRate,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT,
      )
      val recorder = AudioRecord(
        MediaRecorder.AudioSource.VOICE_COMMUNICATION,
        sampleRate,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT,
        minBuf.coerceAtLeast(frame * 2),
      )
      val track = AudioTrack.Builder()
        .setAudioFormat(
          AudioFormat.Builder()
            .setSampleRate(sampleRate)
            .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
            .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
            .build(),
        )
        .setBufferSizeInBytes(minBuf.coerceAtLeast(frame * 2))
        .setTransferMode(AudioTrack.MODE_STREAM)
        .build()
      val buffer = ShortArray(frame)
      recorder.startRecording()
      track.play()
      while (capturing) {
        val read = recorder.read(buffer, 0, frame)
        if (read > 0 && pttActive) {
          ConduitNative.nativeSendVoice(buffer.copyOf(read))
        }
        val tickJson = ConduitNative.nativeTick()
        if (tickJson.isNotEmpty()) {
          try {
            val obj = JSONObject(tickJson)
            val playback = obj.optJSONArray("playback_samples")
            if (playback != null && playback.length() > 0) {
              val out = ShortArray(playback.length()) { i -> playback.getInt(i).toShort() }
              track.write(out, 0, out.size)
            }
          } catch (_: Exception) {
          }
        }
        Thread.sleep(5)
      }
      recorder.stop()
      recorder.release()
      track.stop()
      track.release()
    }
    call.resolve()
  }

  @PluginMethod
  fun stopAudioCapture(call: PluginCall) {
    stopAudioCaptureInternal()
    call.resolve()
  }

  private fun stopAudioCaptureInternal() {
    capturing = false
    audioThread?.join(500)
    audioThread = null
  }

  private fun jsonToJSObject(json: String): JSObject {
    val obj = JSObject()
    if (json.isBlank()) return obj
    val parsed = JSONObject(json)
    parsed.keys().forEach { key -> obj.put(key, parsed.get(key)) }
    return obj
  }
}
