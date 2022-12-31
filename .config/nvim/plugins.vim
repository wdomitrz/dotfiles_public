" vim:foldmethod=marker

" {{{ Install vim-plug automatically
let data_dir = has('nvim') ? stdpath('data') . '/site' : '~/.vim'
if empty(glob(data_dir . '/autoload/plug.vim'))
    execute '!curl -fLo '.data_dir.'/autoload/plug.vim --create-dirs  https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
    autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
endif
" }}}

" {{{ Plugins to install
call plug#begin()
" {{{ Appearance
" Color scheme
Plug 'tomasiser/vim-code-dark'
" Better status line
Plug 'vim-airline/vim-airline'
" }}}

" {{{ Basics
" LSP and other
Plug 'williamboman/mason.nvim'
Plug 'williamboman/mason-lspconfig.nvim'
Plug 'neovim/nvim-lspconfig'
" fzf
Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
Plug 'junegunn/fzf.vim'
" Git integration
Plug 'tpope/vim-fugitive'
" Git gutter
Plug 'airblade/vim-gitgutter'
" Surround
Plug 'tpope/vim-surround'
" Comment stuff out
Plug 'tpope/vim-commentary'
" Adjust shiftwidth automatically
Plug 'tpope/vim-sleuth'
" Lines showing indention
Plug 'Yggdroot/indentLine'
" Edit file as sudo
Plug 'lambdalisue/suda.vim'
" Terminal options
Plug 'skywind3000/vim-terminal-help'
" Vim easymotion
Plug 'easymotion/vim-easymotion'
" Tagbar
Plug 'preservim/tagbar'
" NERDTree
Plug 'preservim/nerdtree'
" Notebooks and python
Plug 'goerz/jupytext.vim'
Plug 'kana/vim-textobj-user'
Plug 'GCBallesteros/vim-textobj-hydrogen'
Plug 'hkupty/iron.nvim'
" }}}
call plug#end()
" }}}

" {{{ Extensions configs
" colorscheme
colorscheme codedark
" With nicer spelling underline
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
" suda config
" If editing file with no permission to write open it with sudo
let g:suda_smart_edit = 1
" airline config
" Open terminal using ctrl + space, which also works as ctrl + tilde
if has('nvim') | let g:terminal_key = "<C-Space>" |
else | let g:terminal_key = "<Nul>" | endif
" tabline config
let g:airline#extensions#tabline#enabled = 1
" mason - LSP
if has('nvim-0.7')
lua << EOF
require("mason").setup()
require("mason-lspconfig").setup()
EOF
endif
" jupytext
" python notebook text format
let g:jupytext_fmt='py:percent'
" }}}

" {{{ Plugins key mappings
" Comment
"" Vim registers <C-/> as <C-_>
map <C-_> <cmd>Commentary<CR>
" Easy motion
map  s          <Plug>(easymotion-jumptoanywhere)
" Git Gutter
nmap <leader>G[ <cmd>GitGutterPrevHunk<CR>
nmap <leader>G] <cmd>GitGutterNextHunk<CR>
nmap <leader>Gi <cmd>GitGutterPreviewHunk<CR>
nmap <leader>Gs <cmd>GitGutterStageHunk<CR>
nmap <leader>Gu <cmd>GitGutterUndoHunk<CR>
" Toggle Tagbar
nmap <leader>t  <cmd>TagbarToggle<CR>
" Toogle NERDTree
nmap <C-b>      <cmd>NERDTreeToggle<CR>
" telescope
nmap <C-p>      <cmd>Files<cr>
" }}}

" Running python in ipython
if has('nvim-0.7')
lua << EOF
require("iron.core").setup {
  config = {
    repl_open_cmd = "vertical split"
  },
  keymaps = {
    send_motion = "<space>sc",
    visual_send = "<space>sc",
    send_file = "<space>sf",
  },
}
EOF
endif
