fn main() {
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
    cc::Build::new()
      .file("jni/conduit_jni.c")
      .compile("conduit_jni");
    println!("cargo:rerun-if-changed=jni/conduit_jni.c");
  }
}
