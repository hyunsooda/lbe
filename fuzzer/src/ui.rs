use crate::{
    campaign::{FuzzShot, FuzzingCampaign},
    cli::FuzzInput,
    ver::VERSION,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Color,
    style::{Modifier, Style, Styled, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    sync::mpsc,
    time::Duration,
};

pub fn run_ui(rx: mpsc::Receiver<FuzzShot>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, rx)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("{}", res);
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    rx: mpsc::Receiver<FuzzShot>,
) -> io::Result<String> {
    let mut fuzz_result = FuzzingCampaign::new(FuzzInput::Stdin);
    loop {
        match rx.recv() {
            Ok(result) => match result {
                FuzzShot::ProgramOutput(o) => {
                    fuzz_result.add_program_output(o);
                }
                FuzzShot::Coverage(cov) => {
                    fuzz_result.set_coverage(cov);
                }
                FuzzShot::Crash(crash) => {
                    fuzz_result.set_crash(crash);
                }
                FuzzShot::Metadata(md) => {
                    fuzz_result.set_metadata(md);
                }
                FuzzShot::SeedInfo(seed) => {
                    fuzz_result.set_seed_info(seed);
                }
                FuzzShot::Terminated => {
                    return Ok("Fuzzer terminated".to_string());
                }
            },
            Err(err) => {
                panic!("{:?}", err);
            }
        }

        terminal.draw(|frame| render_ui(frame, &mut fuzz_result))?;

        let timeout = Duration::from_millis(10);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    return Ok("'Q' entered".to_string());
                }
            }
        }
    }
}

