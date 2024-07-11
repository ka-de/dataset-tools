extern crate intel_mkl_src;

use std::borrow::Cow;
use fancy_regex::Regex;
use rand::prelude::*;
use rand::distributions::{ WeightedIndex, Distribution };
use rand::rngs::StdRng;
use rand::SeedableRng;

use anyhow::{ Error as E, Result };
use clap::Parser;

use candle_transformers::models::qwen2::{ Config as ConfigBase, ModelForCausalLM as ModelBase };
use candle_transformers::models::qwen2_moe::{ Config as ConfigMoe, Model as ModelMoe };

use candle_core::{ DType, Device, Tensor };
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use hf_hub::{ api::sync::Api, Repo, RepoType };
use tokenizers::Tokenizer;

enum Model {
    Base(ModelBase),
    Moe(ModelMoe),
}

impl Model {
    fn forward(&mut self, xs: &Tensor, s: usize) -> candle_core::Result<Tensor> {
        match self {
            Self::Moe(ref mut m) => m.forward(xs, s),
            Self::Base(ref mut m) => m.forward(xs, s),
        }
    }
}

struct TextGeneration {
    model: Model,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
    rng: StdRng,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Model,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);
        Self {
            model,
            tokenizer: TokenOutputStream::new(tokenizer),
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    // Filters out sentences that contain "text", "hearts", "names",
    // "says", "speech bubble", "word bubble", "~", "..." or "/" or asterisks
    // or ".." or "!" or "patreon" or "twitter" or "logo" and only white spaces
    // and filters out excessive use of white spaces and makes sure all sentences
    // are separated by a white space, it also filters out the contents inside parentheses
    // and also removes parentheses and also makes sure there is no extra
    // white space between a word and a special character.
    fn filter_sentences(&self, text: &str) -> String {
        let sentence_regex = Regex::new(r"(?m)(?:[^.!?]+[.!?]+\s*)").unwrap();
        let special_chars_regex = Regex::new(r"^[^a-zA-Z0-9]+$").unwrap();
        let space_regex = Regex::new(r" {2,}").unwrap();
        let fix_spacing_regex = Regex::new(r"([.!?])([A-Za-z])").unwrap();
        let parentheses_content_regex = Regex::new(r"\([^)]*\)").unwrap();
        let parentheses_char_regex = Regex::new(r"[()]+").unwrap();

        // Filter sentences
        let filtered_sentences: Vec<Cow<'_, str>> = sentence_regex
            .find_iter(text)
            .filter_map(|m| m.ok().map(|match_| match_.as_str()))
            .filter(|sentence| {
                !sentence.to_lowercase().contains("text") &&
                    !sentence.to_lowercase().contains("hearts") &&
                    !sentence.to_lowercase().contains("name") &&
                    !sentence.to_lowercase().contains("twitter") &&
                    !sentence.to_lowercase().contains("patreon") &&
                    !sentence.to_lowercase().contains("logo") &&
                    !sentence.to_lowercase().contains("says") &&
                    !sentence.to_lowercase().contains("word bubble") &&
                    !sentence.to_lowercase().contains("speech bubble") &&
                    !sentence.to_lowercase().contains("signature") &&
                    !sentence.to_lowercase().contains("reads") &&
                    !sentence.to_lowercase().contains("i'll") &&
                    !sentence.contains("...") &&
                    !sentence.contains("!") &&
                    !sentence.contains("..") &&
                    !sentence.contains("~") &&
                    !sentence.contains("*") &&
                    !sentence.contains("/") &&
                    !special_chars_regex.is_match(sentence.trim()).unwrap_or(false) &&
                    !sentence.trim().is_empty()
            })
            .map(|sentence| space_regex.replace_all(sentence, " "))
            .collect::<Vec<_>>();

        // Only remove newlines.
        //filtered_sentences.join("").replace('\n', "")

        // Or you can use this to preserve some structure in the text while still
        // removing excessive newlines, this code will join sentences and replace
        // multiple newlines with a single newline, instead of completely removing
        // every single one like the code on top.
        //let joined_text = filtered_sentences.join("");
        //Regex::new(r"\n+").unwrap().replace_all(&joined_text, "\n").into_owned()

        // Remove newlines and fix spacing after punctuation and filter out parentheses and ".
        let mut result = filtered_sentences.join("").replace('\n', "").replace('"', "");
        result = fix_spacing_regex.replace_all(&result, "$1 $2").to_string();
        result = parentheses_content_regex.replace_all(&result, "").to_string();
        result = parentheses_char_regex.replace_all(&result, "").to_string();

        // Ensure space after special characters
        let special_chars = [',', '.', '!', '?', ';', ':'];
        for ch in special_chars.iter() {
            result = result.replace(&format!("{}", ch), &format!("{} ", ch));
        }

        // Trim spaces before special characters
        for ch in special_chars.iter() {
            result = result.replace(&format!(" {}", ch), &ch.to_string());
        }

        // Word replacement.
        // Replace "kind-of-fleisch-like" with "fleshlight".
        result = result.replace("kind-of-fleisch-like", "fleshlight");
        // Replace "some" with "a".
        result = result.replace("some", "a");
        // Replace "etcetera" with ""
        result = result.replace("etcetera", "");
        // Replace "athing" with "something"
        result = result.replace("athing", "something");
        // Replace "engrossedly engaged" with "engaged"
        result = result.replace("engrossedly engaged", "engaged");

        // Trim extra spaces
        result = space_regex.replace_all(&result, " ").to_string();

        result
    }

    fn run(&mut self, prompt: &str, sample_len: usize, model: WhichModel) -> Result<()> {
        use std::io::Write;
        self.tokenizer.clear();

        // Modify the prompt based on the model
        let modified_prompt = if model == WhichModel::Tag2promptQwen2_0_5bV0_1 {
            // Replace spaces with underscores within tags separated by commas
            let tags_with_underscores = prompt
                .split(',')
                .map(|tag| tag.trim().replace(' ', "_"))
                .collect::<Vec<String>>()
                .join(" ");
            format!("{}\n300\n\n", tags_with_underscores)
        } else {
            prompt.to_string()
        };

        // Print the tokens.
        // This is only for debugging purposes.
        let mut tokens = self.tokenizer
            .tokenizer()
            .encode(modified_prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                // ⚠️ TODO: Replace this with env_logger.
                print!("{t}");
            }
        }
        std::io::stdout().flush()?;

        // Common Words
        let common_words = vec![
            "and",
            "is",
            "are",
            "the",
            "for",
            "a",
            "an",
            "in",
            "on",
            "to",
            "of",
            "it",
            "by",
            "with",
            "as",
            "that",
            "this",
            "but",
            "or",
            "at",
            "from"
        ];

        // Tokenize the common words
        let common_word_ids: Vec<u32> = common_words
            .iter()
            .filter_map(|&word| self.tokenizer.tokenizer().token_to_id(word))
            .collect();

        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|endoftext|> token"),
        };
        let start_gen = std::time::Instant::now();
        let mut generated_text = String::new();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;

            // Repeat Penalty
            let logits = if self.repeat_penalty == 1.0 {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                let penalize_tokens: Vec<u32> = tokens[start_at..]
                    .iter()
                    .filter(|&&token| !common_word_ids.contains(&token))
                    .cloned()
                    .collect();
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &penalize_tokens
                )?
            };

            // Sample
            let next_token = self.logits_processor.sample(&logits)?;

            /*
            // Introduce randomness here
            if self.rng.gen::<f32>() < 0.1 {
                // 10% chance to choose a different token
                let logits_vec: Vec<f32> = logits.to_vec1()?;

                // Filter and normalize weights
                let valid_weights: Vec<f32> = logits_vec
                    .iter()
                    .map(|&x| x.exp()) // Convert logits to probabilities
                    .filter(|&x| x.is_finite() && x > 0.0)
                    .collect();

                if !valid_weights.is_empty() {
                    let distribution = WeightedIndex::new(&valid_weights).map_err(E::msg)?;
                    let alternative_index = distribution.sample(&mut self.rng);
                    let alternative_token = valid_weights
                        .iter()
                        .enumerate()
                        .filter(|(_, &w)| w.is_finite() && w > 0.0)
                        .nth(alternative_index)
                        .map(|(i, _)| i as u32)
                        .unwrap_or(next_token);
                    tokens.push(alternative_token);
                } else {
                    // If no valid weights, fall back to the original next_token
                    tokens.push(next_token);
                }
            } else {
                tokens.push(next_token);
            }

			*/
            tokens.push(next_token);

            generated_tokens += 1;
            if next_token == eos_token {
                break;
            }
            if let Some(t) = self.tokenizer.next_token(next_token)? {
                generated_text.push_str(&t);
                // ⚠️ TODO: Replace this with env_logger.
                print!("{t}");
                std::io::stdout().flush()?;
            }
        }
        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            // ⚠️ TODO: Replace this with env_logger.
            print!("{rest}");
        }
        std::io::stdout().flush()?;

        /* ⚠️ TODO: Replace this with env_logger.
		println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            (generated_tokens as f64) / dt.as_secs_f64()
        );
		*/

        // Apply the filter to the generated text
        let filtered_text = self.filter_sentences(&generated_text);
        // ⚠️ TODO: Replace this with env_logger.
        println!("\nFiltered text:");
        println!("{}", filtered_text);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, clap::ValueEnum, PartialEq, Eq)]
