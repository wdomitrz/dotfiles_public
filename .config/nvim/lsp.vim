" vim:foldmethod=marker

" {{{ Extensions
let g:coc_global_extensions = [
            \ 'coc-clangd',
            \ 'coc-css',
            \ 'coc-docker',
            \ 'coc-highlight',
            \ 'coc-html',
            \ 'coc-json',
            \ 'coc-markdownlint',
            \ 'coc-pyright',
            \ 'coc-sh',
            \ 'coc-texlab',
            \ 'coc-vimlsp',
            \ 'coc-yaml',
            \ ]
" }}}
" {{{ Install
" {{{ Install automatically
if empty(glob('~/.config/coc'))
    autocmd VimEnter * PlugInstall --sync
endif
" }}}
" {{{ Install using plug
call plug#end()
Plug 'neoclide/coc.nvim', {'branch': 'release', 'do': { -> coc#util#install()}}
call plug#end()
" }}}
" }}}

" {{{ vim options
" set cmdheight=2
set updatetime=300
set shortmess+=c
" }}}

" {{{ highlighting hovered items
autocmd CursorHold * silent call CocActionAsync('highlight')
" }}}

" {{{ Completion mappings
" Tab completion
function! s:check_back_space() abort
  let col = col('.') - 1
  return !col || getline('.')[col - 1]  =~# '\s'
endfunction
noremap <silent><expr> <TAB>
      \ pumvisible() ? "\<C-n>" :
      \ <SID>check_back_space() ? "\<TAB>" :
      \ coc#refresh()

" CR selects the first completion
inoremap <silent><expr> <cr> pumvisible() ? coc#_select_confirm()
                              \: "\<C-g>u\<CR>\<c-r>=coc#on_enter()\<CR>"
" }}}

" {{{ Helper shortcut function
function! s:show_documentation()
  if (index(['vim','help'], &filetype) >= 0)
    execute 'h '.expand('<cword>')
  elseif (coc#rpc#ready())
    call CocActionAsync('doHover')
  else
    execute '!' . &keywordprg . " " . expand('<cword>')
  endif
endfunction
" }}}

" {{{ Key mappings
nmap <leader>gD <Plug>(coc-declaration)
nmap <leader>gd <Plug>(coc-definition)
nmap <leader>K  :call <SID>show_documentation()<CR>
nmap <leader>gi <Plug>(coc-implementation)
nmap <C-k>      :call CocAction('showSignatureHelp')<CR>
" nmap <leader>wa :lua vim.lsp.buf.add_workspace_folder()<CR>
" nmap <leader>wr :lua vim.lsp.buf.remove_workspace_folder()<CR>
nmap <leader>wl :CocList folders<CR>
nmap <leader>D  <Plug>(coc-type-definition)
nmap <leader>rn <Plug>(coc-rename)
nmap <leader>gr <Plug>(coc-references)
nmap <leader>e  :call CocAction('diagnosticToggle')<CR>
nmap <leader>[  <Plug>(coc-diagnostic-prev)
nmap <leader>]  <Plug>(coc-diagnostic-next)
nmap <leader>{  <Plug>(coc-diagnostic-prev-error)
nmap <leader>}  <Plug>(coc-diagnostic-next-error)
nmap <leader>q  :CocList location<CR>
nmap <leader>f  <Plug>(coc-format-selected)
xmap <leader>f  <Plug>(coc-format-selected)
nmap <leader>ac <Plug>(coc-codeaction)
nmap <leader>as <Plug>(coc-codeaction-selected)
xmap <leader>as <Plug>(coc-codeaction-selected)
nmap <leader>qf <Plug>(coc-fix-current)
nmap <leader>cl <Plug>(coc-codelens-action)
" Mappings for CoCList
" Show all diagnostics.
nmap <leader>la :CocList diagnostics<cr>
nmap <leader>le :CocList extensions<cr>
nmap <leader>lc :CocList commands<cr>
nmap <leader>lo :CocList outline<cr>
nmap <leader>ls :CocList -I symbols<cr>
nmap <leader>lj :CocNext<CR>
nmap <leader>lk :CocPrev<CR>
nmap <leader>lp :CocListResume<CR>
" }}}

" {{{ Custom commands
" Add `:Format` command to format current buffer.
command! -nargs=0 Format :call CocAction('format')

" Add `:Fold` command to fold current buffer.
command! -nargs=? Fold :call     CocAction('fold', <f-args>)

" Add `:OR` command for organize imports of the current buffer.
command! -nargs=0 OR   :call     CocActionAsync('runCommand', 'editor.action.organizeImport')
" }}}
