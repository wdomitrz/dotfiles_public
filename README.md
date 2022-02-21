These are my dotfiles.

### Compatibility

They work with Ubuntu 20.04. With a little bit of work (removing some packages from `~/.config/scripts/install-scripts/install-packages.sh`) they should work under Debian 11. Making them work under other distribution might require manual installation of packages (and if the system does not use `systemd`, then some more work).

### Cloning the repo

To make everything work with almost no effort one might clone the repo, and then move the `.git` directory to their home directory.

```bash
git clone https://github.com/wdomitrz/dotfiles-public
mv dotfiles-public/.git ~/
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

The main installation is done using `~/.config/scripts/install.sh` script. Just run the script.

```bash
~/.config/scripts/install.sh
```
