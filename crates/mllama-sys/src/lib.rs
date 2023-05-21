#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod chat;
pub mod sys;

pub use chat::*;

use std::{
    ffi::{CStr, CString},
    ptr::slice_from_raw_parts_mut,
};

use anyhow::Context;

pub fn print_system_info() -> Result<&'static str, anyhow::Error> {
    unsafe { CStr::from_ptr(sys::llama_print_system_info().cast_mut()) }
        .to_str()
        .context("print_system_info")
}

pub fn mmap_supported() -> bool {
    unsafe { sys::llama_mmap_supported() }
}

pub fn mlock_supported() -> bool {
    unsafe { sys::llama_mlock_supported() }
}

pub type LLamaContextParam = sys::llama_context_params;

impl Default for LLamaContextParam {
    fn default() -> Self {
        unsafe { sys::llama_context_default_params() }
    }
}

pub type LLamaToken = sys::llama_token;

pub fn token_bos() -> LLamaToken {
    unsafe { sys::llama_token_bos() }
}

pub fn token_eos() -> LLamaToken {
    unsafe { sys::llama_token_eos() }
}

pub fn token_nl() -> LLamaToken {
    unsafe { sys::llama_token_nl() }
}

pub type LLamaTokenData = sys::llama_token_data;

impl LLamaTokenData {
    pub fn new(id: LLamaToken, logit: f32, p: f32) -> Self {
        Self { id, logit, p }
    }
}

pub type LLamaTokenDataArray = sys::llama_token_data_array;

impl LLamaTokenDataArray {
    pub fn new(tokens: &mut [LLamaTokenData], sorted: bool) -> Self {
        Self {
            data: tokens.as_mut_ptr(),
            size: tokens.len(),
            sorted,
        }
    }
}

#[derive(Debug)]
pub struct LLamaContext {
    handle: *const sys::llama_context,
    last_n_tokens: usize,
}

unsafe impl Send for LLamaContext {}

impl LLamaContext {
    pub fn new(model: &str, param: LLamaContextParam) -> Result<Self, anyhow::Error> {
        Ok(Self {
            handle: unsafe {
                let ctx = sys::llama_init_from_file(CString::new(model)?.as_ptr(), param);
                if ctx.is_null() {
                    anyhow::bail!("LLama Context init failed");
                }
                ctx
            },
            last_n_tokens: 0,
        })
    }

    fn get_handle(&self) -> *const sys::llama_context {
        self.handle
    }

    fn get_handle_mut(&self) -> *mut sys::llama_context {
        self.handle.cast_mut()
    }

    pub fn get_kv_cache_token_count(&self) -> i32 {
        unsafe { sys::llama_get_kv_cache_token_count(self.get_handle()) }
    }

    pub fn set_rng_seed(&mut self, seed: i32) {
        unsafe { sys::llama_set_rng_seed(self.get_handle_mut(), seed) }
    }

    fn get_state_size(&self) -> usize {
        unsafe { sys::llama_get_state_size(self.get_handle()) }
    }

    pub fn copy_state_data(&mut self, dest: &mut Vec<u8>) -> usize {
        let len = self.get_state_size();
        dest.resize(len, 0);
        let n = unsafe { sys::llama_copy_state_data(self.get_handle_mut(), dest.as_mut_ptr()) };
        assert!(n == len);
        n
    }

    pub fn set_state_data(&mut self, src: &[u8]) -> usize {
        unsafe { sys::llama_set_state_data(self.get_handle_mut(), src.as_ptr()) }
    }

    pub fn tokenize(
        &mut self,
        text: &str,
        add_bos: bool,
    ) -> Result<Vec<LLamaToken>, anyhow::Error> {
        let mut buf = vec![0i32; text.len() + if add_bos { 1 } else { 0 }];
        let n = unsafe {
            sys::llama_tokenize(
                self.get_handle_mut(),
                CString::new(text)?.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as i32,
                add_bos,
            )
        };

        if n < 0 {
            anyhow::bail!("LLama tokenize failed");
        }

        buf.resize(n as usize, 0i32);
        Ok(buf)
    }

    pub fn n_ctx(&self) -> i32 {
        unsafe { sys::llama_n_ctx(self.get_handle()) }
    }

    pub fn n_embd(&self) -> i32 {
        unsafe { sys::llama_n_embd(self.get_handle()) }
    }

    fn n_vocab(&self) -> i32 {
        unsafe { sys::llama_n_vocab(self.get_handle()) }
    }

    pub fn eval(
        &mut self,
        tokens: &[LLamaToken],
        n_past: i32,
        n_threads: i32,
    ) -> Result<(), anyhow::Error> {
        self.last_n_tokens = tokens.len();
        if unsafe {
            sys::llama_eval(
                self.get_handle_mut(),
                tokens.as_ptr(),
                self.last_n_tokens as i32,
                n_past,
                n_threads,
            )
        } != 0
        {
            anyhow::bail!("LLama eval failed");
        }
        Ok(())
    }

