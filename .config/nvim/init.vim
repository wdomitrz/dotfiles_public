" vim:foldmethod=marker

" {{{ Vim options
source ~/.config/nvim/options.vim
" }}}

" {{{ Keymap
source ~/.config/nvim/keymap.vim
" }}}

" {{{ Enable plugins
if $VIM_DISABLE_PLUGINS != 1
source ~/.config/nvim/plugins.vim
endif
" }}}
