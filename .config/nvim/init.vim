" Vim options
" Automatically removing all trailing whitespace
autocmd BufWritePre * %s/\s\+$//e
" Disable spellcheck in terminal
autocmd TermOpen * setlocal nospell nonumber norelativenumber
" set options
set laststatus=3
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

" Keymap
" Use space as leader
let mapleader=" "
" Move line up/down when wrapped
noremap <expr> j v:count == 0 ? 'j' : 'gj'
noremap <expr> k v:count == 0 ? 'k' : 'gk'
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
if $VIM_DISABLE_PLUG != 1
" Install vim-plug automatically
let data_dir = stdpath('data') . '/site'
if empty(glob(data_dir . '/autoload/plug.vim'))
    execute '!curl -fLo '.data_dir.'/autoload/plug.vim --create-dirs  https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
    autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
endif

call plug#begin()
Plug 'airblade/vim-gitgutter'
Plug 'easymotion/vim-easymotion'
Plug 'lambdalisue/suda.vim'
Plug 'Mofiqul/vscode.nvim'
Plug 'nvim-lua/plenary.nvim'
Plug 'nvim-telescope/telescope.nvim'
Plug 'preservim/nerdtree'
Plug 'preservim/tagbar'
Plug 'tpope/vim-commentary'
Plug 'tpope/vim-fugitive'
Plug 'tpope/vim-sleuth'
Plug 'tpope/vim-surround'
Plug 'vim-airline/vim-airline'
Plug 'Yggdroot/indentLine'
call plug#end()
endif


" Plugin options
" Theme
try
    let &background = readfile($HOME . "/.config/nvim/theme.txt")[0]
catch
    let &background = "dark"
endtry
if has('nvim-0.9') | colorscheme vscode | endif
" With nicer spelling underline
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl

let g:EasyMotion_do_mapping = 0
let g:markdown_syntax_conceal = 0
let g:suda_smart_edit = 1
let g:vim_json_conceal = 0

" Plugins key mappings
map <C-b>               <cmd>NERDTreeToggle<cr>
map <C-/>               <cmd>Commentary<cr>
map <C-p>               <cmd>lua require('telescope.builtin').find_files({no_ignore=true, hidden=true})<cr>
map <leader><leader>    <cmd>Telescope commands<cr>
map <leader>t           <cmd>TagbarToggle<cr>
map s                   <plug>(easymotion-jumptoanywhere)
