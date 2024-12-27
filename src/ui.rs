use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::app::{App, Focus, Mode, PopupFocus, SearchTarget};
use crate::helper::KeyBindings;
use crate::search::SearchSource;
use crate::export::ExportFormat;
use std::env;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ].as_ref())
        .split(frame.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.layout_sizes.left_panel_width),
            Constraint::Percentage(app.layout_sizes.right_panel_width),
        ].as_ref())
        .split(chunks[0]);

    draw_sections(frame, app, main_chunks[0]);
    draw_details(frame, app, main_chunks[1]);
    draw_shortcuts(frame, app, chunks[1]);

    match app.mode {
        Mode::Adding | Mode::Editing => draw_edit_popup(frame, app),
        Mode::Help => draw_help_popup(frame, app),
        Mode::Searching => draw_search_popup(frame, app),
        Mode::Exporting => draw_export_popup(frame, app),
        _ => {}
    }
}

fn draw_sections(frame: &mut Frame, app: &App, area: Rect) {
    let mut state = ListState::default();
    state.select(app.selected_section);

    let block = Block::default()
        .title(Span::styled(
            "📚 Secciones",
            Style::default().fg(if app.focus == Focus::Sections {
                Color::Yellow
            } else {
                Color::White
            })
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::Sections {
            Color::Yellow
        } else {
            Color::White
        }));

    let items: Vec<ListItem> = app.sections.iter()
        .map(|section| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    &section.title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(if app.focus == Focus::Sections {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                )
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_details(frame: &mut Frame, app: &App, area: Rect) {
    let mut state = ListState::default();
    state.select(app.selected_detail);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(if let Some(section_idx) = app.selected_section {
            app.sections.get(section_idx)
                .map(|s| s.details.len())
                .unwrap_or(0)
        } else {
            0
        })
        .position(app.selected_detail.unwrap_or(0));

    let block = Block::default()
        .title("Detalles")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::Details {
            Color::Yellow
        } else {
            Color::White
        }));

    let items: Vec<ListItem> = if let Some(section_idx) = app.selected_section {
        if let Some(section) = app.sections.get(section_idx) {
            section.details.iter()
                .map(|detail| {
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                &detail.title,
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD)
                            ),
                        ]),
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                &detail.description,
                                Style::default().fg(Color::White)
                            ),
                        ]),
                        Line::from(vec![
                            Span::raw("  "),
                            Span::styled(
                                &detail.created_at,
                                Style::default()
                                    .fg(Color::DarkGray)
                                    .add_modifier(Modifier::ITALIC)
                            ),
                        ]),
                        Line::from(""),
                    ])
                })
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    let list_area = Rect {
        width: area.width - 1,
        ..area
    };
    frame.render_stateful_widget(list, list_area, &mut state);

    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area,
        &mut scrollbar_state,
    );
}

fn draw_shortcuts(frame: &mut Frame, app: &App, area: Rect) {
    let bindings = KeyBindings::new(app.focus.clone());
    let shortcuts = Line::from(
        bindings
            .commands
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(
                        key,
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(": "),
                    Span::raw(desc),
                    Span::raw(" | "),
                ]
            })
            .collect::<Vec<Span>>()
    );

    let paragraph = Paragraph::new(shortcuts)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_edit_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 50, frame.size());
    
    let title = match app.focus {
        Focus::Sections => "Nueva Sección".to_string(),
        Focus::Details => {
            if let Some(section_idx) = app.selected_section {
                if let Some(section) = app.sections.get(section_idx) {
                    format!("Nuevo Detalle en \"{}\"", section.title)
                } else {
                    "Nuevo Detalle".to_string()
                }
            } else {
                "Nuevo Detalle".to_string()
            }
        }
        _ => "Nuevo".to_string(),
    };

    let title_style = if app.popup_focus == PopupFocus::Title {
        Style::default().bg(Color::Yellow).fg(Color::Black)
    } else {
        Style::default().fg(Color::White)
    };

    let desc_style = if app.popup_focus == PopupFocus::Description {
        Style::default().bg(Color::Yellow).fg(Color::Black)
    } else {
        Style::default().fg(Color::White)
    };

    let content = match app.focus {
        Focus::Sections => {
            let input_text = format!("{}_", app.input_buffer);
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("Título: "),
                    Span::styled(
                        input_text,
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("(El ícono 📁 se agregará automáticamente si no lo incluyes)")
                ]),
            ]
        }
        Focus::Details => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("T��tulo: "),
                    Span::styled(
                        format!("{}_", app.input_buffer),
                        title_style
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Descripción: "),
                    Span::styled(
                        format!("{}_", app.description_buffer),
                        desc_style
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Tab", Style::default().fg(Color::Green)),
                    Span::raw(" para cambiar entre campos"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::raw(" para guardar, "),
                    Span::styled("Esc", Style::default().fg(Color::Green)),
                    Span::raw(" para cancelar"),
                ]),
            ]
        }
        _ => vec![],
    };

    let edit_message = Paragraph::new(content)
        .block(Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: true });

    let clear = Clear;
    frame.render_widget(clear, area);
    frame.render_widget(edit_message, area);
}

