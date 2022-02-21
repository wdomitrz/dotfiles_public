" vim:foldmethod=marker

" {{{ Keymap
" Use space as leader
if exists(":let") | let mapleader=" " | endif
" Remap keys
" Move line up/down when wrapped
noremap <expr> j v:count ? 'j' : 'gj'
noremap <expr> k v:count ? 'k' : 'gk'
" Ctrl + Backspace to remote previous word
noremap! <C-BS> <C-w>
noremap! <C-h> <C-w>
" Tabs
"" Create and close tabs
map <silent> <C-t> :tabnew<CR>
map <silent> <C-q> :tabclose<CR>
map <silent> <C-i> :tabnext<CR>
" Double leader moves courser to different window
map <leader><leader> <C-w><C-w>
" Don't lose selection after changing indentation
vnoremap > >gv
vnoremap < <gv
" Use jk in insert mode as Esc
noremap! jk <Esc>
" Correct spelling
inoremap <C-l> <C-g>u<Esc>[s1z=`]a<c-g>u
noremap <C-l> [s1z=
" {{{ Better tabs navigation
cabbrev tn tabnew
cabbrev th tabprev
cabbrev tl tabnext
" }}}
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
