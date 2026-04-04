use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Voz de referencia para XTTS-v2 ───────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoquiVoz {
    pub nombre:      String,
    pub speaker_wav: String,
}

// ── Modelo Coqui ──────────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoquiModelo {
    pub nombre:     String,
    pub modelo:     String,
    pub voces:      Vec<CoquiVoz>,
    pub voz_activa: usize,
}

impl CoquiModelo {
    pub fn voz_actual(&self) -> Option<&CoquiVoz> {
        self.voces.get(self.voz_activa)
    }
}

// ── Modelos por idioma ────────────────────────────────────────────────────────

pub fn modelos_coqui_es() -> Vec<CoquiModelo> {
    vec![
        CoquiModelo {
            nombre:     "ES · css10 VITS".to_string(),
            modelo:     "tts_models/es/css10/vits".to_string(),
            voces:      vec![],
            voz_activa: 0,
        },
        CoquiModelo {
            nombre:     "XTTS-v2".to_string(),
            modelo:     "tts_models/multilingual/multi-dataset/xtts_v2".to_string(),
            voces: vec![
                CoquiVoz { nombre: "Hannibal".to_string(), speaker_wav: "./voices/hannibal.wav".to_string() },
                CoquiVoz { nombre: "Enrique".to_string(),  speaker_wav: "./voices/enrique.wav".to_string()  },
                CoquiVoz { nombre: "Conchita".to_string(), speaker_wav: "./voices/conchita.wav".to_string() },
            ],
            voz_activa: 0,
        },
    ]
}

pub fn modelos_coqui_en() -> Vec<CoquiModelo> {
    vec![
        CoquiModelo {
            nombre:     "EN · ljspeech VITS".to_string(),
            modelo:     "tts_models/en/ljspeech/vits".to_string(),
            voces:      vec![],
            voz_activa: 0,
        },
        CoquiModelo {
            nombre:     "XTTS-v2".to_string(),
            modelo:     "tts_models/multilingual/multi-dataset/xtts_v2".to_string(),
            voces: vec![
                CoquiVoz { nombre: "Hannibal".to_string(), speaker_wav: "./voices/hannibal.wav".to_string() },
                CoquiVoz { nombre: "Enrique".to_string(),  speaker_wav: "./voices/enrique.wav".to_string()  },
                CoquiVoz { nombre: "Conchita".to_string(), speaker_wav: "./voices/conchita.wav".to_string() },
            ],
            voz_activa: 0,
        },
    ]
}

// ── Config principal ──────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub idioma:           String,   // "es" | "en"
    pub usuario:          String,
    pub token:            String,
    pub canal:            String,
    pub volumen:          f32,
    pub motor_tts:        String,
    pub piper_modelo:     String,
    pub coqui_modelos:    Vec<CoquiModelo>,
    pub coqui_modelo_idx: usize,
    pub longitud_max:     usize,
    pub alsa_card:        u8,
    pub anunciar_usuario: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idioma:           String::new(),   // vacío = primera vez, mostrar bienvenida
            usuario:          String::new(),
            token:            String::new(),
            canal:            String::new(),
            volumen:          1.0,
            motor_tts:        "piper".to_string(),
            piper_modelo:     String::new(),   // se rellena al elegir idioma
            coqui_modelos:    vec![],
            coqui_modelo_idx: 0,
            longitud_max:     120,
            alsa_card:        0,
            anunciar_usuario: true,
        }
    }
}

impl Config {
    /// Devuelve true si el usuario aún no ha elegido idioma
    pub fn es_primera_vez(&self) -> bool {
        self.idioma.is_empty()
    }

    /// Aplica los valores por defecto del idioma elegido
    pub fn aplicar_idioma(&mut self, idioma: &str) {
        self.idioma = idioma.to_string();
        match idioma {
            "en" => {
                self.piper_modelo  = "./piper/en_US-lessac-medium.onnx".to_string();
                self.coqui_modelos = modelos_coqui_en();
            }
            _ => {
                self.piper_modelo  = "./piper/es_ES-sharvard-medium.onnx".to_string();
                self.coqui_modelos = modelos_coqui_es();
            }
        }
    }

    pub fn coqui_activo(&self) -> Option<&CoquiModelo> {
        self.coqui_modelos.get(self.coqui_modelo_idx)
    }
    pub fn coqui_activo_mut(&mut self) -> Option<&mut CoquiModelo> {
        self.coqui_modelos.get_mut(self.coqui_modelo_idx)
    }
}

pub fn ruta_config() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("twitch-tts");
    fs::create_dir_all(&p).ok();
    p.push("config.json");
    p
}

pub fn cargar_config() -> Config {
    let ruta = ruta_config();
    fs::read_to_string(&ruta)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn guardar_config(cfg: &Config) -> std::io::Result<()> {
    let ruta = ruta_config();
    let json = serde_json::to_string_pretty(cfg).unwrap();
    fs::write(ruta, json)
}

pub fn listar_tarjetas_alsa() -> Vec<(u8, String)> {
    use std::collections::BTreeMap;
    use std::process::Command;
    let mut mapa: BTreeMap<u8, String> = BTreeMap::new();
    if let Ok(out) = Command::new("aplay").arg("-l").output() {
        let texto = String::from_utf8_lossy(&out.stdout);
        for linea in texto.lines() {
            if !linea.starts_with("card ") { continue; }
            let mut partes = linea.splitn(2, ':');
            let num_str = partes.next().unwrap_or("").trim_start_matches("card ").trim();
            let resto   = partes.next().unwrap_or("");
            if let Ok(num) = num_str.parse::<u8>() {
                let nombre = if let (Some(a), Some(b)) = (resto.find('['), resto.find(']')) {
                    resto[a+1..b].to_string()
                } else {
                    resto.split(',').next().unwrap_or("?").trim().to_string()
                };
                mapa.entry(num).or_insert(nombre);
            }
        }
    }
    let tarjetas: Vec<(u8, String)> = mapa.into_iter().collect();
    if tarjetas.is_empty() { vec![(0, "default".to_string())] } else { tarjetas }
}
