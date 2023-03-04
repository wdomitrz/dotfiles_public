" vim:foldmethod=marker

" {{{ Vim options
" Automatically removing all trailing whitespace
autocmd BufWritePre * %s/\s\+$//e
" Disable spellcheck in terminal
autocmd TermOpen * setlocal nospell nonumber norelativenumber
" set options
if has('nvim-0.7') | set laststatus=3 | endif
set autochdir
set clipboard=unnamedplus
set colorcolumn=80,88,100,115,120
set completeopt=menuone,noinsert,noselect
set complete=.,w,b,u,U,k,kspell,s,i,t
set foldmethod=syntax nofoldenable
set lazyredraw
set linebreak breakindent
set list
set mouse=a
set noswapfile nowritebackup
set number
set omnifunc=syntaxcomplete#Complete
set scrolloff=1024
set shortmess=IAc
set spell spelllang=en,pl
set splitright splitbelow
set tabstop=4 softtabstop=-1 shiftwidth=0 expandtab
set termguicolors
set undofile
set updatetime=100
set whichwrap+=<,>,h,l,[,]
" Set nicer spelling highlighting
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
" }}}

" {{{ Keymap
" Use space as leader
let mapleader=" "
" Move line up/down when wrapped
noremap <expr> j v:count == 0 ? 'j' : 'gj'
noremap <expr> k v:count == 0 ? 'k' : 'gk'
" Ctrl + Backspace to remote previous word
noremap! <C-BS> <C-w>
noremap! <C-h> <C-w>
" Don't lose selection after changing indentation
vnoremap > >gv
vnoremap < <gv
" Correct spelling
inoremap <C-l> <C-g>u<Esc>[s1z=`]a<c-g>u
noremap <C-l> [s1z=
" Tab completion
function! TabComplete()
    " Cycle through completions
    if pumvisible() | return "\<C-n>" | endif

    let current_word = matchstr(strpart(getline('.'), -1, col('.')), "[^ \t]*$")
    " indentation
    if strlen(current_word) == 0 | return "\<tab>" | endif
    " File
    if match(current_word, '\/') != -1 | return "\<C-X>\<C-F>" | endif
    " Omni completion
    " if match(current_word, '\.') != -1 | return "\<C-X>\<C-O>" | endif
    " Default completion
    return "\<C-X>\<C-P>"
endfunction

inoremap <expr> <TAB>   TabComplete()
inoremap <expr> <S-TAB> pumvisible() ? "\<C-p>" : "\<S-Tab>"
" }}}

" {{{ Enable plugins
if $VIM_DISABLE_PLUGINS != 1
" Install vim-plug automatically
let data_dir = has('nvim') ? stdpath('data') . '/site' : '~/.vim'
if empty(glob(data_dir . '/autoload/plug.vim'))
    execute '!curl -fLo '.data_dir.'/autoload/plug.vim --create-dirs  https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
    autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
endif

call plug#begin()
Plug 'tomasiser/vim-code-dark'
Plug 'vim-airline/vim-airline'
Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
Plug 'junegunn/fzf.vim'
Plug 'tpope/vim-fugitive'
Plug 'airblade/vim-gitgutter'
Plug 'tpope/vim-surround'
Plug 'tpope/vim-commentary'
Plug 'tpope/vim-sleuth'
Plug 'Yggdroot/indentLine'
Plug 'lambdalisue/suda.vim'
Plug 'skywind3000/vim-terminal-help'
Plug 'easymotion/vim-easymotion'
Plug 'preservim/tagbar'
Plug 'preservim/nerdtree'
call plug#end()

" Extensions configs
colorscheme codedark
" With nicer spelling underline
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl

let g:airline#extensions#tabline#enabled = 1
let g:EasyMotion_do_mapping = 0
let g:markdown_syntax_conceal=0
let g:suda_smart_edit = 1
let g:terminal_key = "<C-space>"
let g:vim_json_conceal=0

" Plugins key mappings
map <C-b>               :NERDTreeToggle<CR>
map <C-_>               :Commentary<CR>
map <C-p>               :Files<CR>
map <leader><leader>    :Commands<CR>
map <leader>t           :TagbarToggle<CR>
map s                   <Plug>(easymotion-jumptoanywhere)
endif
" }}}
