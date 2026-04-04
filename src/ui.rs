mod config;
use config::{cargar_config, guardar_config, listar_tarjetas_alsa, Config};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

// ── Paleta ────────────────────────────────────────────────────────────────────
const MORADO:   Color = Color::Rgb(138, 99,  210);
const MORADO_C: Color = Color::Rgb(180, 140, 255);
const GRIS_OSC: Color = Color::Rgb(28,  28,  35);
const GRIS_MED: Color = Color::Rgb(50,  50,  62);
const GRIS_CLR: Color = Color::Rgb(80,  80,  98);
const BLANCO:   Color = Color::Rgb(220, 218, 235);
const VERDE:    Color = Color::Rgb(80,  200, 140);
const ROJO:     Color = Color::Rgb(220, 80,  80);
const AMARILLO: Color = Color::Rgb(240, 190, 70);
const CIAN:     Color = Color::Rgb(80,  200, 220);

// ── Textos bilingües ──────────────────────────────────────────────────────────
struct T<'a> { es: &'a str, en: &'a str }
impl<'a> T<'a> {
    fn get(&self, idioma: &str) -> &'a str {
        if idioma == "en" { self.en } else { self.es }
    }
}

const TXT_TITULO:        T = T { es: "Twitch TTS Bot  —  Configuración",     en: "Twitch TTS Bot  —  Configuration"    };
const TXT_SUBTITULO:     T = T { es: "Ajusta tu bot de texto a voz",          en: "Set up your text-to-speech bot"      };
const TXT_CAMPOS:        T = T { es: " Campos ",                              en: " Fields "                            };
const TXT_GUARDAR_OK:    T = T { es: "✓  Configuración guardada",             en: "✓  Configuration saved"              };
const TXT_ERR_USUARIO:   T = T { es: "⚠  El campo Usuario está vacío",        en: "⚠  Username field is empty"          };
const TXT_ERR_TOKEN:     T = T { es: "⚠  El token debe empezar por oauth:",   en: "⚠  Token must start with oauth:"     };
const TXT_MOSTRAR:       T = T { es: "  [T] mostrar",                         en: "  [T] show"                          };
const TXT_OCULTAR:       T = T { es: "  [T] ocultar",                         en: "  [T] hide"                          };
const TXT_SALIR_TITULO:  T = T { es: " Salir ",                               en: " Exit "                              };
const TXT_SALIR_PREGUNTA:T = T { es: "  ¿Quiere salir?  ",                    en: "  Do you want to exit?  "            };
const TXT_SALIR_SI:      T = T { es: "  [S] Sí / [Enter]",                    en: "  [Y] Yes / [Enter]"                 };
const TXT_SALIR_NO:      T = T { es: "    [Cualquier otra] No",               en: "    [Any other key] No"              };
const TXT_ESCRIBIENDO:   T = T { es: " Escribiendo ",                         en: " Typing "                            };
const TXT_CONFIRMAR:     T = T { es: " confirmar  ",                          en: " confirm  "                          };
const TXT_BORRAR:        T = T { es: " borrar",                               en: " delete"                             };
const TXT_NAVEGAR:       T = T { es: " navegar  ",                            en: " navigate  "                         };
const TXT_AJUSTAR:       T = T { es: " ajustar  ",                            en: " adjust  "                           };
const TXT_EDITAR:        T = T { es: " editar  ",                             en: " edit  "                             };
const TXT_GUARDAR:       T = T { es: " guardar  ",                            en: " save  "                             };
const TXT_SALIR:         T = T { es: " salir",                                en: " exit"                               };
const TXT_MOVER:         T = T { es: " mover  ",                              en: " move  "                             };
const TXT_SELECCIONAR:   T = T { es: " seleccionar  ",                        en: " select  "                           };
const TXT_CANCELAR:      T = T { es: " cancelar",                             en: " cancel"                             };
const TXT_ELEGIR_VOZ:    T = T { es: " elegir voz",                           en: " choose voice"                       };
const TXT_VOZ_UNICA:     T = T { es: "voz única (single-speaker)",            en: "single voice (single-speaker)"       };
const TXT_ENTER_ELEGIR:  T = T { es: "[Enter] elegir voz",                    en: "[Enter] choose voice"                };
const TXT_VOZ_MODAL:     T = T { es: "Elegir voz",                            en: "Choose voice"                        };

