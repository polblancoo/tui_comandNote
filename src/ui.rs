use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::app::{App, Focus, Mode, PopupFocus, SearchTarget};

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Limpiar toda la pantalla primero
    frame.render_widget(Clear, frame.size());

    // Crear layout principal
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

    // Limpiar cada área individualmente antes de dibujar
    frame.render_widget(Clear, main_chunks[0]);
    frame.render_widget(Clear, main_chunks[1]);
    frame.render_widget(Clear, chunks[1]);

    // Dibujar los paneles principales
    draw_sections(frame, app, main_chunks[0]);
    draw_details(frame, app, main_chunks[1]);
    draw_shortcuts(frame, app, chunks[1]);

    // Si hay un popup, limpiar su área y dibujarlo
    if app.mode != Mode::Normal {
        let popup_area = centered_rect(80, 80, frame.size());
        frame.render_widget(Clear, popup_area);
        
        match app.mode {
            Mode::Adding | Mode::Editing => draw_edit_popup(frame, app),
            Mode::Help => draw_help_popup(frame, app),
            Mode::Searching => draw_search_popup(frame, app),
            Mode::Exporting => draw_export_popup(frame, app),
            Mode::Viewing => draw_view_popup(frame, app),
            _ => {}
        }
    }
}

fn draw_sections(frame: &mut Frame, app: &App, area: Rect) {
    let mut state = ListState::default();
    state.select(app.selected_section);

    let block = Block::default()
        .title(Span::styled(
            "Secciones",
            Style::default().fg(if app.focus == Focus::Sections {
                Color::Yellow
            } else {
                Color::White
            })
        ))
        .borders(Borders::ALL);

    let items: Vec<ListItem> = app.sections.iter()
        .map(|section| {
            let title = section.title
                .trim_start_matches('📁')
                .trim_start_matches('📝')
                .trim();
            
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<30} 📁", title), // Padding fijo para alinear iconos
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

    // Obtener el título de la sección actual
    let section_title = app.selected_section
        .and_then(|idx| app.sections.get(idx))
        .map(|s| &s.title)
        .map_or("Sin sección".to_string(), |t| t.clone());

    let block = Block::default()
        .title(Span::styled(
            format!("Detalles de {}", section_title),
            Style::default().fg(if app.focus == Focus::Details {
                Color::Yellow
            } else {
                Color::White
            })
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == Focus::Details {
            Color::Yellow
        } else {
            Color::White
        }));

    // Calcular el número total de líneas para todos los detalles
    let total_lines = if let Some(section_idx) = app.selected_section {
        if let Some(section) = app.sections.get(section_idx) {
            section.details.iter().map(|detail| {
                // Cada detalle ocupa: título + descripción + fecha + línea vacía
                let desc_lines = detail.description.lines().count();
                2 + desc_lines + 2
            }).sum()
        } else {
            0
        }
    } else {
        0
    };

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(total_lines)
        .position(app.details_scroll);

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

    let list_area = Rect {
        width: area.width.saturating_sub(1),
        ..area
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_stateful_widget(list, list_area, &mut state);

    // Renderizar scrollbar
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
    let shortcuts = match app.mode {
        Mode::Normal => vec![
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw(" agregar | "),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw(" editar | "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(" eliminar | "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" cambiar panel | "),
                Span::styled("h", Style::default().fg(Color::Yellow)),
                Span::raw(" ayuda | "),
                Span::styled("Ctrl+q", Style::default().fg(Color::Yellow)),
                Span::raw(" salir"),
            ]),
        ],
        Mode::Adding | Mode::Editing => {
            if app.focus == Focus::Details {
                vec![
                    Line::from(vec![
                        Span::styled("Tab", Style::default().fg(Color::Yellow)),
                        Span::raw(" cambiar campo | "),
                        Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
                        Span::raw(" guardar | "),
                        Span::styled("Ctrl+L", Style::default().fg(Color::Yellow)),
                        Span::raw(" cambiar lenguaje"),
                    ]),
                    Line::from(vec![
                        Span::styled("Ctrl+C/V", Style::default().fg(Color::Yellow)),
                        Span::raw(" copiar/pegar | "),
                        Span::styled("Enter", Style::default().fg(Color::Yellow)),
                        Span::raw(" nueva línea | "),
                        Span::styled("Esc", Style::default().fg(Color::Yellow)),
                        Span::raw(" cancelar"),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
                        Span::raw(" guardar | "),
                        Span::styled("Esc", Style::default().fg(Color::Yellow)),
                        Span::raw(" cancelar"),
                    ]),
                ]
            }
        },
        _ => vec![],
    };

    let shortcuts_block = Paragraph::new(shortcuts)
        .block(Block::default()
            .title("Atajos")
            .borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(shortcuts_block, area);
}

fn draw_edit_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, frame.size());
    frame.render_widget(Clear, area);
    
    if app.focus == Focus::Details {
        // Marco contenedor
        let container_block = Block::default()
            .title(Span::styled(
                if app.mode == Mode::Adding { "Agregar detalle" } else { "Editar detalle" },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        frame.render_widget(container_block.clone(), area);
        let inner_area = container_block.inner(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),   // Título
                Constraint::Length(8),   // Descripción
                Constraint::Min(10),     // Código
                Constraint::Length(3),   // Ayuda
            ])
            .split(inner_area);

        // Renderizar título con cursor
        let title_content = {
            let mut content = app.input_buffer.clone();
            if app.popup_focus == PopupFocus::Title {
                content.insert(app.title_cursor, '█');
            }
            content
        };

        let title = Paragraph::new(title_content)
            .block(Block::default()
                .title("Título")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if app.popup_focus == PopupFocus::Title {
                        Color::Yellow
                    } else {
                        Color::White
                    }
                )));

        frame.render_widget(title, chunks[0]);

        // Renderizar descripción con cursor
        let description_content = {
            let mut content = app.description_buffer.clone();
            if app.popup_focus == PopupFocus::Description {
                content.insert(app.description_cursor, '█');
            }
            content
        };

        let description = Paragraph::new(description_content)
            .block(Block::default()
                .title("Descripción")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if app.popup_focus == PopupFocus::Description {
                        Color::Yellow
                    } else {
                        Color::White
                    }
                )))
            .wrap(Wrap { trim: true });

        frame.render_widget(description, chunks[1]);

        // Renderizar código con cursor
        let code_content = {
            let mut content = app.code_buffer.clone();
            if app.popup_focus == PopupFocus::Code {
                content.insert(app.code_cursor, '█');
            }
            content
        };

        let code = Paragraph::new(code_content)
            .block(Block::default()
                .title(format!("Código - {}", app.selected_language))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if app.popup_focus == PopupFocus::Code {
                        Color::Yellow
                    } else {
                        Color::White
                    }
                )));

        frame.render_widget(code, chunks[2]);

        // Renderizar ayuda
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Tab", Style::default().fg(Color::Yellow)),
                    Span::raw(" cambiar campo | "),
                    Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
                    Span::raw(" guardar | "),
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" cancelar"),
                ])
            ])
            .block(Block::default()
                .title("Atajos")
                .borders(Borders::ALL))
            .alignment(Alignment::Left),
            chunks[3]
        );
    } else {
        // Frame para secciones
        let container_block = Block::default()
            .title(Span::styled(
                if app.mode == Mode::Adding { "Nueva Sección" } else { "Editar Sección" },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        frame.render_widget(container_block.clone(), area);
        let inner_area = container_block.inner(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Input
                Constraint::Length(2),  // Espacio
                Constraint::Length(3),  // Ayuda
            ])
            .split(inner_area);

        // Input con cursor
        let title_content = {
            let mut content = app.input_buffer.clone();
            content.insert(app.title_cursor, '█');
            content
        };

        let input = Paragraph::new(title_content)
            .block(Block::default()
                .title("Título")
                .borders(Borders::ALL));

        // Ayuda
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
                Span::raw(" guardar | "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cancelar"),
            ])
        ])
        .block(Block::default()
            .title("Atajos")
            .borders(Borders::ALL))
        .alignment(Alignment::Left);

        frame.render_widget(input, chunks[0]);
        frame.render_widget(help, chunks[2]);
    }
}

