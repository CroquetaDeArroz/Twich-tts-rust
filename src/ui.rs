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

// ── Paleta ──────────────────────────────────────────────────────────────────
const MORADO:   Color = Color::Rgb(138, 99, 210);
const MORADO_C: Color = Color::Rgb(180, 140, 255);
const GRIS_OSC: Color = Color::Rgb(28,  28,  35);
const GRIS_MED: Color = Color::Rgb(50,  50,  62);
const GRIS_CLR: Color = Color::Rgb(80,  80,  98);
const BLANCO:   Color = Color::Rgb(220, 218, 235);
const VERDE:    Color = Color::Rgb(80,  200, 140);
const ROJO:     Color = Color::Rgb(220, 80,  80);
const AMARILLO: Color = Color::Rgb(240, 190, 70);

// ── Campos del formulario ────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
enum Campo {
    Usuario,
    Token,
    Canal,
    Volumen,
    Motor,
    PiperModelo,
    LongitudMax,
    AlsaCard,
    AnunciarUsuario,
}

const CAMPOS: &[Campo] = &[
    Campo::Usuario,
    Campo::Token,
    Campo::Canal,
    Campo::Volumen,
    Campo::Motor,
    Campo::PiperModelo,
    Campo::LongitudMax,
    Campo::AlsaCard,
    Campo::AnunciarUsuario,
];

// ── Estado de la app ─────────────────────────────────────────────────────────
struct App {
    cfg:           Config,
    campo_activo:  usize,
    modo_edicion:  bool,
    token_oculto:  bool,
    mensaje:       Option<(String, bool)>,
    salir:         bool,
    confirmando:   bool,
    tarjetas_alsa: Vec<(u8, String)>,
}

impl App {
    fn new() -> Self {
        Self {
            cfg:           cargar_config(),
            campo_activo:  0,
            modo_edicion:  false,
            token_oculto:  true,
            mensaje:       None,
            salir:         false,
            confirmando:   false,
            tarjetas_alsa: listar_tarjetas_alsa(),
        }
    }

    fn campo_actual(&self) -> Campo {
        CAMPOS[self.campo_activo]
    }

    fn valor_campo(&self, c: Campo) -> String {
        match c {
            Campo::Usuario     => self.cfg.usuario.clone(),
            Campo::Token       => {
                if self.token_oculto {
                    "•".repeat(self.cfg.token.len().min(32))
                } else {
                    self.cfg.token.clone()
                }
            }
            Campo::Canal       => self.cfg.canal.clone(),
            Campo::Volumen     => format!("{:.0}%", self.cfg.volumen * 100.0),
            Campo::Motor       => self.cfg.motor_tts.clone(),
            Campo::PiperModelo => self.cfg.piper_modelo.clone(),
            Campo::LongitudMax => self.cfg.longitud_max.to_string(),
            Campo::AlsaCard    => {
                let nombre = self.tarjetas_alsa.iter()
                    .find(|(n, _)| *n == self.cfg.alsa_card)
                    .map(|(_, s)| s.as_str())
                    .unwrap_or("desconocida");
                format!("card{}  ({})", self.cfg.alsa_card, nombre)
            }
            Campo::AnunciarUsuario => if self.cfg.anunciar_usuario { "activado".to_string() } else { "desactivado".to_string() },

        }
    }

