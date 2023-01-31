" vim:foldmethod=marker

" {{{ Install vim-plug automatically
let data_dir = has('nvim') ? stdpath('data') . '/site' : '~/.vim'
if empty(glob(data_dir . '/autoload/plug.vim'))
    execute '!curl -fLo '.data_dir.'/autoload/plug.vim --create-dirs  https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'
    autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
endif
" }}}

" {{{ Plugins to install
call plug#begin()
" {{{ Appearance
" Color scheme
Plug 'tomasiser/vim-code-dark'
" Better status line
Plug 'vim-airline/vim-airline'
" }}}

" {{{ Basics
" fzf
Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }
Plug 'junegunn/fzf.vim'
" Git integration
Plug 'tpope/vim-fugitive'
" Git gutter
Plug 'airblade/vim-gitgutter'
" Surround
Plug 'tpope/vim-surround'
" Comment stuff out
Plug 'tpope/vim-commentary'
" Adjust shiftwidth automatically
Plug 'tpope/vim-sleuth'
" Lines showing indention
Plug 'Yggdroot/indentLine'
" Edit file as sudo
Plug 'lambdalisue/suda.vim'
" Terminal options
Plug 'skywind3000/vim-terminal-help'
" Vim easymotion
Plug 'easymotion/vim-easymotion'
" Tagbar
Plug 'preservim/tagbar'
" NERDTree
Plug 'preservim/nerdtree'
" }}}
call plug#end()
" }}}

" {{{ Extensions configs
" colorscheme
colorscheme codedark
" With nicer spelling underline
highlight SpellCap guisp=yellow  gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
highlight SpellBad guisp=red     gui=undercurl guifg=NONE guibg=NONE ctermfg=NONE ctermbg=NONE term=underline cterm=undercurl
" suda config
" If editing file with no permission to write open it with sudo
let g:suda_smart_edit = 1
" Start terminal with <C-Space> = <C-~>
let g:terminal_key = "<C-space>"
" tabline config
let g:airline#extensions#tabline#enabled = 1
" Easymotion don't map things
let g:EasyMotion_do_mapping = 0
" }}}

" {{{ Plugins key mappings
" Comment with <C-_> = <C-/>
map <C-_>               :Commentary<CR>
" Easy motion
map  s                  <Plug>(easymotion-jumptoanywhere)
" Toggle Tagbar
map <leader>t           :TagbarToggle<CR>
" Toogle NERDTree
map <C-b>               :NERDTreeToggle<CR>
" fzf
map <C-p>               :Files<CR>
map <leader><leader>    :Commands<CR>
" }}}