// ── Nombres y descripciones de campos ─────────────────────────────────────────
fn nombre_campo_txt(c: Campo, idioma: &str) -> &'static str {
    match c {
        Campo::Usuario        => T { es: "Usuario",       en: "Username"    }.get(idioma),
        Campo::Token          => "Token OAuth",
        Campo::Canal          => T { es: "Canal",         en: "Channel"     }.get(idioma),
        Campo::Volumen        => T { es: "Volumen",       en: "Volume"      }.get(idioma),
        Campo::Motor          => T { es: "Motor TTS",     en: "TTS Engine"  }.get(idioma),
        Campo::PiperModelo    => T { es: "Modelo Piper",  en: "Piper Model" }.get(idioma),
        Campo::CoquiSelector  => T { es: "Voz Coqui",    en: "Coqui Voice" }.get(idioma),
        Campo::LongitudMax    => T { es: "Long. máxima",  en: "Max Length"  }.get(idioma),
        Campo::AlsaCard       => T { es: "Tarjeta audio", en: "Audio Card"  }.get(idioma),
        Campo::AnunciarUsuario => T { es: "Anunciar user", en: "Announce user" }.get(idioma),
    }
}

fn desc_campo_txt(c: Campo, idioma: &str, tiene_voces: bool) -> &'static str {
    match c {
        Campo::Usuario        => T { es: "Nombre de usuario del bot en Twitch",
                                     en: "Bot username on Twitch" }.get(idioma),
        Campo::Token          => T { es: "oauth:xxxxxxxx  —  obtenlo en twitchapps.com/tmi",
                                     en: "oauth:xxxxxxxx  —  get it at twitchapps.com/tmi" }.get(idioma),
        Campo::Canal          => T { es: "#nombre_del_canal (con la almohadilla)",
                                     en: "#channel_name (with the hash)" }.get(idioma),
        Campo::Volumen        => T { es: "← → para ajustar  |  rango 0%–200%",
                                     en: "← → to adjust  |  range 0%–200%" }.get(idioma),
        Campo::Motor          => T { es: "← → para cambiar entre piper / coqui / gtts",
                                     en: "← → to switch between piper / coqui / gtts" }.get(idioma),
        Campo::PiperModelo    => T { es: "Ruta al archivo .onnx del modelo de voz",
                                     en: "Path to the .onnx voice model file" }.get(idioma),
        Campo::CoquiSelector  => if tiene_voces {
            T { es: "← → cambiar modelo  |  Enter elegir voz",
                en: "← → change model  |  Enter choose voice" }.get(idioma)
        } else {
            T { es: "← → cambiar modelo  |  (modelo sin voces de referencia)",
                en: "← → change model  |  (model has no reference voices)" }.get(idioma)
        },
        Campo::LongitudMax    => T { es: "← → para ajustar  |  máx. caracteres por mensaje",
                                     en: "← → to adjust  |  max characters per message" }.get(idioma),
        Campo::AlsaCard       => T { es: "← → para seleccionar tarjeta de sonido",
                                     en: "← → to select audio card" }.get(idioma),
        Campo::AnunciarUsuario => T { es: "← → activar/desactivar  |  lee \"usuario dice ...\"",
                                      en: "← → enable/disable  |  reads \"user says ...\"" }.get(idioma),
    }
}

// ── Campos ────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
enum Campo {
    Usuario, Token, Canal, Volumen, Motor,
    PiperModelo, CoquiSelector,
    LongitudMax, AlsaCard, AnunciarUsuario,
}

const CAMPOS: &[Campo] = &[
    Campo::Usuario, Campo::Token, Campo::Canal,
    Campo::Volumen, Campo::Motor, Campo::PiperModelo,
    Campo::CoquiSelector,
    Campo::LongitudMax, Campo::AlsaCard, Campo::AnunciarUsuario,
];

// ── Estado global de la app ───────────────────────────────────────────────────
#[derive(PartialEq)]
enum Pantalla { Bienvenida, Config }

struct ModalVoces { modelo_idx: usize, cursor: usize }

struct App {
    cfg:            Config,
    pantalla:       Pantalla,
    idioma_cursor:  usize,          // 0 = ES, 1 = EN (solo en bienvenida)
    campo_activo:   usize,
    modo_edicion:   bool,
    token_oculto:   bool,
    mensaje:        Option<(String, bool)>,
    salir:          bool,
    confirmando:    bool,
    tarjetas_alsa:  Vec<(u8, String)>,
    modal_voces:    Option<ModalVoces>,
}

impl App {
    fn new() -> Self {
        let cfg      = cargar_config();
        let pantalla = if cfg.es_primera_vez() { Pantalla::Bienvenida } else { Pantalla::Config };
        Self {
            cfg, pantalla,
            idioma_cursor:  0,
            campo_activo:   0,
            modo_edicion:   false,
            token_oculto:   true,
            mensaje:        None,
            salir:          false,
            confirmando:    false,
            tarjetas_alsa:  listar_tarjetas_alsa(),
            modal_voces:    None,
        }
    }

    fn idioma(&self) -> &str { &self.cfg.idioma }

    fn confirmar_idioma(&mut self) {
        let idioma = if self.idioma_cursor == 0 { "es" } else { "en" };
        self.cfg.aplicar_idioma(idioma);
        self.pantalla = Pantalla::Config;
    }

