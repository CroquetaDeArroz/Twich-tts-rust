mod config;
use config::cargar_config;

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

struct ColaTTS {
    cola: VecDeque<String>,
}

impl ColaTTS {
    fn nueva() -> Self { Self { cola: VecDeque::new() } }
    fn meter(&mut self, t: String) { self.cola.push_back(t); }
    fn sacar(&mut self) -> Option<String> { self.cola.pop_front() }
}

const AUDIO_TMP: &str = "/tmp/tts_output.wav";

fn piper_dir() -> String {
    // En Docker: /app/files/piper/piper
    // En local:  ./files/piper/piper  (relativo al ejecutable)
    std::env::var("PIPER_DIR").unwrap_or_else(|_| "./files/piper/piper".to_string())
}

fn generar_piper(texto: &str, modelo: &str) -> bool {
    let dir = piper_dir();
    let cmd = format!(
        "LD_LIBRARY_PATH={dir} echo '{}' | {dir}/piper --model {} --output_file {}",
        texto.replace('\'', " "),
        modelo,
        AUDIO_TMP
    );

    match Command::new("bash")
        .arg("-c")
        .arg(&cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut hijo) => hijo.wait().map(|s| s.success()).unwrap_or(false),
        Err(e) => { eprintln!("❌ Piper: {e}"); false }
    }
}

fn reproducir_wav(ruta: &str, _card: u8, volumen: f32) {
    let pulse_vol = ((volumen * 65536.0) as u32).clamp(0, 131072);

    let resultado = Command::new("paplay")
        .arg(ruta)
        .arg(format!("--volume={}", pulse_vol))
        .status();

    if resultado.is_err() {
        let _ = Command::new("aplay").arg(ruta).status();
    }
}

fn reproducir_tts(texto: &str, volumen: f32, motor: &str, modelo: &str, card: u8) {
    println!("🔊 [{motor}] ({:.0}%) {texto}", volumen * 100.0);

    if motor == "piper" && generar_piper(texto, modelo) {
        reproducir_wav(AUDIO_TMP, card, volumen);
        return;
    }

    eprintln!("⚠  Usando gTTS como fallback...");
    let _ = Command::new("python3")
        .arg("-c")
        .arg(format!(
            "from gtts import gTTS; gTTS('{}', lang='es').save('/tmp/tts_fb.mp3')",
            texto.replace('\'', " ")
        ))
        .output();
    reproducir_wav("/tmp/tts_fb.mp3", card, volumen);
}

fn parsear_mensaje(linea: &str) -> Option<(String, String)> {
    if linea.split(' ').count() < 4 { return None; }
    let usuario = linea.split('!').next()?.trim_start_matches(':');
    let mensaje  = linea.split(" :").nth(1)?.trim();
    Some((usuario.to_string(), mensaje.to_string()))
}

fn parsear_volumen(texto: &str, actual: f32) -> Option<f32> {
    let p: Vec<&str> = texto.trim().splitn(2, ' ').collect();
    if p.len() != 2 || p[0].to_lowercase() != "!volumen" { return None; }
    let arg = p[1];
    let nuevo = if let Some(n) = arg.strip_prefix('+') {
        actual + n.parse::<f32>().ok()? / 100.0
    } else if let Some(n) = arg.strip_prefix('-') {
        actual - n.parse::<f32>().ok()? / 100.0
    } else {
        arg.parse::<f32>().ok()? / 100.0
    };
    Some(nuevo.clamp(0.0, 2.0))
}

fn main() {
    let cfg = cargar_config();

    println!("✅ Config cargada:");
    println!("   Usuario : {}", cfg.usuario);
    println!("   Canal   : {}", cfg.canal);
    println!("   Motor   : {}", cfg.motor_tts);
    println!("   Volumen : {:.0}%", cfg.volumen * 100.0);

    let cola    = Arc::new(Mutex::new(ColaTTS::nueva()));
    let volumen = Arc::new(Mutex::new(cfg.volumen));
    let motor   = Arc::new(cfg.motor_tts.clone());
    let modelo  = Arc::new(cfg.piper_modelo.clone());

    let cola_a   = cola.clone();
    let vol_a    = volumen.clone();
    let motor_a  = motor.clone();
    let modelo_a = modelo.clone();
    let cfg_card = cfg.alsa_card;

    thread::spawn(move || loop {
        let texto = { cola_a.lock().unwrap().sacar() };
        if let Some(t) = texto {
            let vol = *vol_a.lock().unwrap();
            reproducir_tts(&t, vol, &motor_a, &modelo_a, cfg_card);
        } else {
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let servidor = "irc.chat.twitch.tv:6667";
    let mut strim = TcpStream::connect(servidor).expect("❌ No conecta");

    write!(strim, "PASS {}\r\n", cfg.token).unwrap();
    write!(strim, "NICK {}\r\n", cfg.usuario).unwrap();
    write!(strim, "JOIN {}\r\n", cfg.canal).unwrap();

    let lector = BufReader::new(strim.try_clone().unwrap());
    println!("🎙  Bot TTS activo en {}  —  !volumen [0-200 / +n / -n]", cfg.canal);

    for linea in lector.lines() {
        let linea = linea.unwrap();
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
                    println!("🎚  Volumen → {:.0}%", nv * 100.0);
                }
                continue;
            }
            
            if texto.len() > cfg.longitud_max { continue; }
            if texto.contains("://")          { continue; }
            if texto.starts_with('!')         { continue; }
            
            let mensaje = if cfg.anunciar_usuario {
                format!("{usuario} dice {texto}")
            } else {
                texto.to_string()
            };

            cola.lock().unwrap().meter(mensaje);
        }
    }
}
