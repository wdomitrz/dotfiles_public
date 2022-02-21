" vim:foldmethod=marker

" {{{ Vim options
source ~/.config/nvim/options.vim
" }}}

" {{{ Filetypes
source ~/.config/nvim/filetypes.vim
" }}}

" {{{ Keymap
source ~/.config/nvim/keymap.vim
" }}}

" {{{ Enable plugins
if $VIM_DISABLE_PLUGINS != 1
source ~/.config/nvim/plugins.vim
" {{{ coc (Conquer of Completion)
if $VIM_DISABLE_LSP != 1
source ~/.config/nvim/lsp.vim
endif
" }}}
endif
" }}}