    fn campo_actual(&self) -> Campo { CAMPOS[self.campo_activo] }

    // ── helpers Coqui ─────────────────────────────────────────────────────────
    fn coqui_label(&self) -> String {
        match self.cfg.coqui_activo() {
            None    => "(—)".to_string(),
            Some(m) => {
                if m.voces.is_empty() { m.nombre.clone() }
                else {
                    let voz = m.voz_actual().map(|v| v.nombre.as_str()).unwrap_or("?");
                    format!("{}  ·  {}", m.nombre, voz)
                }
            }
        }
    }

    fn coqui_modelo_prev(&mut self) {
        if self.cfg.coqui_modelos.is_empty() { return; }
        if self.cfg.coqui_modelo_idx == 0 {
            self.cfg.coqui_modelo_idx = self.cfg.coqui_modelos.len() - 1;
        } else {
            self.cfg.coqui_modelo_idx -= 1;
        }
    }

    fn coqui_modelo_next(&mut self) {
        if self.cfg.coqui_modelos.is_empty() { return; }
        self.cfg.coqui_modelo_idx =
            (self.cfg.coqui_modelo_idx + 1) % self.cfg.coqui_modelos.len();
    }

    fn abrir_modal_voces(&mut self) {
        let idx = self.cfg.coqui_modelo_idx;
        if let Some(m) = self.cfg.coqui_modelos.get(idx) {
            if !m.voces.is_empty() {
                self.modal_voces = Some(ModalVoces { modelo_idx: idx, cursor: m.voz_activa });
            }
        }
    }

    fn confirmar_voz(&mut self) {
        if let Some(mv) = self.modal_voces.take() {
            if let Some(m) = self.cfg.coqui_modelos.get_mut(mv.modelo_idx) {
                m.voz_activa = mv.cursor;
            }
        }
    }

    // ── valores y nombres de campo ────────────────────────────────────────────
    fn valor_campo(&self, c: Campo) -> String {
        match c {
            Campo::Usuario     => self.cfg.usuario.clone(),
            Campo::Token       => {
                if self.token_oculto { "•".repeat(self.cfg.token.len().min(32)) }
                else { self.cfg.token.clone() }
            }
            Campo::Canal          => self.cfg.canal.clone(),
            Campo::Volumen        => format!("{:.0}%", self.cfg.volumen * 100.0),
            Campo::Motor          => self.cfg.motor_tts.clone(),
            Campo::PiperModelo    => self.cfg.piper_modelo.clone(),
            Campo::CoquiSelector  => self.coqui_label(),
            Campo::LongitudMax    => self.cfg.longitud_max.to_string(),
            Campo::AlsaCard       => {
                let n = self.tarjetas_alsa.iter()
                    .find(|(n,_)| *n==self.cfg.alsa_card)
                    .map(|(_, s)| s.as_str()).unwrap_or("?");
                format!("card{}  ({})", self.cfg.alsa_card, n)
            }
            Campo::AnunciarUsuario => {
                let on  = T { es: "activado",   en: "enabled"  }.get(self.idioma());
                let off = T { es: "desactivado", en: "disabled" }.get(self.idioma());
                if self.cfg.anunciar_usuario { on.to_string() } else { off.to_string() }
            }
        }
    }

    // ── ajustes ───────────────────────────────────────────────────────────────
    fn editar_char(&mut self, c: char) {
        match self.campo_actual() {
            Campo::Usuario     => self.cfg.usuario.push(c),
            Campo::Token       => self.cfg.token.push(c),
            Campo::Canal       => self.cfg.canal.push(c),
            Campo::PiperModelo => self.cfg.piper_modelo.push(c),
            _                  => {}
        }
    }

    fn borrar_char(&mut self) {
        match self.campo_actual() {
            Campo::Usuario     => { self.cfg.usuario.pop(); }
            Campo::Token       => { self.cfg.token.pop(); }
            Campo::Canal       => { self.cfg.canal.pop(); }
            Campo::PiperModelo => { self.cfg.piper_modelo.pop(); }
            _                  => {}
        }
    }

    fn ajustar_izq(&mut self) {
        match self.campo_actual() {
            Campo::Volumen   => { self.cfg.volumen = (self.cfg.volumen - 0.05).clamp(0.0, 2.0); }
            Campo::Motor     => { self.cfg.motor_tts = match self.cfg.motor_tts.as_str() {
                "piper" => "gtts", "gtts" => "coqui", _ => "piper",
            }.to_string(); }
            Campo::CoquiSelector  => self.coqui_modelo_prev(),
            Campo::LongitudMax    => { if self.cfg.longitud_max > 10 { self.cfg.longitud_max -= 10; } }
            Campo::AlsaCard       => {
                if !self.tarjetas_alsa.is_empty() {
                    let idx = self.tarjetas_alsa.iter().position(|(n,_)| *n==self.cfg.alsa_card).unwrap_or(0);
                    let nuevo = if idx==0 { self.tarjetas_alsa.len()-1 } else { idx-1 };
                    self.cfg.alsa_card = self.tarjetas_alsa[nuevo].0;
                }
            }
            Campo::AnunciarUsuario => { self.cfg.anunciar_usuario = !self.cfg.anunciar_usuario; }
            _ => {}
        }
    }

