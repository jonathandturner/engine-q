use nu_ansi_term::Style;
use nu_parser::{flatten_block, parse, FlatShape};
use nu_protocol::engine::{EngineState, StateWorkingSet};
use reedline::{Highlighter, StyledText};

pub struct NuHighlighter {
    pub engine_state: EngineState,
}

impl Highlighter for NuHighlighter {
    fn highlight(&self, line: &str) -> StyledText {
        let (shapes, global_span_offset) = {
            let mut working_set = StateWorkingSet::new(&self.engine_state);
            let (block, _) = parse(&mut working_set, None, line.as_bytes(), false);

            let shapes = flatten_block(&working_set, &block);
            (shapes, self.engine_state.next_span_start())
        };

        let mut output = StyledText::default();
        let mut last_seen_span = global_span_offset;

        for shape in &shapes {
            if shape.0.end <= last_seen_span {
                // We've already output something for this span
                // so just skip this one
                continue;
            }
            if shape.0.start > last_seen_span {
                let gap = line
                    [(last_seen_span - global_span_offset)..(shape.0.start - global_span_offset)]
                    .to_string();
                output.push((Style::new(), gap));
            }
            let next_token = line
                [(shape.0.start - global_span_offset)..(shape.0.end - global_span_offset)]
                .to_string();
            match shape.1 {
                FlatShape::Custom(..) => output.push((Style::new().bold(), next_token)),
                FlatShape::External => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Green), next_token))
                }
                FlatShape::ExternalArg => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Green), next_token))
                }
                FlatShape::Garbage => output.push((
                    Style::new()
                        .fg(nu_ansi_term::Color::White)
                        .on(nu_ansi_term::Color::Red)
                        .bold(),
                    next_token,
                )),
                FlatShape::InternalCall => output.push((
                    Style::new().fg(nu_ansi_term::Color::LightBlue).bold(),
                    next_token,
                )),
                FlatShape::Int => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Green), next_token))
                }
                FlatShape::Float => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Green), next_token))
                }
                FlatShape::Range => output.push((
                    Style::new().fg(nu_ansi_term::Color::LightPurple),
                    next_token,
                )),
                FlatShape::Bool => {
                    output.push((Style::new().fg(nu_ansi_term::Color::LightCyan), next_token))
                }
                FlatShape::Literal => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Blue), next_token))
                }
                FlatShape::Operator => output.push((
                    Style::new().fg(nu_ansi_term::Color::LightPurple).bold(),
                    next_token,
                )),
                FlatShape::Signature => output.push((
                    Style::new().fg(nu_ansi_term::Color::Green).bold(),
                    next_token,
                )),
                FlatShape::String => output.push((
                    Style::new().fg(nu_ansi_term::Color::Yellow).bold(),
                    next_token,
                )),
                FlatShape::Flag => {
                    output.push((Style::new().fg(nu_ansi_term::Color::Yellow), next_token))
                }
                FlatShape::Filepath => output.push((
                    Style::new().fg(nu_ansi_term::Color::Yellow).bold(),
                    next_token,
                )),
                FlatShape::GlobPattern => output.push((
                    Style::new().fg(nu_ansi_term::Color::Yellow).bold(),
                    next_token,
                )),
                FlatShape::Variable => output.push((
                    Style::new().fg(nu_ansi_term::Color::Blue).bold(),
                    next_token,
                )),
            }
            last_seen_span = shape.0.end;
        }

        let remainder = line[(last_seen_span - global_span_offset)..].to_string();
        if !remainder.is_empty() {
            output.push((Style::new(), remainder));
        }

        output
    }
}
