use inquire::Text;
use miette::{IntoDiagnostic, Result};
use serde_json::{json, Map, Value};
use std::io::stdout;
use termion::{color, screen::IntoAlternateScreen};

use crate::Meilisearch;

impl Meilisearch {
    pub fn run_interactive_search(
        &self,
        base_search: String,
        base_search_config: Map<String, Value>,
    ) -> Result<()> {
        let _screen = stdout().into_alternate_screen().into_diagnostic()?;
        let available_lines = termion::terminal_size().expect("Unsupported terminal").1;

        Text::new("Search:")
            .with_suggester(&move |input| {
                self.search_suggestor(&base_search_config, available_lines as usize, input)
            })
            .with_placeholder(&base_search)
            .with_initial_value(&base_search)
            .prompt()
            .into_diagnostic()?;

        Ok(())
    }

    /// This could be faster by using smarter ways to check for matches, when dealing with larger datasets.
    fn search_suggestor(
        &self,
        search_config: &Map<String, Value>,
        available_lines: usize,
        input: &str,
    ) -> Vec<String> {
        let mut search = search_config.clone();
        if search.get("attributesToHighlight").is_none() {
            search.insert("attributesToHighlight".to_string(), json!(["*"]));
        }
        search.insert("q".to_string(), json!(input));

        let response = self
            .post(format!("{}/indexes/{}/search", self.addr, self.index))
            .header("Content-Type", "application/json")
            .json(&search)
            .send();
        let response = match response {
            Err(e) => {
                return vec![
                    "Cannot connect to Meilisearch:".to_string(),
                    format!(
                        "\t{}{}{}",
                        color::Fg(color::Red),
                        e,
                        color::Fg(color::Reset)
                    ),
                ]
            }
            Ok(response) => response,
        };
        if response.status().is_success() {
            response.json::<Value>().unwrap()["hits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.get("_formatted").unwrap())
                .map(|value| colored_json::to_colored_json_auto(value).unwrap())
                .map(|s| s.replace("<em>", &color::Fg(color::Red).to_string()))
                .map(|s| s.replace("</em>", &color::Fg(color::Green).to_string()))
                .scan(0, |line, value| {
                    *line += value.lines().count() + 1;
                    if *line > available_lines {
                        None
                    } else {
                        Some(value)
                    }
                })
                .fuse()
                .collect()
        } else {
            vec![colored_json::to_colored_json_auto(&response.json::<Value>().unwrap()).unwrap()]
        }
    }
}
