mod config;
use config::cargar_config;

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

struct ColaTTS { cola: VecDeque<String> }
impl ColaTTS {
    fn nueva() -> Self { Self { cola: VecDeque::new() } }
    fn meter(&mut self, t: String) { self.cola.push_back(t); }
    fn sacar(&mut self) -> Option<String> { self.cola.pop_front() }
}

const AUDIO_TMP: &str = "/tmp/tts_output.wav";

// ── Piper ─────────────────────────────────────────────────────────────────────
fn piper_dir() -> String {
    std::env::var("PIPER_DIR").unwrap_or_else(|_| "./files/piper/piper".to_string())
}

fn generar_piper(texto: &str, modelo: &str) -> bool {
    let dir = piper_dir();
    let cmd = format!(
        "LD_LIBRARY_PATH=\"{dir}\" echo '{texto}' | \"{dir}/piper\" --model \"{modelo}\" --output_file \"{AUDIO_TMP}\" 2>/tmp/piper_err.log",
        dir = dir, texto = texto.replace('\'', " "), modelo = modelo,
    );
    match Command::new("bash").arg("-c").arg(&cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut h) => {
            let ok = h.wait().map(|s| s.success()).unwrap_or(false);
            if !ok {
                if let Ok(log) = std::fs::read_to_string("/tmp/piper_err.log") {
                    eprintln!("❌ Piper stderr: {}", log.trim());
                }
            }
            ok
        }
        Err(e) => { eprintln!("❌ Piper not found: {e}"); false }
    }
}

// ── Coqui ─────────────────────────────────────────────────────────────────────
fn generar_coqui(texto: &str, modelo: &str, speaker_wav: &str) -> bool {
    let mut args = vec![
        "--text".to_string(),       texto.to_string(),
        "--model_name".to_string(), modelo.to_string(),
        "--out_path".to_string(),   AUDIO_TMP.to_string(),
    ];
    if !speaker_wav.is_empty() {
        args.push("--speaker_wav".to_string());
        args.push(speaker_wav.to_string());
    }
    match Command::new("tts").args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut h) => h.wait().map(|s| s.success()).unwrap_or(false),
        Err(e) => { eprintln!("❌ Coqui not found: {e}"); false }
    }
}

// ── Audio ─────────────────────────────────────────────────────────────────────
fn reproducir_wav(ruta: &str, _card: u8, volumen: f32) {
    let pulse_vol = ((volumen * 65536.0) as u32).clamp(0, 131072);
    let pulse_ok  = Command::new("paplay").arg(ruta)
        .arg(format!("--volume={}", pulse_vol))
        .status().map(|s| s.success()).unwrap_or(false);
    if !pulse_ok {
        eprintln!("⚠  paplay failed → trying aplay");
        if !Command::new("aplay").arg(ruta).status().map(|s| s.success()).unwrap_or(false) {
            eprintln!("❌ aplay also failed.");
        }
    }
}

fn reproducir_tts(
    texto: &str, volumen: f32, motor: &str,
    piper_modelo: &str, coqui_modelo: &str,
    speaker_wav: &str, lang: &str, card: u8,
) {
    println!("🔊 [{motor}] ({:.0}%) {texto}", volumen * 100.0);
    if motor == "piper" && generar_piper(texto, piper_modelo) {
        reproducir_wav(AUDIO_TMP, card, volumen); return;
    }
    if motor == "coqui" && generar_coqui(texto, coqui_modelo, speaker_wav) {
        reproducir_wav(AUDIO_TMP, card, volumen); return;
    }
    eprintln!("⚠  Falling back to gTTS...");
    let _ = Command::new("python3").arg("-c").arg(format!(
        "from gtts import gTTS; gTTS('{}', lang='{}').save('/tmp/tts_fb.mp3')",
        texto.replace('\'', " "), lang
    )).output();
    reproducir_wav("/tmp/tts_fb.mp3", card, volumen);
}

// ── IRC ───────────────────────────────────────────────────────────────────────
fn parsear_mensaje(linea: &str) -> Option<(String, String)> {
    if linea.split(' ').count() < 4 { return None; }
    let usuario = linea.split('!').next()?.trim_start_matches(':');
    let mensaje  = linea.split(" :").nth(1)?.trim();
    Some((usuario.to_string(), mensaje.to_string()))
}

