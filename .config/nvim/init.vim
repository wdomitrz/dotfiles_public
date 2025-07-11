" Vim options
" Automatically removing all trailing spaces and blank trailing lines
autocmd BufWritePre * %substitute/\s\+$//e
autocmd BufWritePre * %substitute/$\n\+\%$//e
" Formatting
autocmd BufWritePre * silent Format
" Disable spellcheck in terminal
autocmd TermOpen * setlocal nospell
" set options
set autochdir
set breakindent
set clipboard=unnamedplus
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
set smartcase
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

" Commands
function! Format_fn()
    let current_pos = getpos(".")
    execute '%!format.sh stdin --filename %'
    call setpos(".", current_pos)
endfunction
command! Format             call Format_fn()
command! CodeAction         lua vim.lsp.buf.code_action()
command! NextDiagnostic     lua vim.diagnostic.goto_next()
command! PreviousDiagnostic lua vim.diagnostic.goto_prev()

" Key mappings
" Use space as leader and , as local leader
let mapleader=" "
let maplocalleader=","
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
" LSP
" Code action
noremap <C-.>           <cmd>CodeAction<CR>
noremap <localleader>en <cmd>NextDiagnostic<CR>
noremap <localleader>ep <cmd>PreviousDiagnostic<CR>
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
    packadd fzf
    packadd fzf.vim
    packadd nvim-lspconfig
    packadd suda.vim
    packadd vim-commentary
    packadd vim-gitgutter
    packadd vim-sleuth
    packadd vim-surround
endif


" Plugin options
let $FZF_DEFAULT_OPTS = '--reverse'
let g:fzf_layout = { 'window': { 'width': 0.9, 'height': 0.6, 'yoffset': 0.0 } }
let g:fzf_preview_window = []

" Plugins key mappings
noremap <C-/>               :Commentary<cr>|  " `:` to support visual mode ranges
noremap <C-p>               <cmd>History<cr>
noremap <leader>ff          <cmd>Files<cr>
noremap <leader>sf          <cmd>Rg<cr>
noremap <leader><leader>    <cmd>Commands<cr>

" LSP servers
if has('nvim-0.11')
    lua vim.lsp.config['shls'] = { cmd = { 'shls.py' }, filetypes = { 'bash', 'sh' } }
    lua vim.lsp.enable('shls')
    lua vim.lsp.enable('ruff')
    lua vim.lsp.enable('clangd')
endif