    fn ajustar_der(&mut self) {
        match self.campo_actual() {
            Campo::Volumen   => { self.cfg.volumen = (self.cfg.volumen + 0.05).clamp(0.0, 2.0); }
            Campo::Motor     => { self.cfg.motor_tts = match self.cfg.motor_tts.as_str() {
                "piper" => "coqui", "coqui" => "gtts", _ => "piper",
            }.to_string(); }
            Campo::CoquiSelector  => self.coqui_modelo_next(),
            Campo::LongitudMax    => { if self.cfg.longitud_max < 500 { self.cfg.longitud_max += 10; } }
            Campo::AlsaCard       => {
                if !self.tarjetas_alsa.is_empty() {
                    let idx = self.tarjetas_alsa.iter().position(|(n,_)| *n==self.cfg.alsa_card).unwrap_or(0);
                    self.cfg.alsa_card = self.tarjetas_alsa[(idx+1)%self.tarjetas_alsa.len()].0;
                }
            }
            Campo::AnunciarUsuario => { self.cfg.anunciar_usuario = !self.cfg.anunciar_usuario; }
            _ => {}
        }
    }

    fn guardar(&mut self) {
        let id = self.idioma().to_string();
        if self.cfg.usuario.is_empty() {
            self.mensaje = Some((TXT_ERR_USUARIO.get(&id).to_string(), true)); return;
        }
        if !self.cfg.token.starts_with("oauth:") {
            self.mensaje = Some((TXT_ERR_TOKEN.get(&id).to_string(), true)); return;
        }
        if !self.cfg.canal.starts_with('#') {
            self.cfg.canal = format!("#{}", self.cfg.canal);
        }
        match guardar_config(&self.cfg) {
            Ok(_)  => self.mensaje = Some((TXT_GUARDAR_OK.get(&id).to_string(), false)),
            Err(e) => self.mensaje = Some((format!("✗  {e}"), true)),
        }
    }
}

// ── Bucle principal ───────────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut term = Terminal::new(CrosstermBackend::new(stdout))?;
    let mut app  = App::new();

    loop {
        term.draw(|f| render(f, &app))?;

        if let Event::Key(key) = event::read()? {

            // ════════════════════════════════════════════════════════════════
            //  PANTALLA BIENVENIDA
            // ════════════════════════════════════════════════════════════════
            if app.pantalla == Pantalla::Bienvenida {
                match key.code {
                    KeyCode::Up   | KeyCode::Char('k') | KeyCode::Char('K') => {
                        app.idioma_cursor = 0;
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        app.idioma_cursor = 1;
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.confirmar_idioma();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => { app.salir = true; }
                    _ => {}
                }
                if app.salir { break; }
                continue;
            }

            // ════════════════════════════════════════════════════════════════
            //  PANTALLA CONFIG
            // ════════════════════════════════════════════════════════════════

            // ── Modal voces ──────────────────────────────────────────────────
            if let Some(ref mut mv) = app.modal_voces {
                let n = app.cfg.coqui_modelos.get(mv.modelo_idx)
                    .map(|m| m.voces.len()).unwrap_or(0);
                match key.code {
                    KeyCode::Esc => { app.modal_voces = None; }
                    KeyCode::Enter => { app.confirmar_voz(); }
                    KeyCode::Up   | KeyCode::Char('k') | KeyCode::Char('K') => {
                        if mv.cursor > 0 { mv.cursor -= 1; }
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        if mv.cursor + 1 < n { mv.cursor += 1; }
                    }
                    _ => {}
                }
                if app.salir { break; }
                continue;
            }

            // ── Modal salida ─────────────────────────────────────────────────
            let id = app.idioma().to_string();
            if app.confirmando {
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S')
                    | KeyCode::Char('y') | KeyCode::Char('Y')
                    | KeyCode::Enter => { app.salir = true; }
                    _ => { app.confirmando = false; }
                }

            // ── Edición texto ─────────────────────────────────────────────────
            } else if app.modo_edicion {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => { app.modo_edicion = false; }
                    KeyCode::Char(c)   => { app.editar_char(c); }
                    KeyCode::Backspace => { app.borrar_char(); }
                    _ => {}
                }

            // ── Navegación ────────────────────────────────────────────────────
            } else {
                let _ = id;
                match key.code {
                    KeyCode::Up | KeyCode::BackTab
                    | KeyCode::Char('k') | KeyCode::Char('K') => {
                        app.campo_activo = app.campo_activo.saturating_sub(1);
                        app.mensaje = None;
                    }
                    KeyCode::Down | KeyCode::Tab
                    | KeyCode::Char('j') | KeyCode::Char('J') => {
                        app.campo_activo = (app.campo_activo + 1).min(CAMPOS.len() - 1);
                        app.mensaje = None;
                    }
                    KeyCode::Left  => { app.ajustar_izq(); app.mensaje = None; }
                    KeyCode::Right => { app.ajustar_der(); app.mensaje = None; }

                    KeyCode::Enter | KeyCode::Char(' ') => {
                        match app.campo_actual() {
                            Campo::Volumen | Campo::Motor | Campo::LongitudMax
                            | Campo::AlsaCard | Campo::AnunciarUsuario => {}
                            Campo::CoquiSelector => { app.abrir_modal_voces(); }
                            _ => { app.modo_edicion = true; }
                        }
                    }

                    KeyCode::Char('t') | KeyCode::Char('T')
                        if app.campo_actual() == Campo::Token =>
                    { app.token_oculto = !app.token_oculto; }

                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.guardar();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => { app.confirmando = true; }
                    _ => {}
                }
            }
        }
        if app.salir { break; }
    }

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    term.show_cursor()?;
    if let Some((msg, _)) = &app.mensaje { println!("{}", msg); }
    println!("{}", config::ruta_config().display());
    Ok(())
}

