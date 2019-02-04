#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate restson;
extern crate serde;
extern crate serde_json;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use restson::{Error, RestClient, RestPath};

#[derive(Clone, Debug, Deserialize)]
struct VoiceLine {
    name: String,
    text: String,
    female: bool,
    city17: Option<bool>,
    pitch: Option<f32>,
    gain: Option<f32>,
    rate: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct VoiceLines {
    lines: Vec<VoiceLine>,
}

#[derive(Serialize, Deserialize)]
struct InputConfig {
    text: Option<String>,
    ssml: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct VoiceConfig {
    #[serde(rename = "languageCode")]
    language_code: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "ssmlGender")]
    gender: String,
}

#[derive(Serialize, Deserialize)]
struct AudioConfig {
    #[serde(rename = "audioEncoding")]
    audio_encoding: String,

    #[serde(rename = "pitch")]
    pitch: f32,

    #[serde(rename = "speakingRate")]
    speaking_rate: f32,

    #[serde(rename = "volumeGainDb")]
    gain: f32,
}

#[derive(Serialize, Deserialize)]
struct SynthesizeRequest {
    #[serde(rename = "input")]
    input: InputConfig,

    #[serde(rename = "voice")]
    voice: VoiceConfig,

    #[serde(rename = "audioConfig")]
    audio_config: AudioConfig,
}

#[derive(Deserialize)]
struct SynthesizeResponse {
    #[serde(rename = "audioContent")]
    audio_content: String,
}

impl RestPath<String> for SynthesizeRequest {
    fn get_path(param: String) -> Result<String, Error> {
        Ok(format!("v1beta1/{}", param))
    }
}

fn synthesize_voice(
    api_key: &str,
    text: &str,
    pitch: f32,
    gain: f32,
    speaking_rate: f32,
    female: bool,
) -> Vec<u8> {
    // https://cloud.google.com/storage/docs/json_api/v1/how-tos/authorizing
    let params = vec![("key", api_key)];

    let language = "en-US";

    let (voice_name, gender) = if female {
        ("en-US-Wavenet-C", "FEMALE")
    } else {
        ("en-US-Wavenet-D", "MALE")
    };

    let data = SynthesizeRequest {
        input: InputConfig {
            ssml: Some(String::from(text)),
            text: None,
        },
        voice: VoiceConfig {
            language_code: String::from(language), // https://cloud.google.com/speech/docs/languages
            name: String::from(voice_name),        // en-US-Wavenet-C (female)
            gender: String::from(gender),          // MALE, FEMALE, NEUTRAL
        },
        audio_config: AudioConfig {
            audio_encoding: String::from("LINEAR16"), // OGG_OPUS, LINEAR16, MP3
            pitch,
            gain,
            speaking_rate,
        },
    };

    let mut client = RestClient::new("https://texttospeech.googleapis.com").unwrap();

    // https://cloudplatform.googleblog.com/2018/03/introducing-Cloud-Text-to-Speech-powered-by-Deepmind-WaveNet-technology.html
    // https://developers.google.com/web/updates/2014/01/Web-apps-that-talk-Introduction-to-the-Speech-Synthesis-API
    // https://cloud.google.com/speech/reference/rpc/google.cloud.speech.v1beta1
    // https://cloud.google.com/text-to-speech/docs/reference/rest/v1beta1/text/synthesize
    let resp: Result<SynthesizeResponse, Error> =
        client.post_capture_with(String::from("text:synthesize"), &data, &params);
    match resp {
        Err(err) => {
            println!("Failed processing request: {:?}", err);

            // Print out serialized request
            let serialized = serde_json::to_string(&data).unwrap();
            println!("Serialized request is: {}", serialized);
            Vec::new()
        }
        Ok(val) => {
            let bytes_vec = base64::decode(&val.audio_content).unwrap();
            bytes_vec
        }
    }
}