    fn nombre_campo(&self, c: Campo) -> &'static str {
        match c {
            Campo::Usuario     => "Usuario",
            Campo::Token       => "Token OAuth",
            Campo::Canal       => "Canal",
            Campo::Volumen     => "Volumen",
            Campo::Motor       => "Motor TTS",
            Campo::PiperModelo => "Modelo Piper",
            Campo::LongitudMax => "Long. máxima",
            Campo::AlsaCard    => "Tarjeta audio",
            Campo::AnunciarUsuario => "Anunciar user",
        }
    }

    fn descripcion_campo(&self, c: Campo) -> &'static str {
        match c {
            Campo::Usuario     => "Nombre de usuario del bot en Twitch",
            Campo::Token       => "oauth:xxxxxxxx  —  obtenlo en twitchapps.com/tmi",
            Campo::Canal       => "#nombre_del_canal (con la almohadilla)",
            Campo::Volumen     => "← → para ajustar  |  rango 0%–200%",
            Campo::Motor       => "← → para cambiar entre piper / gtts",
            Campo::PiperModelo => "Ruta al archivo .onnx del modelo de voz",
            Campo::LongitudMax => "← → para ajustar  |  máx. caracteres por mensaje",
            Campo::AlsaCard    => "← → para seleccionar tarjeta de sonido",
            Campo::AnunciarUsuario => "← → para activar/desactivar  |  lee \"usuario dice ...\"",
        }
    }

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
            Campo::Volumen => {
                self.cfg.volumen = (self.cfg.volumen - 0.05).clamp(0.0, 2.0);
            }
            Campo::Motor => {
                self.cfg.motor_tts = if self.cfg.motor_tts == "piper" {
                    "gtts".to_string()
                } else {
                    "piper".to_string()
                };
            }
            Campo::LongitudMax => {
                if self.cfg.longitud_max > 10 {
                    self.cfg.longitud_max -= 10;
                }
            }
            Campo::AlsaCard => {
                if !self.tarjetas_alsa.is_empty() {
                    let idx = self.tarjetas_alsa.iter()
                        .position(|(n, _)| *n == self.cfg.alsa_card)
                        .unwrap_or(0);
                    let nuevo = if idx == 0 { self.tarjetas_alsa.len() - 1 } else { idx - 1 };
                    self.cfg.alsa_card = self.tarjetas_alsa[nuevo].0;
                }
            }
            Campo::AnunciarUsuario => {    
                self.cfg.anunciar_usuario = !self.cfg.anunciar_usuario;
            }
            _ => {}
        }
    }

    fn ajustar_der(&mut self) {
        match self.campo_actual() {
            Campo::Volumen => {
                self.cfg.volumen = (self.cfg.volumen + 0.05).clamp(0.0, 2.0);
            }
            Campo::Motor => {
                self.cfg.motor_tts = if self.cfg.motor_tts == "piper" {
                    "gtts".to_string()
                } else {
                    "piper".to_string()
                };
            }
            Campo::LongitudMax => {
                if self.cfg.longitud_max < 500 {
                    self.cfg.longitud_max += 10;
                }
            }
            Campo::AlsaCard => {
                if !self.tarjetas_alsa.is_empty() {
                    let idx = self.tarjetas_alsa.iter()
                        .position(|(n, _)| *n == self.cfg.alsa_card)
                        .unwrap_or(0);
                    let nuevo = (idx + 1) % self.tarjetas_alsa.len();
                    self.cfg.alsa_card = self.tarjetas_alsa[nuevo].0;
                }
            }
            Campo::AnunciarUsuario => {          
                self.cfg.anunciar_usuario = !self.cfg.anunciar_usuario;
            }
            _ => {}
        }
    }

    fn guardar(&mut self) {
        if self.cfg.usuario.is_empty() {
            self.mensaje = Some(("⚠  El campo Usuario está vacío".into(), true));
            return;
        }
        if !self.cfg.token.starts_with("oauth:") {
            self.mensaje = Some(("⚠  El token debe empezar por oauth:".into(), true));
            return;
        }
        if !self.cfg.canal.starts_with('#') {
            self.cfg.canal = format!("#{}", self.cfg.canal);
        }
        match guardar_config(&self.cfg) {
            Ok(_)  => self.mensaje = Some(("✓  Configuración guardada".into(), false)),
            Err(e) => self.mensaje = Some((format!("✗  Error al guardar: {e}"), true)),
        }
    }
}

// ── Bucle principal ──────────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        term.draw(|f| render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if app.confirmando {
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                        app.salir = true;
                    }
                    _ => { app.confirmando = false; }
                }
            } else if app.modo_edicion {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => { app.modo_edicion = false; }
                    KeyCode::Char(c) => { app.editar_char(c); }
                    KeyCode::Backspace => { app.borrar_char(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::BackTab => {
                        app.campo_activo = app.campo_activo.saturating_sub(1);
                        app.mensaje = None;
                    }
                    KeyCode::Down | KeyCode::Tab => {
                        app.campo_activo = (app.campo_activo + 1).min(CAMPOS.len() - 1);
                        app.mensaje = None;
                    }
                    KeyCode::Left  => { app.ajustar_izq(); app.mensaje = None; }
                    KeyCode::Right => { app.ajustar_der(); app.mensaje = None; }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        match app.campo_actual() {
                            Campo::Volumen | Campo::Motor | Campo::LongitudMax | Campo::AlsaCard => {}
                            _ => { app.modo_edicion = true; }
                        }
                    }
                    KeyCode::Char('t') | KeyCode::Char('T')
                        if app.campo_actual() == Campo::Token =>
                    {
                        app.token_oculto = !app.token_oculto;
                    }
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.guardar();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.confirmando = true;
                    }
                    _ => {}
                }
            }
        }

        if app.salir { break; }
    }

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    if let Some((msg, _)) = &app.mensaje {
        println!("{}", msg);
    }
    println!("Config en: {}", config::ruta_config().display());
    Ok(())
}

// ── Render ───────────────────────────────────────────────────────────────────
fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    f.render_widget(
        Block::default().style(Style::default().bg(GRIS_OSC)),
        area,
    );

    let raiz = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_cabecera(f, raiz[0]);
    render_cuerpo(f, app, raiz[1]);
    render_ayuda(f, app, raiz[2]);

    if app.confirmando {
        render_modal_salida(f, area);
    }
}

