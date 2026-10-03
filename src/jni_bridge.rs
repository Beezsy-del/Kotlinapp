use crate::contract_engine::ContractEngine;
use crate::models::Language;
use crate::phi35_engine::Phi35Engine;
use crate::voice_engine::VoiceEngine;
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::jstring;
use jni::JNIEnv;

/// Analyze contract text from Android Java/Kotlin, returning JSON string
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clausehawk_VerdictEdgeBridge_analyzeContract(
    mut env: JNIEnv,
    _class: JClass,
    contract_text: JString,
    dealbreakers_array: JObjectArray,
    lang_code: JString,
) -> jstring {
    let contract_str: String = match env.get_string(&contract_text) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let lang_str: String = match env.get_string(&lang_code) {
        Ok(s) => s.into(),
        Err(_) => "en".to_string(),
    };

    let language = match lang_str.as_str() {
        "hi" => Language::Hindi,
        "kn" => Language::Kannada,
        _ => Language::English,
    };

    // Extract dealbreaker rules from Java array
    let mut dealbreakers = Vec::new();
    if let Ok(len) = env.get_array_length(&dealbreakers_array) {
        for i in 0..len {
            if let Ok(obj) = env.get_object_array_element(&dealbreakers_array, i) {
                let jstr: JString = obj.into();
                if let Ok(rust_str) = env.get_string(&jstr) {
                    dealbreakers.push(rust_str.into());
                }
            }
        }
    }

    let engine = ContractEngine::new();
    let phi35 = Phi35Engine::new();

    let result = engine.analyze_contract_with_llm(
        &contract_str,
        &dealbreakers,
        &phi35,
        language,
    );

    let json_output = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
    env.new_string(json_output)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Normalize text for speech synthesis from Android
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clausehawk_VerdictEdgeBridge_normalizeSpeech(
    mut env: JNIEnv,
    _class: JClass,
    raw_text: JString,
    lang_code: JString,
) -> jstring {
    let text: String = match env.get_string(&raw_text) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let lang_str: String = match env.get_string(&lang_code) {
        Ok(s) => s.into(),
        Err(_) => "en".to_string(),
    };

    let language = match lang_str.as_str() {
        "hi" => Language::Hindi,
        "kn" => Language::Kannada,
        _ => Language::English,
    };

    let normalized = VoiceEngine::normalize_for_speech(&text, language);
    env.new_string(normalized)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Generate a balanced counter-clause draft via Microsoft Phi-3.5 Mini from Android
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clausehawk_VerdictEdgeBridge_generateCounterClause(
    mut env: JNIEnv,
    _class: JClass,
    original_clause: JString,
    legal_issue: JString,
) -> jstring {
    let clause: String = match env.get_string(&original_clause) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let issue: String = match env.get_string(&legal_issue) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let phi35 = Phi35Engine::new();
    let counter_draft = phi35.generate_counter_clause(&clause, &issue);

    env.new_string(counter_draft)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