enum WhichModel {
    #[value(name = "0.5b")]
    W0_5b,
    #[value(name = "1.8b")]
    W1_8b,
    #[value(name = "4b")]
    W4b,
    #[value(name = "7b")]
    W7b,
    #[value(name = "14b")]
    W14b,
    #[value(name = "72b")]
    W72b,
    #[value(name = "moe-a2.7b")]
    MoeA27b,
    #[value(name = "2-0.5b")]
    W2_0_5b,
    #[value(name = "2-1.5b")]
    W2_1_5b,
    #[value(name = "2-7b")]
    W2_7b,
    #[value(name = "2-72b")]
    W2_72b,
    //#[value(name = "prompt2tag-qwen2-0.5b-v0.1")]
    //Prompt2tagQwen2_0_5bV0_1,
    #[value(name = "tag2prompt-qwen2-0.5b-v0.1")]
    Tag2promptQwen2_0_5bV0_1,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run on CPU rather than on GPU.
    #[arg(long)]
    cpu: bool,

    /// Enable tracing (generates a trace-timestamp.json file).
    #[arg(long)]
    tracing: bool,

    #[arg(long)]
    use_flash_attn: bool,

    #[arg(long)]
    prompt: String,

    /// The temperature used to generate samples.
    #[arg(long)]
    temperature: Option<f64>,

