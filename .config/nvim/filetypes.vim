" vim:foldmethod=marker

" {{{ Custom filetype
" Dockerfile
autocmd BufNewFile,BufFilePre,BufRead Dockerfile* setlocal filetype=dockerfile
" Markdownlint rc as json
autocmd BufNewFile,BufFilePre,BufRead *markdownlint setlocal filetype=json
" pylint rc as ini file
autocmd BufNewFile,BufFilePre,BufRead *pylintrc setlocal filetype=dosini
" Prolog
autocmd BufNewFile,BufFilePre,BufRead *.pl setlocal filetype=prolog
" }}}

" {{{ File type specific settings
" Markdown
autocmd Filetype markdown setlocal makeprg=pandoc\ -o\ %:r.pdf\ %\ --from\ markdown+lefinition_lists\ --wrap=preserve
" Python
autocmd FileType python setlocal foldmethod=indent
" }}}
