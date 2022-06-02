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
" Replace with register
Plug 'vim-scripts/ReplaceWithRegister'
" CalmeCaseMotion
Plug 'bkad/CamelCaseMotion'
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
" Multiple coursors
Plug 'terryma/vim-multiple-cursors'
" Delete buffers without closing windows
Plug 'moll/vim-bbye'
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
" }}}

" {{{ Syntax highlighting
" Markdown
Plug 'tpope/vim-markdown'
" Haskell
Plug 'neovimhaskell/haskell-vim'
" Prolog
Plug 'mxw/vim-prolog'
" BNFC
Plug 'neapel/vim-bnfc-syntax'
" i3
Plug 'PotatoesMaster/i3-vim-syntax'
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
" }}}

" {{{ Plugins key mappings
" Comment
"" Vim registers <C-/> as <C-_>
map <C-_> :Commentary<CR>
" CamelCaseMotion
map <silent> \w <Plug>CamelCaseMotion_w
map <silent> \b <Plug>CamelCaseMotion_b
map <silent> \e <Plug>CamelCaseMotion_e
map <silent> \ge <Plug>CamelCaseMotion_ge
" Easy motion
map  f          <Plug>(easymotion-jumptoanywhere)
" Git Gutter
nmap <leader>G[ :GitGutterPrevHunk<CR>
nmap <leader>G] :GitGutterNextHunk<CR>
nmap <leader>Gi :GitGutterPreviewHunk<CR>
nmap <leader>Gs :GitGutterStageHunk<CR>
nmap <leader>Gu :GitGutterUndoHunk<CR>
" Toggle Tagbar
nmap <leader>t  :TagbarToggle<CR>
" Toogle NERDTree
nmap <C-b>      :NERDTreeToggle<CR>
" fzf
nmap <C-p>      :Files<CR>
" bbye
nmap <leader>q  :Bdelete<CR>
" }}}