#[cfg(feature = "wavenet")]
fn load_lines() -> Vec<VoiceLine> {
    let mut file = File::open("./resources/voice/lines.toml").expect("failed to open lines.toml");
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str)
        .expect("failed to load lines.toml");
    let mut decoded: VoiceLines = toml::from_str(&toml_str).unwrap();
    let mut to_process: Vec<VoiceLine> = Vec::with_capacity(decoded.lines.len());
    let mut cache_file = File::open("./resources/voice/lines_cache.toml");
    if let Ok(ref mut cache_file) = cache_file {
        let mut toml_cache_str = String::new();
        cache_file
            .read_to_string(&mut toml_cache_str)
            .expect("failed to load lines_cache.toml");
        let cache: VoiceLines = toml::from_str(&toml_cache_str).unwrap();
        for line in &decoded.lines {
            let named_entry = cache
                .lines
                .iter()
                .find(|&cache_line| cache_line.name == line.name);
            if let Some(ref entry) = named_entry {
                let wav_name = format!("./resources/voice/{}.wav", line.name);
                let ogg_name = format!("./resources/voice/{}.ogg", line.name);
                let wav_exists = Path::new(&wav_name).exists();
                let ogg_exists = Path::new(&ogg_name).exists();

                // Check if files exist and all fields match
                if wav_exists
                    && ogg_exists
                    && entry.text == line.text
                    && entry.female == line.female
                    && entry.pitch.unwrap_or_default() == line.pitch.unwrap_or_default()
                    && entry.gain.unwrap_or_default() == line.gain.unwrap_or_default()
                    && entry.rate.unwrap_or_default() == line.rate.unwrap_or_default()
                {
                    // Already built
                    println!("Already built: {}", line.name);
                    continue;
                }
            }

            println!("Building: {}", line.name);
            to_process.push(line.clone());
        }
    } else {
        // No cache yet, build everything
        println!("Building everything");
        to_process.append(&mut decoded.lines);
    }
    std::fs::copy(
        "./resources/voice/lines.toml",
        "./resources/voice/lines_cache.toml",
    )
    .expect("failed to copy cache file");
    to_process
}

#[cfg(not(feature = "wavenet"))]
fn load_lines() -> Vec<VoiceLine> {
    Vec::new()
}

#[cfg(not(feature = "wavenet"))]
fn main() {}

#[cfg(feature = "wavenet")]
fn main() {
    //println!("cargo:rerun-if-changed=./resources/voice");

    let mut key_file = File::open("./api_key.txt").expect("failed to open api_key.txt");
    let mut key_str = String::new();
    key_file
        .read_to_string(&mut key_str)
        .expect("failed to load api_key.txt");

    let lines = load_lines();
    for line in &lines {
        let data = synthesize_voice(
            &key_str,
            &line.text,
            line.pitch.unwrap_or(0f32),
            line.gain.unwrap_or(0f32),
            line.rate.unwrap_or(1f32),
            line.female,
        );
        let data = data.as_slice();

        let wav_name = format!("./resources/voice/{}.wav", line.name);
        let ogg_name = format!("./resources/voice/{}.ogg", line.name);

        {
            let mut wav_file = File::create(&wav_name).expect("Unable to create wav file");
            wav_file.write_all(&data).expect("Unable to write data");
        }

        let mut output = Command::new("ffmpeg");
        output.arg("-i");
        output.arg(wav_name);
        if line.city17.unwrap_or_default() {
            output.arg("-filter_complex");
            output.arg("acrusher=bits=2.5:mode=lin:samples=4:aa=0,asetrate=24000*0.7,aresample=24000,atempo=1.429,treble=g=10");
        }
        output.arg("-c:a");
        output.arg("libvorbis");
        output.arg("-y");
        output.arg("-qscale:a");
        output.arg("4");
        output.arg(ogg_name);
        output.output().expect("failed to run ffmpeg");
    }
}
