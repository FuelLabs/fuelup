# Configuration

_fuelup_ has a [TOML](https://github.com/toml-lang/toml) settings file at
`.fuelup/settings.toml`. The schema for this file is not part of the public
interface for _fuelup_ - the fuelup CLI should be used to query and set settings.

## Generate Shell Completions

Enable tab completion for Bash, Fish, Zsh, or PowerShell. The script prints output on `stdout`,
allowing one to re-direct the output to the file of their choosing. Where you place the file will
depend on which shell, and which operating system you are using. Your particular configuration may
also determine where these scripts need to be placed.

Here are some common set ups for the supported shells under Unix and similar operating systems
(such as GNU/Linux). For these settings to take effect, you may have to log out and log back in to
your shell session.

### BASH

Completion files are commonly stored in `/etc/bash_completion.d/` for system-wide commands, but can
be stored in `~/.local/share/bash-completion/completions` for user-specific commands.

```sh
mkdir -p ~/.local/share/bash-completion/completions
fuelup completions --shell=bash >> ~/.local/share/bash-completion/completions/fuelup
```

## BASH (macOS/Homebrew)

Homebrew stores bash completion files within the Homebrew directory. With the `bash-completion` brew
formula installed.

```sh
mkdir -p $(brew --prefix)/etc/bash_completion.d
fuelup completions --shell=bash > $(brew --prefix)/etc/bash_completion.d/fuelup.bash-completion
```

### FISH

Fish completion files are commonly stored in `$HOME/.config/fish/completions`.

```sh
mkdir -p ~/.config/fish/completions
fuelup completions --shell=fish > ~/.config/fish/completions/fuelup.fish
```

### ZSH

ZSH completions are commonly stored in any directory listed in your `$fpath` variable. To use these
completions, you must either add the generated script to one of those directories, or add your own
to this list.

Adding a custom directory is often the safest bet if you are unsure of which directory to use. First
create the directory; for this example we'll create a hidden directory inside our `$HOME` directory:

```sh
mkdir ~/.zfunc
```

Then add the following lines to your `.zshrc` just before `compinit`:

```sh
fpath+=~/.zfunc
```

Now you can install the completions script using the following command:

```sh
fuelup completions --shell=zsh > ~/.zfunc/_fuelup
```

### POWERSHELL

The powershell completion scripts require PowerShell v5.0+ (which comes with Windows 10, but can be
downloaded separately for windows 7 or 8.1).

First, check if a profile has already been set

```sh
Test-Path $profile
```

If the above command returns `False` run the following

```sh
New-Item -path $profile -type file -force
```

Now open the file provided by `$profile` (if you used the `New-Item` command it will be
`${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`).

Next, we either save the completions file into our profile, or into a separate file and source it
inside our profile. To save the completions into our profile simply use

```sh
fuelup completions --shell=powershell >> ${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
```

### ELVISH

Elvish completions are commonly stored in [`epm`](https://elv.sh/ref/epm.html#the-epm-managed-directory)-managed
directories.

```sh
fuelup completions --shell=elvish > ~/.local/share/elvish/lib/fuelup.elv
```

Then in [`rc.elv`](https://elv.sh/ref/command.html#rc-file), add the following line to activate the
generated completions.

```sh
use fuelup
```
