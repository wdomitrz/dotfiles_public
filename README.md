These are my dotfiles.

### Compatibility

They work with the current version of [Debian stable](https://www.debian.org/releases/stable/).

### Cloning the repo

To make everything work with almost no effort one might clone the repo, and then move the `.git` directory to their home directory.

```bash
git clone https://github.com/wdomitrz/dotfiles_public
mv dotfiles_public/.git ~/
```

Now you can checkout all the files, but before doing so, it is a good idea to check what will change.

```bash
git diff
```

Now, if you want, you can checkout all the files.

```bash
git checkout -- .
```

Or just the selected files.

```bash
git checkout -- <path_to_the_first_file> <path_to_the_second_file>
```

### Installing

The main installation is done using `~/.local/bin/install_scripts/main.sh` script. Just run the script.

Before you do it note that it will also remove some packages in script `~/.local/bin/install_scripts/clear_packages.sh`. To avoid it, you can overwrite this script `echo "" > ~/.local/bin/install_scripts/clear_packages.sh`.

```bash
~/.local/bin/install_scripts/main.sh
```
