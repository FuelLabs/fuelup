// #[derive(Debug, Parser)]
// pub struct NixShellCommand {
//     /// Open a new bash development shell with specified toolchain or component.
//     pub name: String,
// }

// pub fn nix_shell(command: NixShellCommand) -> Result<()> {
//     info!(
//         "starting new bash shell with fuel {} toolchain available on $PATH...",
//         command.name
//     );
//     if let Ok(mut child) = Command::new(NIX_CMD)
//         .arg(SHELL_ARG)
//         .arg(command.toolchain_link()?)
//         .spawn()
//     {
//         child.wait()?;
//     }

//     Ok(())
// }
