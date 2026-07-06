#include <jni.h>
#include <stdlib.h>
#include <string.h>

extern char *conduit_version(void);
extern void conduit_free_string(char *ptr);
extern int conduit_init(const char *name);
extern int conduit_join(void);
extern int conduit_leave(void);
extern char *conduit_tick(void);
extern int conduit_set_ptt(unsigned char active);
extern int conduit_set_voice_mode(const char *mode);
extern int conduit_send_voice(const short *samples, size_t len);
extern char *conduit_get_diagnostics(void);
extern char *conduit_get_packet_log(void);
extern char *conduit_get_event_log(void);
extern char *conduit_export_logs(void);

static jstring to_jstring(JNIEnv *env, char *cstr) {
  if (cstr == NULL) {
    return (*env)->NewStringUTF(env, "");
  }
  jstring result = (*env)->NewStringUTF(env, cstr);
  conduit_free_string(cstr);
  return result;
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeVersion(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_version());
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeInit(JNIEnv *env, jclass clazz, jstring name) {
  (void)clazz;
  const char *utf = (*env)->GetStringUTFChars(env, name, NULL);
  int result = conduit_init(utf);
  (*env)->ReleaseStringUTFChars(env, name, utf);
  return result;
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeJoin(JNIEnv *env, jclass clazz) {
  (void)env;
  (void)clazz;
  return conduit_join();
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeLeave(JNIEnv *env, jclass clazz) {
  (void)env;
  (void)clazz;
  return conduit_leave();
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeTick(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_tick());
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeSetPtt(JNIEnv *env, jclass clazz, jboolean active) {
  (void)env;
  (void)clazz;
  return conduit_set_ptt(active ? 1 : 0);
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeSetVoiceMode(JNIEnv *env, jclass clazz, jstring mode) {
  (void)clazz;
  const char *utf = (*env)->GetStringUTFChars(env, mode, NULL);
  int result = conduit_set_voice_mode(utf);
  (*env)->ReleaseStringUTFChars(env, mode, utf);
  return result;
}

JNIEXPORT jint JNICALL
Java_com_conduit_plugins_ConduitNative_nativeSendVoice(JNIEnv *env, jclass clazz, jshortArray samples) {
  (void)clazz;
  jsize len = (*env)->GetArrayLength(env, samples);
  jshort *buf = (*env)->GetShortArrayElements(env, samples, NULL);
  int result = conduit_send_voice(buf, (size_t)len);
  (*env)->ReleaseShortArrayElements(env, samples, buf, JNI_ABORT);
  return result;
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeGetDiagnostics(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_get_diagnostics());
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeGetPacketLog(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_get_packet_log());
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeGetEventLog(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_get_event_log());
}

JNIEXPORT jstring JNICALL
Java_com_conduit_plugins_ConduitNative_nativeExportLogs(JNIEnv *env, jclass clazz) {
  (void)clazz;
  return to_jstring(env, conduit_export_logs());
}