fn draw_help_popup(frame: &mut Frame, app: &App) {
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
            Span::styled("Ctrl+q", Style::default().fg(Color::Green)),
            Span::raw(" - Salir"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Green)),
            Span::raw(" - Cerrar ayuda"),
        ]),
    ];

    let help_message = Paragraph::new(help_text)
        .block(Block::default()
            .title("Ayuda")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
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

pub fn draw_search_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 90, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            "👁 Búsqueda",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(block.clone(), area);
    let inner_area = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Área de búsqueda
            Constraint::Min(1),     // Resultados
            Constraint::Length(5),  // Ayuda
        ])
        .split(inner_area);

    // Área de búsqueda
    let search_input = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Buscar en "),
            Span::styled(
                format!("{}", app.search_target),
                Style::default().fg(Color::Yellow)
            ),
            Span::raw(": "),
            Span::styled(
                format!("{}_", app.search_query),
                Style::default().fg(Color::White)
            ),
        ])
    ])
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(search_input, chunks[0]);

    // Resultados de búsqueda
    let results: Vec<Line> = app.search_results.iter().enumerate().map(|(i, result)| {
        let style = if i == app.search_scroll {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        Line::from(vec![
            Span::styled(
                if i == app.search_scroll { "▶ " } else { "  " },
                style
            ),
            Span::styled(&result.title, style.fg(Color::Green)),
            Span::raw(" - "),
            Span::styled(&result.description, style),
        ])
    }).collect();

    let results_widget = Paragraph::new(results)
        .block(Block::default()
            .title("Resultados")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(results_widget, chunks[1]);

    // Ayuda
    let help_text = match app.search_target {
        SearchTarget::Local => vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" cambiar fuente | "),
                Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                Span::raw(" navegar | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" ver detalle"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cerrar"),
            ]),
        ],
        _ => vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" cambiar fuente | "),
                Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                Span::raw(" navegar | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" abrir enlace"),
            ]),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" cerrar"),
            ]),
        ],
    };

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title("Atajos")
            .borders(Borders::ALL))
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