fn render_ui(frame: &mut Frame, fuzz_result: &mut FuzzingCampaign) {
    let area = frame.area();
    let allocated_title_height = 3;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(allocated_title_height),
            Constraint::Min(0),
        ])
        .split(area);

    let title_paragraph =
        Paragraph::new(Line::from(format!(" Fuzzer v{} ", VERSION).yellow().bold()))
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::vertical(
                (main_chunks[0].height.saturating_sub(1)) / 2,
            )));
    frame.render_widget(title_paragraph, main_chunks[0]);

    let content_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Min(0),
        ])
        .split(main_chunks[1]);

    let ab_compartments = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_rows[0]);

    let compartment_a = Block::default()
        .title(Line::from(" A: Metadata ".green().bold()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(&compartment_a, ab_compartments[0]);

    {
        let a_data = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::raw("                 Fuzzing loop: "),
                Span::styled(
                    format!("{}", fuzz_result.metadata.fuzz_cnt),
                    Style::default().yellow(),
                ),
            ]),
            Line::from(vec![
                Span::raw("           Fuzzing input type: "),
                Span::styled(
                    format!("{:?}", fuzz_result.metadata.fuzz_input_typ),
                    Style::default().light_yellow(),
                ),
            ]),
            Line::from(vec![
                Span::raw("                      Timeout: "),
                Span::styled(
                    format!("{:?}", fuzz_result.metadata.timeout),
                    Style::default().cyan(),
                ),
            ]),
            Line::from(vec![
                Span::raw("   Elapsed time of single run: "),
                Span::styled(
                    format!("{:?}", fuzz_result.metadata.target_elpased_time),
                    Style::default().green(),
                ),
            ]),
            Line::from(vec![
                Span::raw("                       Uptime: "),
                Span::styled(
                    format!("{:?}", fuzz_result.metadata.total_elpased_time),
                    Style::default().light_blue(),
                ),
            ]),
        ]));
        frame.render_widget(a_data, compartment_a.inner(ab_compartments[0]));
    }

    let compartment_b = Block::default()
        .title(Line::from(
            " B: Crash ".red().bold().add_modifier(Modifier::REVERSED),
        ))
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Rgb(30, 0, 0)));
    frame.render_widget(&compartment_b, ab_compartments[1]);
    {
        let b_data = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                " !!! Critical Attention Required !!!",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::RAPID_BLINK),
            )),
            Line::from(vec![
                Span::raw("          Crashes: "),
                Span::styled(
                    format!("{}", fuzz_result.crash.crashes),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("           Origin: "),
                Span::styled(
                    fuzz_result.crash.origin.to_hex(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    Origin length: "),
                Span::styled(
                    format!("{}", fuzz_result.crash.origin.get_input().len()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("        Minimized: "),
                Span::styled(
                    fuzz_result.crash.minimized.to_hex(),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED),
                ),
            ]),
            Line::from(vec![
                Span::raw(" Minimized length: "),
                Span::styled(
                    format!("{}", fuzz_result.crash.minimized.get_input().len()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]));
        frame.render_widget(&b_data, compartment_b.inner(ab_compartments[1]));
    }

    let cd_compartments = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_rows[1]);

    let compartment_c = Block::default()
        .title(Line::from(
            " C: Coverage ".set_style(
                Style::default()
                    .fg(Color::Rgb(255, 165, 0))
                    .add_modifier(Modifier::BOLD),
            ),
        ))
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().fg(Color::Rgb(255, 165, 0)));
    frame.render_widget(&compartment_c, cd_compartments[0]);
    {
        let single_file_cov_report = &fuzz_result.coverage[0];
        let c_data = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::raw("               File: "),
                Span::styled(
                    &single_file_cov_report.file,
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw(" Function hit ratio: "),
                Span::styled(
                    format!("{}", single_file_cov_report.funcs_hit_ratio),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("   Branch hit ratio: "),
                Span::styled(
                    format!("{}", single_file_cov_report.brs_hit_ratio),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("     Line hit ratio: "),
                Span::styled(
                    format!("{}", single_file_cov_report.lines_hit_ratio),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ]));
        frame.render_widget(c_data, compartment_c.inner(cd_compartments[0]));
    }

    let compartment_d = Block::default()
        .title(Line::from(" D: Seed ".blue().bold()))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(Color::Rgb(20, 20, 40)).fg(Color::Gray));
    frame.render_widget(&compartment_d, cd_compartments[1]);
    {
        let d_data = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::raw("              seeds: "),
                Span::styled(
                    format!("{}", fuzz_result.seed_info.seeds),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("       Current seed: "),
                Span::styled(
                    fuzz_result.seed_info.cur_seed.to_hex(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                Span::raw(" Current seed score: "),
                Span::styled(
                    format!("{}", fuzz_result.seed_info.cur_seed.get_score()),
                    Style::default().fg(Color::Rgb(0, 130, 200)),
                ),
            ]),
            Line::from(vec![
                Span::raw("      Visited edges: "),
                Span::styled(
                    format!("{}", fuzz_result.seed_info.visit_edges),
                    Style::default().fg(Color::Rgb(0, 130, 200)),
                ),
            ]),
            Line::from(vec![
                Span::raw("          Next seed: "),
                Span::styled(
                    fuzz_result.seed_info.next_seed.to_hex(),
                    Style::default().fg(Color::LightMagenta),
                ),
            ]),
            Line::from(vec![
                Span::raw("          New paths: "),
                Span::styled(
                    format!("{}", fuzz_result.seed_info.new_paths),
                    Style::default().fg(Color::LightGreen),
                ),
            ]),
        ]));
        frame.render_widget(d_data, compartment_d.inner(cd_compartments[1]));
    }

    let compartment_e = Block::default()
        .title(" E: Stdout ".light_green().bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ratatui::style::Color::DarkGray));
    frame.render_widget(&compartment_e, content_rows[2]);

    let lines: Vec<_> = fuzz_result
        .program_output
        .iter()
        .map(|line| Line::from(Span::styled(line, Style::default().gray())))
        .collect();
    let lines_len = lines.len();
    let e_data = Paragraph::new(lines).scroll((
        lines_len
            .saturating_sub(content_rows[2].height.try_into().unwrap())
            .try_into()
            .unwrap(),
        0,
    ));
    frame.render_widget(e_data, compartment_e.inner(content_rows[2]));

    if fuzz_result.program_output.len() > content_rows[2].height.into() {
        fuzz_result
            .program_output
            .drain(0..content_rows[2].height.into());
    }
}