// ── Render principal ──────────────────────────────────────────────────────────
fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(GRIS_OSC)), area);

    match app.pantalla {
        Pantalla::Bienvenida => render_bienvenida(f, app, area),
        Pantalla::Config     => {
            let raiz = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(3)])
                .split(area);
            render_cabecera(f, app, raiz[0]);
            render_cuerpo(f, app, raiz[1]);
            render_ayuda(f, app, raiz[2]);
            if app.confirmando { render_modal_salida(f, app, area); }
            if let Some(mv) = &app.modal_voces { render_modal_voces(f, app, mv, area); }
        }
    }
}

// ── Pantalla de bienvenida ────────────────────────────────────────────────────
fn render_bienvenida(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let w = 50u16.min(area.width.saturating_sub(4));
    let h = 16u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = ratatui::layout::Rect::new(x, y, w, h);

    f.render_widget(
        Block::default()
            .title(Span::styled(
                "  🐸  Twitch TTS Bot  ",
                Style::default().fg(MORADO_C).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(MORADO))
            .style(Style::default().bg(GRIS_OSC)),
        rect,
    );

    let inner = rect.inner(Margin { horizontal: 3, vertical: 1 });
    let secs  = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // espacio
            Constraint::Length(2), // título pregunta
            Constraint::Length(1), // espacio
            Constraint::Length(3), // opción ES
            Constraint::Length(1), // espacio
            Constraint::Length(3), // opción EN
            Constraint::Min(0),    // ayuda
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Select language / Selecciona idioma",
                Style::default().fg(BLANCO).add_modifier(Modifier::BOLD)),
        ])).alignment(Alignment::Center),
        secs[1],
    );

    let opciones = [
        (0usize, "🇪🇸  Español", MORADO_C),
        (1usize, "🇬🇧  English", CIAN),
    ];

    for (i, (idx, label, color)) in opciones.iter().enumerate() {
        let sel    = app.idioma_cursor == *idx;
        let area_o = secs[3 + i * 2];
        let bloque = Block::default()
            .borders(Borders::ALL)
            .border_type(if sel { BorderType::Double } else { BorderType::Rounded })
            .border_style(Style::default().fg(if sel { *color } else { GRIS_CLR }))
            .style(Style::default().bg(if sel { GRIS_MED } else { GRIS_OSC }));
        let p = Paragraph::new(Line::from(vec![
            Span::styled(if sel { "▶  " } else { "   " },
                Style::default().fg(*color)),
            Span::styled(*label,
                Style::default()
                    .fg(if sel { *color } else { GRIS_CLR })
                    .add_modifier(if sel { Modifier::BOLD } else { Modifier::empty() })),
        ]))
        .block(bloque)
        .alignment(Alignment::Left);
        f.render_widget(p, area_o);
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↑↓/j/k", Style::default().fg(MORADO_C)),
            Span::styled("  mover / move  ", Style::default().fg(GRIS_CLR)),
            Span::styled("Enter", Style::default().fg(VERDE)),
            Span::styled("  confirmar / confirm", Style::default().fg(GRIS_CLR)),
        ])).alignment(Alignment::Center),
        secs[6],
    );
}