fn render_cabecera(f: &mut Frame, area: ratatui::layout::Rect) {
    let bloque = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MORADO))
        .style(Style::default().bg(GRIS_OSC));

    let titulo = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  ▶  ", Style::default().fg(MORADO_C).add_modifier(Modifier::BOLD)),
            Span::styled(
                "Twitch TTS Bot",
                Style::default().fg(BLANCO).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  —  Configuración", Style::default().fg(GRIS_CLR)),
        ]),
        Line::from(vec![
            Span::styled(
                "     Ajusta tu bot de texto a voz para Twitch",
                Style::default().fg(GRIS_CLR),
            ),
        ]),
    ])
    .block(bloque)
    .alignment(Alignment::Left);

    f.render_widget(titulo, area);
}

fn render_cuerpo(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_lista_campos(f, app, cols[0]);
    render_detalle_campo(f, app, cols[1]);
}

fn render_lista_campos(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = CAMPOS
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let activo = i == app.campo_activo;
            let valor  = app.valor_campo(c);
            let nombre = app.nombre_campo(c);

            let prefijo = if activo { "▶ " } else { "  " };
            let truncado = if valor.len() > 22 {
                format!("{}…", &valor[..21])
            } else {
                valor.clone()
            };

            let linea = Line::from(vec![
                Span::styled(
                    prefijo,
                    Style::default().fg(if activo { MORADO_C } else { GRIS_CLR }),
                ),
                Span::styled(
                    format!("{:<14}", nombre),
                    Style::default()
                        .fg(if activo { BLANCO } else { Color::Rgb(160, 155, 180) })
                        .add_modifier(if activo { Modifier::BOLD } else { Modifier::empty() }),
                ),
                Span::styled(
                    truncado,
                    Style::default().fg(if activo { MORADO_C } else { GRIS_CLR }),
                ),
            ]);

            if activo {
                ListItem::new(linea).style(Style::default().bg(GRIS_MED))
            } else {
                ListItem::new(linea)
            }
        })
        .collect();

    let lista = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(" Campos ", Style::default().fg(MORADO_C)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR))
                .style(Style::default().bg(GRIS_OSC)),
        );

    f.render_widget(lista, area);
}

fn render_detalle_campo(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let campo = app.campo_actual();

    let bloq = Block::default()
        .title(Span::styled(
            format!(" {} ", app.nombre_campo(campo)),
            Style::default().fg(MORADO_C).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MORADO))
        .style(Style::default().bg(GRIS_OSC));

    f.render_widget(bloq, area);

    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });

    let secciones = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let desc = Paragraph::new(app.descripcion_campo(campo))
        .style(Style::default().fg(GRIS_CLR))
        .wrap(Wrap { trim: true });
    f.render_widget(desc, secciones[0]);

    match campo {
        Campo::Volumen     => render_gauge_volumen(f, app, secciones[2]),
        Campo::Motor       => render_toggle_motor(f, app, secciones[2]),
        Campo::LongitudMax => render_gauge_longitud(f, app, secciones[2]),
        Campo::AlsaCard    => render_selector_alsa(f, app, secciones[2]),
        _                  => render_input_texto(f, app, campo, secciones[2]),
    }

    if let Some((msg, es_error)) = &app.mensaje {
        let color  = if *es_error { ROJO } else { VERDE };
        let estado = Paragraph::new(msg.as_str()).style(Style::default().fg(color));
        f.render_widget(estado, secciones[4]);
    }
}

fn render_input_texto(f: &mut Frame, app: &App, campo: Campo, area: ratatui::layout::Rect) {
    let editando = app.modo_edicion && app.campo_actual() == campo;
    let valor    = app.valor_campo(campo);

    let texto_mostrado = if editando {
        format!("{}_", valor)
    } else {
        valor.clone()
    };

    let extra = if campo == Campo::Token {
        if app.token_oculto { "  [T] mostrar" } else { "  [T] ocultar" }
    } else { "" };

    let bloq = Block::default()
        .borders(Borders::ALL)
        .border_type(if editando { BorderType::Double } else { BorderType::Rounded })
        .border_style(Style::default().fg(if editando { AMARILLO } else { GRIS_CLR }))
        .title(Span::styled(extra, Style::default().fg(GRIS_CLR)));

    let p = Paragraph::new(texto_mostrado)
        .block(bloq)
        .style(Style::default().fg(if editando { AMARILLO } else { BLANCO }));

    f.render_widget(p, area);
}

