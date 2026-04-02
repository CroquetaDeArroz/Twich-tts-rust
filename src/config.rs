use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub usuario: String,
    pub token: String,
    pub canal: String,
    pub volumen: f32,
    pub motor_tts: String,
    pub piper_modelo: String,
    pub longitud_max: usize,
    pub alsa_card: u8,
    pub anunciar_usuario: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            usuario: String::new(),
            token: String::new(),
            canal: String::new(),
            volumen: 1.0,
            motor_tts: "piper".to_string(),
            piper_modelo: "./piper/es_ES-sharvard-medium.onnx".to_string(),
            longitud_max: 120,
            alsa_card: 0,
            anunciar_usuario: true,
        }
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

/// Devuelve las tarjetas ALSA disponibles como Vec<(numero, nombre)>
/// Parsea la salida de `aplay -l` para funcionar tanto en host como en Docker
pub fn listar_tarjetas_alsa() -> Vec<(u8, String)> {
    use std::process::Command;
    use std::collections::BTreeMap;

    let mut mapa: BTreeMap<u8, String> = BTreeMap::new();

    if let Ok(out) = Command::new("aplay").arg("-l").output() {
        let texto = String::from_utf8_lossy(&out.stdout);
        for linea in texto.lines() {
            // Formato: "card N: ID [Nombre largo], device ..."
            if !linea.starts_with("card ") { continue; }
            let mut partes = linea.splitn(2, ':');
            let num_str = partes.next().unwrap_or("").trim_start_matches("card ").trim();
            let resto   = partes.next().unwrap_or("");
            if let Ok(num) = num_str.parse::<u8>() {
                // Extraer el nombre corto entre [ ]
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
    if tarjetas.is_empty() {
        vec![(0, "default".to_string())]
    } else {
        tarjetas
    }
}