// ── Cabecera ──────────────────────────────────────────────────────────────────
fn render_cabecera(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let id = app.idioma();
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("  ▶  ", Style::default().fg(MORADO_C).add_modifier(Modifier::BOLD)),
                Span::styled(TXT_TITULO.get(id),
                    Style::default().fg(BLANCO).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(Span::styled(
                format!("     {}", TXT_SUBTITULO.get(id)),
                Style::default().fg(GRIS_CLR),
            )),
        ])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(MORADO)).style(Style::default().bg(GRIS_OSC)))
        .alignment(Alignment::Left),
        area,
    );
}

// ── Cuerpo ────────────────────────────────────────────────────────────────────
fn render_cuerpo(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    render_lista_campos(f, app, cols[0]);
    render_detalle_campo(f, app, cols[1]);
}

fn render_lista_campos(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let id    = app.idioma();
    let items: Vec<ListItem> = CAMPOS.iter().enumerate().map(|(i, &c)| {
        let activo   = i == app.campo_activo;
        let valor    = app.valor_campo(c);
        let nombre   = nombre_campo_txt(c, id);
        let truncado = if valor.len() > 22 { format!("{}…", &valor[..21]) } else { valor };
        let linea = Line::from(vec![
            Span::styled(if activo { "▶ " } else { "  " },
                Style::default().fg(if activo { MORADO_C } else { GRIS_CLR })),
            Span::styled(format!("{:<14}", nombre),
                Style::default()
                    .fg(if activo { BLANCO } else { Color::Rgb(160,155,180) })
                    .add_modifier(if activo { Modifier::BOLD } else { Modifier::empty() })),
            Span::styled(truncado,
                Style::default().fg(if activo { MORADO_C } else { GRIS_CLR })),
        ]);
        if activo { ListItem::new(linea).style(Style::default().bg(GRIS_MED)) }
        else      { ListItem::new(linea) }
    }).collect();

    f.render_widget(
        List::new(items).block(Block::default()
            .title(Span::styled(TXT_CAMPOS.get(id), Style::default().fg(MORADO_C)))
            .borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(GRIS_CLR))
            .style(Style::default().bg(GRIS_OSC))),
        area,
    );
}

fn render_detalle_campo(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let campo = app.campo_actual();
    let id    = app.idioma();
    let tiene_voces = app.cfg.coqui_activo()
        .map(|m| !m.voces.is_empty()).unwrap_or(false);

    f.render_widget(
        Block::default()
            .title(Span::styled(
                format!(" {} ", nombre_campo_txt(campo, id)),
                Style::default().fg(MORADO_C).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(MORADO))
            .style(Style::default().bg(GRIS_OSC)),
        area,
    );
    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
    let secs  = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1),
                      Constraint::Length(3), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(
        Paragraph::new(desc_campo_txt(campo, id, tiene_voces))
            .style(Style::default().fg(GRIS_CLR)).wrap(Wrap { trim: true }),
        secs[0],
    );

    match campo {
        Campo::Volumen       => render_gauge_volumen(f, app, secs[2]),
        Campo::Motor         => render_toggle_motor(f, app, secs[2]),
        Campo::LongitudMax   => render_gauge_longitud(f, app, secs[2]),
        Campo::AlsaCard      => render_selector_alsa(f, app, secs[2]),
        Campo::CoquiSelector => render_coqui_selector(f, app, secs[2], inner),
        _                    => render_input_texto(f, app, campo, secs[2]),
    }

    if let Some((msg, es_error)) = &app.mensaje {
        f.render_widget(
            Paragraph::new(msg.as_str())
                .style(Style::default().fg(if *es_error { ROJO } else { VERDE })),
            secs[4],
        );
    }
}

// ── Widgets ───────────────────────────────────────────────────────────────────
fn render_input_texto(f: &mut Frame, app: &App, campo: Campo, area: ratatui::layout::Rect) {
    let id       = app.idioma();
    let editando = app.modo_edicion && app.campo_actual() == campo;
    let valor    = app.valor_campo(campo);
    let extra    = if campo == Campo::Token {
        if app.token_oculto { TXT_MOSTRAR.get(id) } else { TXT_OCULTAR.get(id) }
    } else { "" };
    f.render_widget(
        Paragraph::new(if editando { format!("{}_", valor) } else { valor })
            .block(Block::default().borders(Borders::ALL)
                .border_type(if editando { BorderType::Double } else { BorderType::Rounded })
                .border_style(Style::default().fg(if editando { AMARILLO } else { GRIS_CLR }))
                .title(Span::styled(extra, Style::default().fg(GRIS_CLR))))
            .style(Style::default().fg(if editando { AMARILLO } else { BLANCO })),
        area,
    );
}

fn render_gauge_volumen(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let pct   = (app.cfg.volumen * 100.0) as u16;
    let color = match pct { 0..=50 => VERDE, 51..=100 => MORADO_C, _ => AMARILLO };
    f.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)))
            .gauge_style(Style::default().fg(color).bg(GRIS_MED))
            .ratio((app.cfg.volumen / 2.0).clamp(0.0, 1.0) as f64)
            .label(Span::styled(format!("{pct}%"),
                Style::default().fg(BLANCO).add_modifier(Modifier::BOLD))),
        area,
    );
}