pub fn draw_export_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            "📤 Exportar Datos",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(3),    // Contenido
            Constraint::Length(3), // Ayuda
        ])
        .split(inner_area);

    // Contenido
    let content = if let Some(message) = &app.export_message {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    message,
                    if message.starts_with('✅') {
                        Style::default().fg(Color::Green)
                    } else if message.starts_with('❌') {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                )
            ])
        ]
    } else {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "Selecciona el formato de exportación:",
                    Style::default().add_modifier(Modifier::BOLD)
                )
            ]),
            Line::from(""),
        ];

        for (i, format) in app.export_formats.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::raw(if i == app.selected_export_format { "▶ " } else { "  " }),
                Span::styled(
                    format.to_string(),
                    if i == app.selected_export_format {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                )
            ]));
        }
        lines
    };

    let content_widget = Paragraph::new(content)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(content_widget, chunks[0]);

    // Ayuda
    let help_text = if app.export_message.is_some() {
        vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" o "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" para cerrar"),
        ]
    } else {
        vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" seleccionar | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" confirmar | "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" cancelar"),
        ]
    };

    let help = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[1]);
}

fn draw_view_popup(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 80, frame.size());
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Título
            Constraint::Length(5),  // Descripción
            Constraint::Min(10),    // Código
            Constraint::Length(5),  // Ayuda
        ])
        .split(area);

    // Título
    let title = Paragraph::new(app.input_buffer.clone())
        .block(Block::default()
            .title("Título")
            .borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Descripción
    let description = Paragraph::new(app.description_buffer.clone())
        .block(Block::default()
            .title("Descripción")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(description, chunks[1]);

    // Código con resaltado de selección y cursor
    let code_lines: Vec<Line> = app.code_buffer
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let mut spans = vec![
                Span::styled(
                    format!("{:4} │ ", i + 1),
                    Style::default().fg(Color::DarkGray)
                ),
            ];

            let mut line_spans = Vec::new();
            for (j, c) in line.chars().enumerate() {
                let pos = app.get_position_in_buffer(i, j);
                let mut style = Style::default();

                // Aplicar estilo de selección si corresponde
                if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
                    let (start, end) = if start <= end { (start, end) } else { (end, start) };
                    if pos >= start && pos <= end {
                        style = style.bg(Color::DarkGray);
                    }
                }

                // Mostrar cursor
                if pos == app.code_cursor {
                    style = Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::SLOW_BLINK);
                }

                line_spans.push(Span::styled(c.to_string(), style));
            }

            // Cursor al final de la línea
            if app.code_cursor == app.get_position_in_buffer(i, line.len()) {
                line_spans.push(Span::styled(
                    "█",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::SLOW_BLINK)
                ));
            }

            spans.extend(line_spans);
            Line::from(spans)
        })
        .collect();

    let code = Paragraph::new(code_lines)
        .block(Block::default()
            .title(format!("Código - {}", app.selected_language))
            .borders(Borders::ALL))
        .scroll((app.code_scroll as u16, 0));
    frame.render_widget(code, chunks[2]);

    // Ayuda mejorada
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Navegación", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": "),
            Span::styled("←/→/↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" mover cursor | "),
            Span::styled("Ctrl + ←/→/↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" seleccionar"),
        ]),
        Line::from(vec![
            Span::styled("Acciones", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": "),
            Span::styled("Ctrl+Y", Style::default().fg(Color::Yellow)),
            Span::raw(" copiar | "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" salir"),
        ])
    ])
    .block(Block::default()
        .title("Atajos")
        .borders(Borders::ALL))
    .alignment(Alignment::Left);

    frame.render_widget(help, chunks[3]);

    // Actualizar scroll horizontal según la posición del cursor
    let current_column = app.code_cursor_column();
    let visible_columns = chunks[2].width as usize - 6; // Restar el ancho del número de línea
    
    if current_column > app.code_scroll + visible_columns {
        app.code_scroll = current_column - visible_columns + 1;
    } else if current_column < app.code_scroll {
        app.code_scroll = current_column;
    }

    // Scrollbar horizontal
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::HorizontalBottom)
        .begin_symbol(Some("◄"))
        .end_symbol(Some("►"));

    let max_line_length = app.code_buffer.lines()
        .map(|l| l.len())
        .max()
        .unwrap_or(0);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(max_line_length)
        .position(app.code_scroll);

    frame.render_stateful_widget(
        scrollbar,
        chunks[2],
        &mut scrollbar_state,
    );
}
