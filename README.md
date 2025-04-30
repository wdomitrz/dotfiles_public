These are my dotfiles.

### Compatibility

Works with [Debian stable](https://www.debian.org/releases/stable/).

### Cloning the repo

```bash
git clone https://github.com/wdomitrz/dotfiles_public
mv dotfiles_public/.git ~/
```

#### Check what will change

```bash
git diff
```

#### Checkout all the files

```bash
git checkout -- .
```

#### Or selected files

```bash
git checkout -- <path_to_the_first_file> <path_to_the_second_file>
```

### Installing

```bash
~/.local/bin/install_scripts/main.sh
```
