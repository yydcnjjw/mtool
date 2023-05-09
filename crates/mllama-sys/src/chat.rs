use std::thread;

use serde::Deserialize;

use crate::{token_eos, LLamaContext, LLamaToken, LLamaTokenData, LLamaTokenDataArray};

static default_prompt: &'static str = "Below is an instruction that describes a task. Write a response that appropriately completes the request.";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ChatConfig {
    pub prompt: String,

    pub instruct: bool,
    pub n_keep: usize,
    pub n_batch: usize,
    pub n_threads: usize,
    pub n_predict: isize,

    pub top_k: i32,
    pub top_p: f32,
    pub tfs_z: f32,
    pub typical_p: f32,
    pub temp: f32,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub mirostat: usize,
    pub mirostat_tau: f32,
    pub mirostat_eta: f32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            prompt: default_prompt.to_string(),
            instruct: true,
            n_keep: default_prompt.len(),
            n_batch: 512,
            n_threads: thread::available_parallelism().unwrap().get(),
            n_predict: -1,
            top_k: 40,
            top_p: 0.95,
            tfs_z: 1.,
            typical_p: 1.,
            temp: 0.8,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            frequency_penalty: 0.,
            presence_penalty: 0.,
            mirostat: 0,
            mirostat_tau: 5.,
            mirostat_eta: 0.1,
        }
    }
}

#[derive(Debug)]
pub struct Chat {
    ctx: LLamaContext,

    cfg: ChatConfig,

    n_past: usize,
    n_ctx: usize,
    last_tokens: Vec<LLamaToken>,
    inp_pfx: Vec<LLamaToken>,
    inp_sfx: Vec<LLamaToken>,
}

impl Chat {
    pub fn new(mut ctx: LLamaContext, cfg: ChatConfig) -> Result<Self, anyhow::Error> {
        let n_ctx = ctx.n_ctx() as usize;

        let inp_pfx = ctx.tokenize("\n\n### Instruction:\n\n", true)?;
        let inp_sfx = ctx.tokenize("\n\n### Response:\n\n", false)?;

        let mut this = Self {
            ctx,
            cfg,
            n_past: 0,
            n_ctx,
            last_tokens: vec![0; n_ctx],
            inp_pfx,
            inp_sfx,
        };

        this.eval_prompt(default_prompt)?;

        Ok(this)
    }

    fn eval_prompt(&mut self, prompt: &str) -> Result<(), anyhow::Error> {
        let tokens = self.ctx.tokenize(prompt, true)?;
        self.eval(tokens)
    }

    fn eval(&mut self, mut embd: Vec<LLamaToken>) -> Result<(), anyhow::Error> {
        if self.n_past + embd.len() > self.n_ctx {
            let n_left = self.n_past - self.cfg.n_keep;
            self.n_past = 1.max(self.cfg.n_keep);
            for token in &self.last_tokens
                [(self.n_ctx - n_left / 2 - embd.len())..self.last_tokens.len() - embd.len()]
            {
                embd.insert(0, *token);
            }
        }

        for chunk in embd.chunks(self.cfg.n_batch) {
            self.ctx
                .eval(chunk, self.n_past as i32, self.cfg.n_threads as i32)?;
            self.n_past += chunk.len();

            self.last_tokens.drain(0..chunk.len());
            self.last_tokens.extend(chunk);
        }
        Ok(())
    }

    fn infer(&mut self) -> LLamaToken {
        let logits = self.ctx.get_logits();

        let mut candidates = logits
            .iter()
            .enumerate()
            .map(|(i, logit)| LLamaTokenData::new(i as i32, *logit, 0.0))
            .collect::<Vec<_>>();

        let mut candidates_ref = LLamaTokenDataArray::new(&mut candidates, false);

        let last_n_tokens = self.last_tokens.len();
        let last_n_repeat = last_n_tokens.min(self.cfg.repeat_last_n).min(self.n_ctx);

        self.ctx.sample_repetition_penalty(
            &mut candidates_ref,
            &self.last_tokens[(last_n_tokens - last_n_repeat)..],
            self.cfg.repeat_penalty,
        );
        self.ctx.sample_frequency_and_presence_penalties(
            &mut candidates_ref,
            &self.last_tokens[(last_n_tokens - last_n_repeat)..],
            self.cfg.frequency_penalty,
            self.cfg.presence_penalty,
        );

        if self.cfg.temp <= 0. {
            self.ctx.sample_token_greedy(&mut candidates_ref)
        } else {
            match self.cfg.mirostat {
                1 => {
                    self.ctx
                        .sample_temperature(&mut candidates_ref, self.cfg.temp);
                    let (id, _) = self.ctx.sample_token_mirostat(
                        &mut candidates_ref,
                        self.cfg.mirostat_tau,
                        self.cfg.mirostat_eta,
                        100,
                        2. * self.cfg.mirostat_tau,
                    );
                    id
                }
                2 => {
                    self.ctx
                        .sample_temperature(&mut candidates_ref, self.cfg.temp);
                    let (id, _) = self.ctx.sample_token_mirostat_v2(
                        &mut candidates_ref,
                        self.cfg.mirostat_tau,
                        self.cfg.mirostat_eta,
                        2. * self.cfg.mirostat_tau,
                    );
                    id
                }
                _ => {
                    self.ctx
                        .sample_top_k(&mut candidates_ref, self.cfg.top_k, 1);
                    self.ctx
                        .sample_tail_free(&mut candidates_ref, self.cfg.tfs_z, 1);
                    self.ctx
                        .sample_typical(&mut candidates_ref, self.cfg.typical_p, 1);
                    self.ctx
                        .sample_top_p(&mut candidates_ref, self.cfg.top_p, 1);
                    self.ctx
                        .sample_temperature(&mut candidates_ref, self.cfg.temp);
                    self.ctx.sample_token(&mut candidates_ref)
                }
            }
        }
    }

    fn generate(&mut self) -> Result<String, anyhow::Error> {
        let mut n_remain = self.cfg.n_predict;

        let mut output = String::new();

        while n_remain != 0 {
            let id = self.infer();
            n_remain -= 1;
            self.eval(vec![id])?;

            if id == token_eos() {
                break;
            }

            output.push_str(self.ctx.token_to_str(id)?);
        }

        Ok(output)
    }

    pub fn chat(&mut self, input: &str) -> Result<String, anyhow::Error> {
        let input = self.ctx.tokenize(input, false)?;
        let input = if self.cfg.instruct {
            self.inp_pfx
                .iter()
                .copied()
                .chain(input.iter().copied())
                .chain(self.inp_sfx.iter().copied())
                .collect::<Vec<_>>()
        } else {
            input
        };

        self.eval(input)?;

        self.generate()
    }
}