    pub fn sample_repetition_penalty(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        last_tokens: &[LLamaToken],
        penalty: f32,
    ) {
        unsafe {
            sys::llama_sample_repetition_penalty(
                self.get_handle_mut(),
                candidates as *mut _,
                last_tokens.as_ptr(),
                last_tokens.len(),
                penalty,
            )
        }
    }

    pub fn sample_frequency_and_presence_penalties(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        last_tokens: &[LLamaToken],
        alpha_frequency: f32,
        alpha_presence: f32,
    ) {
        unsafe {
            sys::llama_sample_frequency_and_presence_penalties(
                self.get_handle_mut(),
                candidates as *mut _,
                last_tokens.as_ptr(),
                last_tokens.len(),
                alpha_frequency,
                alpha_presence,
            )
        }
    }

    pub fn sample_softmax(&mut self, candidates: &mut LLamaTokenDataArray) {
        unsafe { sys::llama_sample_softmax(self.get_handle_mut(), candidates as *mut _) }
    }

    pub fn sample_top_k(&mut self, candidates: &mut LLamaTokenDataArray, k: i32, min_keep: usize) {
        unsafe { sys::llama_sample_top_k(self.get_handle_mut(), candidates as *mut _, k, min_keep) }
    }

    pub fn sample_top_p(&mut self, candidates: &mut LLamaTokenDataArray, p: f32, min_keep: usize) {
        unsafe { sys::llama_sample_top_p(self.get_handle_mut(), candidates as *mut _, p, min_keep) }
    }

    pub fn sample_tail_free(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        z: f32,
        min_keep: usize,
    ) {
        unsafe {
            sys::llama_sample_tail_free(self.get_handle_mut(), candidates as *mut _, z, min_keep)
        }
    }

    pub fn sample_typical(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        p: f32,
        min_keep: usize,
    ) {
        unsafe {
            sys::llama_sample_typical(self.get_handle_mut(), candidates as *mut _, p, min_keep)
        }
    }

    pub fn sample_temperature(&mut self, candidates: &mut LLamaTokenDataArray, p: f32) {
        unsafe { sys::llama_sample_temperature(self.get_handle_mut(), candidates as *mut _, p) }
    }

    pub fn sample_token_mirostat(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        tau: f32,
        eta: f32,
        m: i32,
        mut mu: f32,
    ) -> (LLamaToken, f32) {
        unsafe {
            (
                sys::llama_sample_token_mirostat(
                    self.get_handle_mut(),
                    candidates as *mut _,
                    tau,
                    eta,
                    m,
                    &mut mu as *mut _,
                ),
                mu,
            )
        }
    }

    pub fn sample_token_mirostat_v2(
        &mut self,
        candidates: &mut LLamaTokenDataArray,
        tau: f32,
        eta: f32,
        mut mu: f32,
    ) -> (LLamaToken, f32) {
        unsafe {
            (
                sys::llama_sample_token_mirostat_v2(
                    self.get_handle_mut(),
                    candidates as *mut _,
                    tau,
                    eta,
                    &mut mu as *mut _,
                ),
                mu,
            )
        }
    }

    pub fn sample_token_greedy(&mut self, candidates: &mut LLamaTokenDataArray) -> LLamaToken {
        unsafe { sys::llama_sample_token_greedy(self.get_handle_mut(), candidates as *mut _) }
    }

    pub fn sample_token(&mut self, candidates: &mut LLamaTokenDataArray) -> LLamaToken {
        unsafe { sys::llama_sample_token(self.get_handle_mut(), candidates as *mut _) }
    }

    // pub fn print_timings() {}

    // pub fn reset_timings() {}

    pub fn get_embeddings(&self) -> *mut [f32] {
        let data = unsafe { sys::llama_get_embeddings(self.get_handle_mut()) };
        let len = self.n_embd() as usize;
        slice_from_raw_parts_mut(data, len)
    }

    pub fn get_logits(&self) -> &mut [f32] {
        let data = unsafe { sys::llama_get_logits(self.get_handle_mut()) };
        let len = self.n_vocab() as usize; // * self.last_n_tokens;
        unsafe { &mut *slice_from_raw_parts_mut(data, len) }
    }

    // pub fn load_session_file(
    //     &self,
    //     path_session: &str,

    //     n_token_capacity: usize,
    // ) -> Result<Vec<LLamaToken>, anyhow::Error> {

    // }

    // pub fn save_session_file(&self) -> bool {}

    pub fn token_to_str(&self, token: LLamaToken) -> Result<&str, anyhow::Error> {
        unsafe { CStr::from_ptr(sys::llama_token_to_str(self.get_handle(), token).cast_mut()) }
            .to_str()
            .context("token to str")
    }
}

impl Drop for LLamaContext {
    fn drop(&mut self) {
        unsafe {
            sys::llama_free(self.get_handle_mut());
        }
    }
}