    /// Nucleus sampling probability cutoff.
    #[arg(long)]
    top_p: Option<f64>,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    seed: u64,

    /// The length of the sample to generate (in tokens).
    #[arg(long, short = 'n', default_value_t = 10000)]
    sample_len: usize,

    #[arg(long)]
    model_id: Option<String>,

    #[arg(long, default_value = "main")]
    revision: String,

    #[arg(long)]
    tokenizer_file: Option<String>,

    #[arg(long)]
    weight_files: Option<String>,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    #[arg(long, default_value_t = 1.1)]
    repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    #[arg(long, default_value_t = 64)]
    repeat_last_n: usize,

    #[arg(long, default_value = "0.5b")]
    model: WhichModel,
}

fn main() -> Result<()> {
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::prelude::*;

    let args = Args::parse();
    let _guard = if args.tracing {
        let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
        tracing_subscriber::registry().with(chrome_layer).init();
        Some(guard)
    } else {
        None
    };
    /* ⚠️ TODO: Replace this with env_logger
    println!(
        "avx: {}, neon: {}, simd128: {}, f16c: {}",
        candle::utils::with_avx(),
        candle::utils::with_neon(),
        candle::utils::with_simd128(),
        candle::utils::with_f16c()
    );
    println!(
        "temp: {:.2} repeat-penalty: {:.2} repeat-last-n: {}",
        args.temperature.unwrap_or(0.0),
        args.repeat_penalty,
        args.repeat_last_n
    );
    */
    let start = std::time::Instant::now();
    let api = Api::new()?;
    let model_id = match args.model_id {
        Some(model_id) => model_id,
        None => {
            let (version, size) = match args.model {
                WhichModel::W2_0_5b => ("Qwen/Qwen2", "-0.5B"),
                WhichModel::W2_1_5b => ("Qwen/Qwen2", "-1.5B"),
                WhichModel::W2_7b => ("Qwen/Qwen2", "-7B"),
                WhichModel::W2_72b => ("Qwen/Qwen2", "-72B"),
                WhichModel::W0_5b => ("Qwen/Qwen1.5", "-0.5B"),
                WhichModel::W1_8b => ("Qwen/Qwen1.5", "-1.8B"),
                WhichModel::W4b => ("Qwen/Qwen1.5", "-4B"),
                WhichModel::W7b => ("Qwen/Qwen1.5", "-7B"),
                WhichModel::W14b => ("Qwen/Qwen1.5", "-14B"),
                WhichModel::W72b => ("Qwen/Qwen1.5", "-72B"),
                WhichModel::MoeA27b => ("Qwen/Qwen1.5", "-MoE-A2.7B"),
                //WhichModel::Prompt2tagQwen2_0_5bV0_1 => ("Thouph/", "prompt2tag-qwen2-0.5b-v0.1"),
                WhichModel::Tag2promptQwen2_0_5bV0_1 => ("Thouph/", "tag2prompt-qwen2-0.5b-v0.1"),
            };
            format!("{version}{size}")
        }
    };
    let repo = api.repo(Repo::with_revision(model_id, RepoType::Model, args.revision));
    let tokenizer_filename = match args.tokenizer_file {
        Some(file) => std::path::PathBuf::from(file),
        None => repo.get("tokenizer.json")?,
    };
    let filenames = match args.weight_files {
        Some(files) => files.split(',').map(std::path::PathBuf::from).collect::<Vec<_>>(),
        None =>
            match args.model {
                //| WhichModel::Prompt2tagQwen2_0_5bV0_1
                | WhichModel::Tag2promptQwen2_0_5bV0_1
                | WhichModel::W0_5b
                | WhichModel::W2_0_5b
                | WhichModel::W2_1_5b
                | WhichModel::W1_8b => {
                    vec![repo.get("model.safetensors")?]
                }
                | WhichModel::W4b
                | WhichModel::W7b
                | WhichModel::W2_7b
                | WhichModel::W14b
                | WhichModel::W72b
                | WhichModel::W2_72b
                | WhichModel::MoeA27b => {
                    candle_examples::hub_load_safetensors(&repo, "model.safetensors.index.json")?
                }
            }
    };
    // ⚠️ TODO: Replace this with env_logger.
    //println!("retrieved the files in {:?}", start.elapsed());
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    let start = std::time::Instant::now();
    let config_file = repo.get("config.json")?;
    let device = candle_examples::device(args.cpu)?;
    let dtype = if device.is_cuda() { DType::BF16 } else { DType::F32 };
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
    let model = match args.model {
        WhichModel::MoeA27b => {
            let config: ConfigMoe = serde_json::from_slice(&std::fs::read(config_file)?)?;
            Model::Moe(ModelMoe::new(&config, vb)?)
        }
        _ => {
            let config: ConfigBase = serde_json::from_slice(&std::fs::read(config_file)?)?;
            Model::Base(ModelBase::new(&config, vb)?)
        }
    };

    // ⚠️ TODO: Replace this with env_logger.
    //println!("loaded the model in {:?}", start.elapsed());

    let mut pipeline = TextGeneration::new(
        model,
        tokenizer,
        args.seed,
        args.temperature,
        args.top_p,
        args.repeat_penalty,
        args.repeat_last_n,
        &device
    );

    pipeline.run(&args.prompt, args.sample_len, args.model)?;

    Ok(())
}