fn render_gauge_volumen(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let pct = (app.cfg.volumen * 100.0) as u16;
    let color = match pct {
        0..=50   => VERDE,
        51..=100 => MORADO_C,
        _        => AMARILLO,
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)),
        )
        .gauge_style(Style::default().fg(color).bg(GRIS_MED))
        .ratio((app.cfg.volumen / 2.0).clamp(0.0, 1.0) as f64)
        .label(Span::styled(
            format!("{pct}%"),
            Style::default().fg(BLANCO).add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

fn render_gauge_longitud(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let max = 500usize;
    let pct = app.cfg.longitud_max as f64 / max as f64;

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)),
        )
        .gauge_style(Style::default().fg(MORADO).bg(GRIS_MED))
        .ratio(pct)
        .label(Span::styled(
            format!("{} chars", app.cfg.longitud_max),
            Style::default().fg(BLANCO).add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

fn render_toggle_motor(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let es_piper = app.cfg.motor_tts == "piper";

    let linea = Line::from(vec![
        Span::styled(
            "  [ Piper ]  ",
            Style::default()
                .fg(if es_piper { GRIS_OSC } else { GRIS_CLR })
                .bg(if es_piper { MORADO_C } else { GRIS_MED })
                .add_modifier(if es_piper { Modifier::BOLD } else { Modifier::empty() }),
        ),
        Span::raw("  "),
        Span::styled(
            "  [ gTTS ]  ",
            Style::default()
                .fg(if !es_piper { GRIS_OSC } else { GRIS_CLR })
                .bg(if !es_piper { VERDE } else { GRIS_MED })
                .add_modifier(if !es_piper { Modifier::BOLD } else { Modifier::empty() }),
        ),
    ]);

    let p = Paragraph::new(vec![Line::default(), linea])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)),
        )
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}

fn render_selector_alsa(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let nombre = app.tarjetas_alsa.iter()
        .find(|(n, _)| *n == app.cfg.alsa_card)
        .map(|(_, s)| s.as_str())
        .unwrap_or("desconocida");

    let linea = Line::from(vec![
        Span::styled("  ◀  ", Style::default().fg(GRIS_CLR)),
        Span::styled(
            format!("  card{}  —  {}  ", app.cfg.alsa_card, nombre),
            Style::default().fg(BLANCO).bg(GRIS_MED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ▶  ", Style::default().fg(GRIS_CLR)),
    ]);

    let p = Paragraph::new(vec![Line::default(), linea])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GRIS_CLR)),
        )
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}

fn render_ayuda(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let atajos = if app.modo_edicion {
        vec![
            Span::styled(" Escribiendo ", Style::default().fg(AMARILLO).add_modifier(Modifier::BOLD)),
            Span::styled("  Enter/Esc", Style::default().fg(GRIS_CLR)),
            Span::styled(" confirmar ", Style::default().fg(BLANCO)),
            Span::styled("  Backspace", Style::default().fg(GRIS_CLR)),
            Span::styled(" borrar", Style::default().fg(BLANCO)),
        ]
    } else {
        vec![
            Span::styled(" ↑↓", Style::default().fg(MORADO_C)),
            Span::styled(" navegar  ", Style::default().fg(BLANCO)),
            Span::styled("←→", Style::default().fg(MORADO_C)),
            Span::styled(" ajustar  ", Style::default().fg(BLANCO)),
            Span::styled("Enter", Style::default().fg(MORADO_C)),
            Span::styled(" editar  ", Style::default().fg(BLANCO)),
            Span::styled("Ctrl+S", Style::default().fg(VERDE)),
            Span::styled(" guardar  ", Style::default().fg(BLANCO)),
            Span::styled("Q/Esc", Style::default().fg(ROJO)),
            Span::styled(" salir", Style::default().fg(BLANCO)),
        ]
    };

    let ayuda = Paragraph::new(Line::from(atajos))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(GRIS_CLR))
                .style(Style::default().bg(GRIS_OSC)),
        )
        .alignment(Alignment::Center);

    f.render_widget(ayuda, area);
}

fn render_modal_salida(f: &mut Frame, area: ratatui::layout::Rect) {
    let overlay = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));

    let w = 40u16;
    let h = 6u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let modal_area = ratatui::layout::Rect::new(x, y, w, h);

    f.render_widget(Clear, modal_area);
    f.render_widget(overlay, modal_area);

    let modal = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ¿Quiere salir?  ", Style::default().fg(AMARILLO)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [S] Sí / [Enter]", Style::default().fg(ROJO)),
            Span::styled("    [Cualquier otra] No", Style::default().fg(VERDE)),
        ]),
    ])
    .block(
        Block::default()
            .title(Span::styled(" Salir ", Style::default().fg(AMARILLO).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(AMARILLO))
            .style(Style::default().bg(GRIS_OSC)),
    )
    .alignment(Alignment::Center);

    f.render_widget(modal, modal_area);
}