fn render_gauge_longitud(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    f.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)))
            .gauge_style(Style::default().fg(MORADO).bg(GRIS_MED))
            .ratio(app.cfg.longitud_max as f64 / 500.0)
            .label(Span::styled(format!("{} chars", app.cfg.longitud_max),
                Style::default().fg(BLANCO).add_modifier(Modifier::BOLD))),
        area,
    );
}

fn render_toggle_motor(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let m   = app.cfg.motor_tts.as_str();
    let btn = |label: &'static str, on: bool, col: Color| Span::styled(label,
        Style::default().fg(if on { GRIS_OSC } else { GRIS_CLR })
            .bg(if on { col } else { GRIS_MED })
            .add_modifier(if on { Modifier::BOLD } else { Modifier::empty() }));
    f.render_widget(
        Paragraph::new(vec![Line::default(), Line::from(vec![
            btn("  [ Piper ]  ", m=="piper", MORADO_C), Span::raw("  "),
            btn("  [ Coqui ]  ", m=="coqui", AMARILLO), Span::raw("  "),
            btn("  [ gTTS ]  ",  m=="gtts",  VERDE),
        ])])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(GRIS_CLR)))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_selector_alsa(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let nombre = app.tarjetas_alsa.iter()
        .find(|(n,_)| *n==app.cfg.alsa_card)
        .map(|(_,s)| s.as_str()).unwrap_or("?");
    f.render_widget(
        Paragraph::new(vec![Line::default(), Line::from(vec![
            Span::styled("  ◀  ", Style::default().fg(GRIS_CLR)),
            Span::styled(format!("  card{}  —  {}  ", app.cfg.alsa_card, nombre),
                Style::default().fg(BLANCO).bg(GRIS_MED).add_modifier(Modifier::BOLD)),
            Span::styled("  ▶  ", Style::default().fg(GRIS_CLR)),
        ])])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(GRIS_CLR)))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_coqui_selector(
    f: &mut Frame, app: &App,
    area: ratatui::layout::Rect,
    full: ratatui::layout::Rect,
) {
    let modelo_label = app.cfg.coqui_activo()
        .map(|m| m.nombre.clone()).unwrap_or_else(|| "(—)".to_string());
    f.render_widget(
        Paragraph::new(vec![Line::default(), Line::from(vec![
            Span::styled("  ◀  ", Style::default().fg(GRIS_CLR)),
            Span::styled(format!("  {}  ", modelo_label),
                Style::default().fg(BLANCO).bg(GRIS_MED).add_modifier(Modifier::BOLD)),
            Span::styled("  ▶  ", Style::default().fg(GRIS_CLR)),
        ])])
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            .border_style(Style::default().fg(GRIS_CLR)))
        .alignment(Alignment::Center),
        area,
    );

    if let Some(m) = app.cfg.coqui_activo() {
        let det = ratatui::layout::Rect {
            x: full.x, y: area.y + area.height,
            width: full.width,
            height: full.height.saturating_sub(area.y - full.y + area.height),
        };
        let id = app.idioma();
        let mut lineas: Vec<Line> = vec![Line::from("")];
        if m.voces.is_empty() {
            lineas.push(Line::from(Span::styled(
                TXT_VOZ_UNICA.get(id), Style::default().fg(GRIS_CLR))));
        } else {
            let voz = m.voz_actual().map(|v| v.nombre.as_str()).unwrap_or("?");
            lineas.push(Line::from(vec![
                Span::styled(T { es: "  Voz  : ", en: "  Voice: " }.get(id),
                    Style::default().fg(GRIS_CLR)),
                Span::styled(voz,
                    Style::default().fg(CIAN).add_modifier(Modifier::BOLD)),
            ]));
            lineas.push(Line::from(""));
            lineas.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("[Enter]", Style::default().fg(AMARILLO)),
                Span::raw(" "),
                Span::styled(TXT_ELEGIR_VOZ.get(id), Style::default().fg(GRIS_CLR)),
            ]));
        }
        f.render_widget(Paragraph::new(lineas), det);
    }
}

