set laststatus=0 cmdheight=0

enew
setlocal buftype=nofile bufhidden=wipe

let s:file = g:kitty_scrollback_file
let s:top  = g:kitty_scrollback_top
let s:line = g:kitty_scrollback_line
let s:col  = g:kitty_scrollback_col

let s:target_line = s:top + max([0, s:line - 1])

if filereadable(s:file)
    setlocal modifiable
    let s:ch = nvim_open_term(0, {})
    call chansend(s:ch, join(readfile(s:file, 'b'), "\n"))
    call delete(s:file)
    setlocal nomodifiable
endif

noremap <silent> q <cmd>qa!<CR>

function! s:restore_position() abort
    call cursor(s:top, 1)
    normal! zt
    call cursor(s:target_line, virtcol2col(0, s:target_line, max([1, s:col])))
endfunction

call timer_start(50, {-> s:restore_position()})
autocmd BufLeave <buffer> qa!
