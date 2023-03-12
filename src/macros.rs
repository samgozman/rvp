#[macro_export]
/// Generates `Commands` enum based on the modules in the /commands directory.
///
/// The macro takes a list of modules as arguments, and generates an enum with
/// arguments for each module. The enum is then used to match the subcommand
/// passed to the CLI.
macro_rules! commands_builder {
    ($($module:ident),*) => (
      // `paste!` is used to generate non-existent identifiers for enum, in this case the enum elements
      // which should represent the modules in the /commands directory.
      paste::paste! {
        #[derive(Subcommand)]
        enum Commands {
            $(
              [<$module:camel>]($module::Args),
            )*
        }

        impl Commands {
            async fn exec(cli: Cli) -> Result<()> {
              // Match the subcommand passed to the CLI.
              // This little magic saves us from having to write a clap-match statement for each module.
              // Rust if fucking awesome! 🦀
              match cli.command {
                $(
                  // In the `::command(args)` section you can pass global options after the args (from `cli`)
                  Commands::[<$module:camel>](args) => $module::command(args).await?,
                )*
              }
              Ok(())
            }
        }
      }
    );
}
