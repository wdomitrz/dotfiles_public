These are my dotfiles.

### Compatibility

They work with Ubuntu 20.04. With a little bit of work (removing some packages from `~/.config/scripts/install-scripts/install-packages.sh`) they should work under Debian 11. Making them work under other distribution might require manual installation of packages (and if the system does not use `systemd`, then some more work).

### Cloning the repo

To make everything work with almost no effort one might clone the repo, and then move the `.git` directory to their home directory:

```bash
git clone https://github.com/wdomitrz/dotfiles-public
mv dotfiles-public/.git ~/
```

### Installing

The main installation is done using `` script. Just run:

```bash
~/.config/scripts/install.sh
```
