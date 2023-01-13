use clap::Command;

pub fn cli() -> Command {
    Command::new("sat")
        .about("Super Auto Test - A Super Auto Test database and testing framework.")
        .subcommand_required(true)
        .subcommand(Command::new("init").about("Initializes SAP database. Used to query."))
        .subcommand(Command::new("run").about("Run server. Updates database prior to starting up."))
}
