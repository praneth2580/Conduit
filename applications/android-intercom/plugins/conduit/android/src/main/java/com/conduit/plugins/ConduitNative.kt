package com.conduit.plugins

object ConduitNative {
  init {
    System.loadLibrary("conduit_ffi")
  }

  @JvmStatic external fun nativeVersion(): String
  @JvmStatic external fun nativeInit(name: String): Int
  @JvmStatic external fun nativeJoin(): Int
  @JvmStatic external fun nativeLeave(): Int
  @JvmStatic external fun nativeTick(): String
  @JvmStatic external fun nativeSetPtt(active: Boolean): Int
  @JvmStatic external fun nativeSetVoiceMode(mode: String): Int
  @JvmStatic external fun nativeSendVoice(samples: ShortArray): Int
  @JvmStatic external fun nativeGetDiagnostics(): String
  @JvmStatic external fun nativeGetPacketLog(): String
  @JvmStatic external fun nativeGetEventLog(): String
  @JvmStatic external fun nativeExportLogs(): String
}