fn parsear_volumen(texto: &str, actual: f32) -> Option<f32> {
    let p: Vec<&str> = texto.trim().splitn(2, ' ').collect();
    if p.len() != 2 || p[0].to_lowercase() != "!volumen" { return None; }
    let nuevo = if let Some(n) = p[1].strip_prefix('+') {
        actual + n.parse::<f32>().ok()? / 100.0
    } else if let Some(n) = p[1].strip_prefix('-') {
        actual - n.parse::<f32>().ok()? / 100.0
    } else {
        p[1].parse::<f32>().ok()? / 100.0
    };
    Some(nuevo.clamp(0.0, 2.0))
}

// ── Main ──────────────────────────────────────────────────────────────────────
fn main() {
    let cfg = cargar_config();

    // Si no se ha elegido idioma aún, el bot no puede arrancar correctamente.
    // El usuario debe pasar primero por el config-ui.
    if cfg.es_primera_vez() {
        eprintln!("⚠  No language configured. Please run config-ui first.");
        std::process::exit(1);
    }

    let lang = cfg.idioma.clone();

    let (coqui_modelo_str, speaker_wav_str) = cfg.coqui_activo()
        .map(|m| {
            let wav = m.voz_actual().map(|v| v.speaker_wav.clone()).unwrap_or_default();
            (m.modelo.clone(), wav)
        })
        .unwrap_or_else(|| ("tts_models/en/ljspeech/vits".to_string(), String::new()));

    println!("✅ Config loaded:");
    println!("   User    : {}", cfg.usuario);
    println!("   Channel : {}", cfg.canal);
    println!("   Engine  : {}", cfg.motor_tts);
    println!("   Volume  : {:.0}%", cfg.volumen * 100.0);
    println!("   Lang    : {}", lang);
    if cfg.motor_tts == "coqui" {
        println!("   Model   : {}", coqui_modelo_str);
        if !speaker_wav_str.is_empty() {
            println!("   Voice   : {}", speaker_wav_str);
        }
    }

    let cola         = Arc::new(Mutex::new(ColaTTS::nueva()));
    let volumen      = Arc::new(Mutex::new(cfg.volumen));
    let motor        = Arc::new(cfg.motor_tts.clone());
    let piper_modelo = Arc::new(cfg.piper_modelo.clone());
    let coqui_modelo = Arc::new(coqui_modelo_str);
    let speaker_wav  = Arc::new(speaker_wav_str);
    let lang_arc     = Arc::new(lang);

    let (cola_a, vol_a, motor_a, piper_a, coqui_a, wav_a, lang_a) = (
        cola.clone(), volumen.clone(), motor.clone(),
        piper_modelo.clone(), coqui_modelo.clone(),
        speaker_wav.clone(), lang_arc.clone(),
    );
    let card = cfg.alsa_card;

    thread::spawn(move || loop {
        if let Some(t) = { cola_a.lock().unwrap().sacar() } {
            let vol = *vol_a.lock().unwrap();
            reproducir_tts(&t, vol, &motor_a, &piper_a, &coqui_a, &wav_a, &lang_a, card);
        } else {
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let mut strim = TcpStream::connect("irc.chat.twitch.tv:6667").expect("❌ Cannot connect");
    write!(strim, "PASS {}\r\nNICK {}\r\nJOIN {}\r\n",
        cfg.token, cfg.usuario, cfg.canal).unwrap();

    let lector = BufReader::new(strim.try_clone().unwrap());
    println!("🎙  Bot active in {}  —  !volumen [0-200/+n/-n]", cfg.canal);

    for linea in lector.lines().map_while(Result::ok) {
        println!("{linea}");
        if linea.starts_with("PING") {
            write!(strim, "{}\r\n", linea.replace("PING", "PONG")).unwrap();
            continue;
        }
        if !linea.contains("PRIVMSG") { continue; }
        if let Some((usuario, texto)) = parsear_mensaje(&linea) {
            println!("[{usuario}]: {texto}");
            if texto.to_lowercase().starts_with("!volumen")
                && usuario.to_lowercase() == cfg.usuario.to_lowercase()
            {
                let actual = *volumen.lock().unwrap();
                if let Some(nv) = parsear_volumen(&texto, actual) {
                    *volumen.lock().unwrap() = nv;
                    println!("🎚  Volume → {:.0}%", nv * 100.0);
                }
                continue;
            }
            if texto.len() > cfg.longitud_max { continue; }
            if texto.contains("://")          { continue; }
            if texto.starts_with('!')         { continue; }
            let msg = if cfg.anunciar_usuario {
                if *lang_arc == "en" {
                    format!("{usuario} says {texto}")
                } else {
                    format!("{usuario} dice {texto}")
                }
            } else { texto.to_string() };
            cola.lock().unwrap().meter(msg);
        }
    }
}
