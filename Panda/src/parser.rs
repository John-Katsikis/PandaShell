#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineAst {
    pub commands: Vec<CommandAst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAst {
    pub name: String,
    pub args: Vec<String>,
    pub raw: String,
}

impl CommandAst {
    pub fn as_input(&self) -> &str {
        &self.raw
    }

    pub fn arg_text(&self) -> &str {
        self.raw.strip_prefix(&self.name).unwrap_or("").trim_start()
    }
}

pub fn parse_line(input: &str) -> Result<PipelineAst, String> {
    let mut commands = Vec::new();

    for segment in split_pipeline(input)? {
        let raw = segment.trim().to_string();
        if raw.is_empty() {
            continue;
        }

        let tokens = tokenize(&raw)?;
        let Some((name, args)) = tokens.split_first() else {
            continue;
        };

        commands.push(CommandAst {
            name: name.clone(),
            args: args.to_vec(),
            raw,
        });
    }

    Ok(PipelineAst { commands })
}

fn split_pipeline(input: &str) -> Result<Vec<String>, String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            current.push(c);
            continue;
        }

        match quote {
            Some(q) if c == q => {
                quote = None;
                current.push(c);
            }
            Some(_) => current.push(c),
            None if c == '\'' || c == '"' => {
                quote = Some(c);
                current.push(c);
            }
            None if c == '|' => {
                segments.push(current);
                current = String::new();
            }
            None => current.push(c),
        }
    }

    if let Some(q) = quote {
        return Err(format!("Unclosed quote '{}'", q));
    }

    if escaped {
        current.push('\\');
    }

    segments.push(current);
    Ok(segments)
}

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => current.push(c),
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
            }
            None => current.push(c),
        }
    }

    if let Some(q) = quote {
        return Err(format!("Unclosed quote '{}'", q));
    }

    if escaped {
        current.push('\\');
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    #[test]
    fn parses_literal_command_arguments() {
        let ast = parse_line("spark hash Yianni").unwrap();

        assert_eq!(ast.commands.len(), 1);
        assert_eq!(ast.commands[0].name, "spark");
        assert_eq!(ast.commands[0].args, ["hash", "Yianni"]);
        assert_eq!(ast.commands[0].arg_text(), "hash Yianni");
    }

    #[test]
    fn splits_pipes_outside_quotes() {
        let ast = parse_line("echo \"a | b\" | hash Yianni").unwrap();

        assert_eq!(ast.commands.len(), 2);
        assert_eq!(ast.commands[0].args, ["a | b"]);
        assert_eq!(ast.commands[1].name, "hash");
    }

    #[test]
    fn rejects_unclosed_quotes() {
        assert!(parse_line("spark \"oops").is_err());
    }
}
