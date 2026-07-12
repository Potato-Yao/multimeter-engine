#[derive(Debug)]
pub enum MigrationCommand {
    Init,
    Backup,
    Restore,
}

#[derive(Debug)]
pub enum CliCommand {
    Migration(MigrationCommand),
}

pub fn parse_cli_command() -> Option<CliCommand> {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);

    if iter.next()?.as_str() != "--cli" {
        return None;
    }

    match iter.next()?.as_str() {
        "migration" => match iter.next()?.as_str() {
            "init" => Some(CliCommand::Migration(MigrationCommand::Init)),
            "backup" => Some(CliCommand::Migration(MigrationCommand::Backup)),
            "restore" => Some(CliCommand::Migration(MigrationCommand::Restore)),
            _ => None,
        },
        _ => None,
    }
}
