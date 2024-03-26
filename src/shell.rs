use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Posix,
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn all() -> Vec<Shell> {
        vec![Shell::Posix, Shell::Bash, Shell::Zsh, Shell::Fish]
    }

    pub fn does_exists(&self) -> bool {
        match self {
            Shell::Posix => true,
            Shell::Bash => true,
            Shell::Zsh => true,
            Shell::Fish => true,
        }
    }

    pub fn rc_files(&self) -> Vec<PathBuf> {
        let home_path = dirs::home_dir().unwrap();
        match self {
            Self::Bash => [".bash_profile", ".bash_login", ".bashrc"]
                .into_iter()
                .map(|s| home_path.join(s))
                .collect::<Vec<_>>(),
            Self::Zsh => [".zshenv", ".zprofile", ".zshrc", ".zlogin"]
                .into_iter()
                .map(|s| home_path.join(s))
                .collect::<Vec<_>>(),
            Self::Posix => [".profile"]
                .into_iter()
                .map(|s| home_path.join(s))
                .collect::<Vec<_>>(),
            Self::Fish => [".config/fish/config.fish"]
                .into_iter()
                .map(|s| home_path.join(s))
                .collect::<Vec<_>>(),
        }
    }
}
