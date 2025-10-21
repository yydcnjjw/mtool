

fn launch (){
    
}

use dioxus_desktop::wry::{android_setup, prelude::*};

android_fn![
    org_yydcnjjw_mtool_dioxus,
    wry,
    WryActivity,
    onCreate,
    [JObject]
];

#[allow(non_snake_case)]
pub unsafe fn onCreate(jenv: JNIEnv, _: JClass, activity: JObject) {
    let activity = jenv.new_global_ref(activity).unwrap();

    android_setup(
        "org/yydcnjjw/mtool/dioxus/wry",
        jenv,
        &ndk::looper::ThreadLooper::for_thread().unwrap(),
        activity,
    );
}

