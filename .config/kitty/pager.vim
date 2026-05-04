set clipboard=unnamedplus
set cmdheight=0
set laststatus=0
set scrolloff=0
set shortmess=IAcFO
set sidescrolloff=0
set virtualedit=all

enew
setlocal noreadonly modifiable
setlocal noswapfile bufhidden=wipe

let s:top = str2nr($KITTY_PAGER_TOP)
let s:cursor_line = str2nr($KITTY_PAGER_CURSOR_LINE)
let s:cursor_col = str2nr($KITTY_PAGER_CURSOR_COL)

if s:top <= 0
    let s:top = 1
endif

if s:cursor_col <= 0
    let s:cursor_col = 1
endif

if s:cursor_line <= 0
    let s:target_line = s:top
else
    let s:target_line = s:top + s:cursor_line - 1
endif

let s:chan = nvim_open_term(0, {})
let s:data = join(readfile($KITTY_PAGER_FILE, 'b'), "\n")
call chansend(s:chan, s:data)

silent! call delete($KITTY_PAGER_FILE)

silent! setlocal noreadonly nomodifiable nomodified

nnoremap <silent> <buffer> q :qa!<CR>

function! s:restore_position() abort
    " Put the same terminal line at the top of the pager.
    call cursor(s:top, 1)
    normal! zt

    " Convert Kitty's terminal/screen column to a real buffer byte column.
    let l:byte_col = virtcol2col(0, s:target_line, s:cursor_col)

    if l:byte_col <= 0
        let l:byte_col = 1
    endif

    " Move the real Neovim cursor as a best effort.
    call cursor(s:target_line, l:byte_col)

    " Paint the visible cursor exactly where Kitty reported it.
    call matchaddpos('Cursor', [[s:target_line, l:byte_col, 1]], 100)
endfunction

call timer_start(100, {-> s:restore_position()})