// ── Modal voces ───────────────────────────────────────────────────────────────
fn render_modal_voces(
    f: &mut Frame, app: &App,
    mv: &ModalVoces,
    area: ratatui::layout::Rect,
) {
    let modelo = match app.cfg.coqui_modelos.get(mv.modelo_idx) {
        Some(m) => m, None => return,
    };
    let id  = app.idioma();
    let w   = 44u16.min(area.width.saturating_sub(4));
    let h   = (modelo.voces.len() as u16 + 6).min(area.height.saturating_sub(4));
    let x   = area.x + (area.width.saturating_sub(w)) / 2;
    let y   = area.y + (area.height.saturating_sub(h)) / 2;
    let rec = ratatui::layout::Rect::new(x, y, w, h);

    f.render_widget(Clear, rec);
    f.render_widget(
        Block::default()
            .title(Span::styled(
                format!(" 🎙 {} — {} ", modelo.nombre, TXT_VOZ_MODAL.get(id)),
                Style::default().fg(AMARILLO).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL).border_type(BorderType::Double)
            .border_style(Style::default().fg(AMARILLO))
            .style(Style::default().bg(GRIS_OSC)),
        rec,
    );

    let inner = rec.inner(Margin { horizontal: 2, vertical: 1 });
    let secs  = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let items: Vec<ListItem> = modelo.voces.iter().enumerate().map(|(i, v)| {
        let es_cursor = i == mv.cursor;
        let es_activa = i == modelo.voz_activa;
        let linea = Line::from(vec![
            Span::styled(if es_cursor { "▶" } else { " " }, Style::default().fg(AMARILLO)),
            Span::styled(if es_activa { " ✓ " } else { "   " }, Style::default().fg(VERDE)),
            Span::styled(v.nombre.clone(),
                Style::default()
                    .fg(if es_cursor { AMARILLO } else if es_activa { VERDE } else { BLANCO })
                    .add_modifier(if es_cursor || es_activa { Modifier::BOLD } else { Modifier::empty() })),
        ]);
        if es_cursor { ListItem::new(linea).style(Style::default().bg(GRIS_MED)) }
        else         { ListItem::new(linea) }
    }).collect();

    f.render_widget(List::new(items), secs[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↑↓/j/k", Style::default().fg(MORADO_C)),
            Span::styled(TXT_MOVER.get(id), Style::default().fg(BLANCO)),
            Span::styled("Enter", Style::default().fg(VERDE)),
            Span::styled(TXT_SELECCIONAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("Esc", Style::default().fg(ROJO)),
            Span::styled(TXT_CANCELAR.get(id), Style::default().fg(BLANCO)),
        ])).alignment(Alignment::Center),
        secs[1],
    );
}

// ── Ayuda ─────────────────────────────────────────────────────────────────────
fn render_ayuda(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let id     = app.idioma();
    let atajos = if app.modo_edicion {
        vec![
            Span::styled(TXT_ESCRIBIENDO.get(id),
                Style::default().fg(AMARILLO).add_modifier(Modifier::BOLD)),
            Span::styled("  Enter/Esc", Style::default().fg(GRIS_CLR)),
            Span::styled(TXT_CONFIRMAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("  Backspace", Style::default().fg(GRIS_CLR)),
            Span::styled(TXT_BORRAR.get(id), Style::default().fg(BLANCO)),
        ]
    } else {
        vec![
            Span::styled(" ↑↓/j/k", Style::default().fg(MORADO_C)),
            Span::styled(TXT_NAVEGAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("←→", Style::default().fg(MORADO_C)),
            Span::styled(TXT_AJUSTAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("Enter", Style::default().fg(MORADO_C)),
            Span::styled(TXT_EDITAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("Ctrl+S", Style::default().fg(VERDE)),
            Span::styled(TXT_GUARDAR.get(id), Style::default().fg(BLANCO)),
            Span::styled("Q/Esc", Style::default().fg(ROJO)),
            Span::styled(TXT_SALIR.get(id), Style::default().fg(BLANCO)),
        ]
    };
    f.render_widget(
        Paragraph::new(Line::from(atajos))
            .block(Block::default().borders(Borders::TOP)
                .border_style(Style::default().fg(GRIS_CLR))
                .style(Style::default().bg(GRIS_OSC)))
            .alignment(Alignment::Center),
        area,
    );
}

// ── Modal salida ──────────────────────────────────────────────────────────────
fn render_modal_salida(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let id = app.idioma();
    let w=44u16; let h=6u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let m = ratatui::layout::Rect::new(x, y, w, h);
    f.render_widget(Clear, m);
    f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(0,0,0))), m);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(TXT_SALIR_PREGUNTA.get(id),
                Style::default().fg(AMARILLO))),
            Line::from(""),
            Line::from(vec![
                Span::styled(TXT_SALIR_SI.get(id), Style::default().fg(ROJO)),
                Span::styled(TXT_SALIR_NO.get(id), Style::default().fg(VERDE)),
            ]),
        ])
        .block(Block::default()
            .title(Span::styled(TXT_SALIR_TITULO.get(id),
                Style::default().fg(AMARILLO).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL).border_type(BorderType::Double)
            .border_style(Style::default().fg(AMARILLO))
            .style(Style::default().bg(GRIS_OSC)))
        .alignment(Alignment::Center),
        m,
    );
}
