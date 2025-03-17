" Vim options
" Automatically removing all trailing spaces
autocmd BufWritePre * %s/\s\+$//e
" Disable spellcheck in terminal
autocmd TermOpen * setlocal nospell
" set options
set autochdir
set breakindent
set clipboard=unnamedplus
set colorcolumn=80,100
set completeopt=menuone,noinsert,noselect
set complete=.,w,b,u,U,k,kspell,s,i,t
set foldmethod=syntax foldlevelstart=99
set laststatus=0
set linebreak
set list
set nonumber
set noswapfile nowritebackup
set omnifunc=syntaxcomplete#Complete
set scrolloff=1024
set shortmess=IAc
set spell spelllang=en,pl
set splitright splitbelow
set tabstop=4 softtabstop=-1 shiftwidth=0 expandtab
set undofile
set whichwrap+=<,>,h,l,[,]

" Theme
try
    let &background = readfile($HOME . "/.config/nvim/theme.txt")[0]
catch
    let &background = "dark"
endtry

" Formatter
function! Format_file()
    if &readonly | return | endif
    if match(expand('%:p'), "^suda://") != -1 | return | endif

    " Actually run the formatting
    !format.sh %
    " Reload the formatted file
    edit!
endfunction
autocmd BufWritePost * silent call Format_file()

" Key mappings
" Use space as leader
let mapleader=" "
" Move line up/down when wrapped
noremap j gj
noremap k gk
" Paste in visual mode without overwriting
vnoremap p pgvy
" Ctrl + Backspace to remote previous word
noremap! <C-bs> <C-w>
noremap! <C-h> <C-w>
" Don't lose selection after changing indentation
vnoremap > >gv
vnoremap < <gv
" Correct spelling
inoremap <C-l> <C-g>u<esc>[s1z=`]a<c-g>u
noremap <C-l> [s1z=
" Tab completion
function! Tab_complete()
    " Cycle through completions
    if pumvisible() | return "\<C-n>" | endif

    let current_word = matchstr(strpart(getline('.'), -1, col('.')), "[^ \t]*$")
    " indentation
    if strlen(current_word) == 0 | return "\<tab>" | endif
    " File
    if match(current_word, '\/') != -1 | return "\<C-X>\<C-F>" | endif
    " Omni completion
    if match(current_word, '\.') != -1 | return "\<C-X>\<C-O>" | endif
    " Default completion
    return "\<C-X>\<C-P>"
endfunction
inoremap <expr> <tab>   Tab_complete()
inoremap <expr> <S-tab> pumvisible() ? "\<C-p>" : "\<S-tab>"
" Terminal shortcut
function! Terminal_toggle()
    if exists("s:terminal_buffer") && nvim_buf_is_valid(s:terminal_buffer)
        let l:windows_with_buffer = win_findbuf(s:terminal_buffer)
        if empty(l:windows_with_buffer)
            execute "sbuffer " . s:terminal_buffer
            startinsert
        else
            for window_with_buffer in windows_with_buffer
                call nvim_win_close(window_with_buffer, 0)
            endfor
        endif
    else
        new
        let s:terminal_buffer = bufnr()
        terminal
        setlocal nobuflisted
        startinsert
    endif
endfunction
noremap  <C-space>  <cmd>call Terminal_toggle()<cr>
tnoremap <C-space>  <cmd>call Terminal_toggle()<cr>


" Plugins
if ! $VIM_DISABLE_PLUG
    " Install vim-plug automatically
    let data_dir = stdpath('data') . '/site'
    if empty(glob(data_dir . '/autoload/plug.vim'))
        execute '!curl -fLo '.data_dir.'/autoload/plug.vim --create-dirs  https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
        autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
    endif

    call plug#begin()
    Plug 'airblade/vim-gitgutter'  " Git plugin, adding line markers
    Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
    Plug 'junegunn/fzf.vim'
    Plug 'lambdalisue/suda.vim'
    Plug 'tpope/vim-commentary'
    Plug 'tpope/vim-sleuth'  " Automatic tab sizing
    Plug 'tpope/vim-surround'
    call plug#end()
endif


" Plugin options
let g:suda_smart_edit = 1

" Plugins key mappings
noremap <C-/>       :Commentary<cr>|  " `:` to support visual mode ranges
noremap <C-p>       <cmd>History<cr>
noremap <leader>bb  <cmd>Buffers<cr>
noremap <leader>ff  <cmd>Files<cr>