fn draw_help_popup(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 60, frame.size());
    
    let help_text = vec![
        Line::from(vec![
            Span::styled("Comandos Disponibles", Style::default().fg(Color::Yellow))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::raw(" - Muestra esta ayuda"),
        ]),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::Green)),
            Span::raw(" - Agrega una nueva sección/detalle"),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::raw(" - Edita la sección/detalle seleccionado"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Green)),
            Span::raw(" - Elimina la sección/detalle seleccionado"),
        ]),
        Line::from(vec![
            Span::styled("s", Style::default().fg(Color::Green)),
            Span::raw(" - Buscar en secciones y detalles"),
        ]),
        Line::from(vec![
            Span::styled("x", Style::default().fg(Color::Green)),
            Span::raw(" - Exportar datos (JSON/HTML/CSV)"),
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Green)),
            Span::raw(" - Cambia el foco entre paneles"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl + ←/→", Style::default().fg(Color::Green)),
            Span::raw(" - Ajusta el tamaño de los paneles"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::raw(" - Salir"),
        ]),
    ];

    let help_message = Paragraph::new(help_text)
        .block(Block::default()
            .title("Ayuda")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .wrap(Wrap { trim: true });

    let clear = Clear;
    frame.render_widget(clear, area);
    frame.render_widget(help_message, area);
}

