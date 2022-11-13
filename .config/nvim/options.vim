" vim:foldmethod=marker

" {{{ Automatically removing all trailing whitespace
autocmd BufWritePre * %s/\s\+$//e
" }}}

" {{{ file type and syntax
" File type stuff
filetype plugin indent on
" Syntax highlighting
if has("syntax") | syntax enable | endif
" }}}

" {{{ set options
" Automatically set working
set autochdir
" Use system clipboard
set clipboard=unnamedplus
" Set completion from current file, windows loaded and unloaded buffers, tags, includes
set complete=.,w,b,u,U,k,kspell,s,i,t
" Set completeopt to have a better completion experience
set completeopt=menuone,noinsert,noselect
" Fold using syntax and dont fold by default
set foldmethod=syntax nofoldenable
" Always show status line and tab line
set showtabline=2
" Global statusline
if has('nvim-0.7') | set laststatus=3 | endif
" Do not update when not needed
set lazyredraw
" Show extra spaces
set list
" Support mouse
set mouse=a
" Do not use swap files and backups
set noswapfile nobackup nowritebackup
" set line numbers
set number norelativenumber
" Set ruler after 80 line
set ruler colorcolumn=81
" Keep distance from the border
set scrolloff=3
" set no start message, no existing swap file info, no completion-menu message
set shortmess=IAc
" Automatically hide and show sign column
if has('nvim-0.5') | set signcolumn=number | else | set signcolumn=auto | endif
" Smartcase
set smartcase
" Add spell checking
set spell spelllang=en,pl
" Split to the right and to the bottom
set splitright splitbelow
" Show tabs as 4 spaces, indent with 4 spaces, expand tabs to spaces
set tabstop=4 softtabstop=-1 shiftwidth=0 expandtab
" Use real colors
set termguicolors
" Omnicomplete
set omnifunc=syntaxcomplete#Complete
" Maintain undo history between sessions
set undofile
" Shorter update time
set updatetime=100
" Wrap cursor
set whichwrap+=<,>,h,l,[,]
" Soft wrapping
set wrap linebreak breakindent
" }}}

" {{{ Set nicer spelling highlighting
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
" }}}

" {{{ Set undodir for nvim and vim
if has('nvim') | set undodir=~/.cache/vim-undodir | endif
" Disable spellcheck in terminal
if has('nvim') | autocmd TermOpen * setlocal nospell nonumber norelativenumber
else | autocmd TerminalOpen * setlocal nospell nonumber norelativenumber
endif
" }}}
