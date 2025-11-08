use crate::Context;

pub struct CommandInfo<'a> {
    pub command: &'a poise::Command<crate::Data, crate::Error>,
    pub subcommand: Option<&'a poise::Command<crate::Data, crate::Error>>,
}

pub struct CommandFinder<'a> {
    ctx: &'a Context<'a>,
}

impl<'a> CommandFinder<'a> {
    pub fn new(ctx: &'a Context<'a>) -> Self {
        Self { ctx }
    }

    pub fn get_all_commands(
        &self,
    ) -> Vec<&'a poise::Command<crate::Data, crate::Error>> {
        self.ctx
            .framework()
            .options()
            .commands
            .iter()
            .filter(|cmd| !cmd.hide_in_help)
            .collect()
    }

    pub fn get_all_command_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        for cmd in self.ctx.framework().options().commands.iter() {
            if cmd.hide_in_help {
                continue;
            }

            if cmd.subcommands.is_empty() {
                names.push(cmd.name.to_string());
            } else {
                for subcmd in &cmd.subcommands {
                    if !subcmd.hide_in_help {
                        names.push(format!("{} {}", cmd.name, subcmd.name));
                    }
                }
            }
        }

        names
    }

    pub fn find_command(&self, search_name: &str) -> Option<CommandInfo<'a>> {
        let needle = search_name.trim().to_ascii_lowercase();

        for cmd in self.ctx.framework().options().commands.iter() {
            if cmd.hide_in_help {
                continue;
            }

            let cmd_matches = cmd.name.to_ascii_lowercase() == needle
                || cmd.aliases.iter().any(|alias| alias.to_ascii_lowercase() == needle);

            if cmd_matches {
                return Some(CommandInfo {
                    command: cmd,
                    subcommand: None,
                });
            }

            if !cmd.subcommands.is_empty() {
                for subcmd in &cmd.subcommands {
                    if subcmd.hide_in_help {
                        continue;
                    }

                    let full_name = format!("{} {}", cmd.name, subcmd.name);
                    let full_name_lower = full_name.to_ascii_lowercase();

                    if full_name_lower == needle {
                        return Some(CommandInfo {
                            command: cmd,
                            subcommand: Some(subcmd),
                        });
                    }

                    let subcmd_matches = subcmd.name.to_ascii_lowercase() == needle
                        || subcmd.aliases.iter().any(|alias| alias.to_ascii_lowercase() == needle);

                    if subcmd_matches {
                        return Some(CommandInfo {
                            command: cmd,
                            subcommand: Some(subcmd),
                        });
                    }
                }
            }
        }

        None
    }
}