// Función helper para centrar el popup
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_search_popup(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 90, frame.size());
    
    // Limpiar el área antes de dibujar
    let clear = Clear;
    frame.render_widget(clear, area);

    // Obtener el nombre de la sección actual
    let section_name = app.selected_section
        .and_then(|idx| app.sections.get(idx))
        .map_or("ninguna", |section| section.title.as_str());

    let mut content = vec![
        Line::from(vec![
            Span::raw("Buscar en "),
            Span::styled(
                match app.search_target {
                    SearchTarget::Local => "📝 Local",
                    SearchTarget::CratesIo => "📦 Crates.io",
                    SearchTarget::CheatsRs => "📚 Cheats.sh",
                    SearchTarget::All => "🔍 Todas las fuentes",
                },
                Style::default().fg(Color::Yellow)
            ),
            Span::raw(" ("),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" para cambiar)"),
        ]),
        Line::from(vec![
            Span::raw("Término: "),
            Span::styled(
                format!("{}_", app.search_query),
                Style::default()
                    .fg(Color::Yellow)
                    .bg(if app.searching { Color::DarkGray } else { Color::Reset })
            ),
            if app.searching {
                Span::styled(" 🔄 Buscando...", Style::default().fg(Color::Blue))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
    ];

    if !app.search_results.is_empty() {
        content.push(Line::from(vec![
            Span::styled(
                format!("🔍 {} resultado(s):", app.search_results.len()),
                Style::default().fg(Color::Green)
            )
        ]));
        content.push(Line::from(""));

        app.links.clear();

        for (index, result) in app.search_results.iter().enumerate() {
            let source_icon = match result.source {
                SearchSource::Local => "📝",
                SearchSource::CratesIo => "📦",
                SearchSource::CheatsRs => "📚",
            };

            let is_focused = app.search_scroll == index;
            let focus_indicator = if is_focused { "➤ " } else { "  " };
            
            let title_style = if is_focused {
                Style::default().fg(Color::Black).bg(Color::Blue)
            } else {
                Style::default().fg(Color::Cyan)
            };

            if is_focused {
                app.selected_link = match result.source {
                    SearchSource::CratesIo | SearchSource::CheatsRs => Some(index),
                    _ => None,
                };
            }

            content.push(Line::from(vec![
                Span::raw(focus_indicator),
                Span::styled(source_icon, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(&result.title, title_style),
            ]));

            content.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(&result.description, Style::default().fg(Color::White)),
            ]));

            let url = match result.source {
                SearchSource::CratesIo => {
                    let url = format!("https://crates.io/crates/{}", result.title);
                    app.links.push(url.clone());
                    format!("   🌐 {}", url)
                }
                SearchSource::CheatsRs => {
                    let url = format!("https://cheat.sh/rust/{}", result.title);
                    app.links.push(url.clone());
                    format!("   🌐 {}", url)
                }
                SearchSource::Local => String::new(),
            };

            if !url.is_empty() {
                let link_style = if Some(app.links.len() - 1) == app.selected_link {
                    Style::default().fg(Color::Blue).bg(Color::White)
                } else {
                    Style::default().fg(Color::Blue)
                };

                content.push(Line::from(vec![
                    Span::styled(url, link_style)
                ]));
            }

            content.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!("[{}]", result.source),
                    Style::default().fg(Color::DarkGray)
                ),
            ]));
            content.push(Line::from("")); // Separador
        }

        if app.copying {
            content.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Yellow)),
                Span::raw("Usa "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" para seleccionar y "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" para guardar"),
            ]));
        } else if !app.links.is_empty() {
            content.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Yellow)),
                Span::raw("Usa "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" para navegar | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" para abrir | "),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw(" para copiar"),
            ]));
        } else {
            content.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Yellow)),
                Span::raw("Usa "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" para navegar y "),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw(" para guardar el resultado seleccionado"),
            ]));
        }
    } else if !app.search_query.is_empty() && !app.searching {
        content.push(Line::from(vec![
            Span::styled("❌ No se encontraron resultados", Style::default().fg(Color::Red))
        ]));
    }

    // Agregar ayuda en un recuadro separado
    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let help_content = vec![
        Line::from(vec![
            Span::styled("Comandos:", Style::default().fg(Color::DarkGray))
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Cambiar fuente"),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Navegar"),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("PgUp/PgDn", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Scroll"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Abrir enlace"),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Copiar/Guardar"),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::DarkGray)),
            Span::raw(" - Cerrar"),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_content)
        .block(help_block)
        .style(Style::default().fg(Color::DarkGray));

    let content_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(5),  // Altura para el área de ayuda
        ])
        .split(area);

    let main_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(content_area[0]);

    // Calcular el scroll para mantener el ítem seleccionado visible
    let viewport_height = main_area[0].height.saturating_sub(4);
    let total_items = content.len() as u16;

    // Cada resultado ocupa aproximadamente 5 líneas (título, descripción, URL, fuente, separador)
    let lines_per_result = 5;
    let real_selected_position = (app.search_scroll * lines_per_result) as u16;

    // Ajustar el scroll para mantener el ítem seleccionado visible
    let scroll_offset = if real_selected_position > viewport_height {
        real_selected_position - (viewport_height / 2)
    } else {
        0
    };

    // Asegurarse de que no excedamos el máximo scroll posible
    let max_scroll = (content.len() as u16).saturating_sub(viewport_height);
    let scroll_offset = scroll_offset.min(max_scroll);

    let paragraph = Paragraph::new(content.clone())
        .block(Block::default()
            .title(format!("Búsqueda [Sección: {}]", section_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .scroll((scroll_offset, 0))
        .wrap(Wrap { trim: true });

    // Actualizar el estado del scrollbar
    let mut scroll_state = ScrollbarState::new((content.len() / lines_per_result as usize) + 1)
        .position(app.search_scroll);

    frame.render_widget(paragraph, main_area[0]);
    frame.render_widget(help_paragraph, content_area[1]);

    // Renderizar scrollbar solo si es necesario
    if total_items > viewport_height {
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            main_area[1],
            &mut scroll_state,
        );
    }
}

pub fn draw_export_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.size());
    
    let clear = Clear;
    frame.render_widget(clear, area);

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut content = vec![
        Line::from(vec![
            Span::styled("Exportar datos", Style::default().fg(Color::Yellow))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Directorio: "),
            Span::styled(&home, Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from("Selecciona el formato:"),
        Line::from(""),
    ];

    for (i, format) in app.export_formats.iter().enumerate() {
        let prefix = if i == app.selected_export_format { "➤ " } else { "  " };
        let filename = match format {
            ExportFormat::JSON => "rust-tui-export.json",
            ExportFormat::HTML => "rust-tui-export.html",
            ExportFormat::CSV => "rust-tui-export.csv",
        };
        
        content.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(
                format.to_string(),
                if i == app.selected_export_format {
                    Style::default().fg(Color::Black).bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                }
            ),
            Span::raw(" → "),
            Span::styled(filename, Style::default().fg(Color::Blue)),
        ]));
    }

    content.push(Line::from(""));
    
    // Mostrar mensaje de éxito/error si existe
    if let Some(message) = &app.export_message {
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled(message, Style::default().fg(Color::Green))
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" para seleccionar, "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" para exportar, "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" para cancelar"),
    ]));

    let export_message = Paragraph::new(content)
        .block(Block::default()
            .title("Exportar")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(export_message, area);
}
