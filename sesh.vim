let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/dev/rust/polar-arctic
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
set shortmess=aoO
badd +29 src/blue/mod.rs
badd +109 src/lib.rs
badd +16 ~/dev/rust/polar-arctic/src/blue/setting.rs
badd +1 src/blue/fs.rs
badd +5 ~/dev/rust/fs-tokio-test/src/main.rs
badd +83 src/menu.rs
badd +294 ~/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/string.rs
badd +24 Cargo.toml
badd +73 ~/.config/nvim/init.vim
badd +71 src/data.rs
badd +35 ~/.cargo/registry/src/github.com-1ecc6299db9ec823/iced_native-0.5.1/src/command.rs
badd +16 src/modal/mod.rs
badd +8 src/modal/ble.rs
badd +853 ~/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs
argglobal
%argdel
$argadd NvimTree_1
edit src/lib.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
wincmd _ | wincmd |
vsplit
2wincmd h
wincmd w
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 30 + 119) / 238)
exe 'vert 2resize ' . ((&columns * 96 + 119) / 238)
exe 'vert 3resize ' . ((&columns * 110 + 119) / 238)
argglobal
enew
file NvimTree_1
balt src/menu.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal nofen
wincmd w
argglobal
balt ~/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 127 - ((40 * winheight(0) + 29) / 59)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 127
normal! 045|
wincmd w
argglobal
if bufexists(fnamemodify("~/.cargo/registry/src/github.com-1ecc6299db9ec823/iced_native-0.5.1/src/command.rs", ":p")) | buffer ~/.cargo/registry/src/github.com-1ecc6299db9ec823/iced_native-0.5.1/src/command.rs | else | edit ~/.cargo/registry/src/github.com-1ecc6299db9ec823/iced_native-0.5.1/src/command.rs | endif
if &buftype ==# 'terminal'
  silent file ~/.cargo/registry/src/github.com-1ecc6299db9ec823/iced_native-0.5.1/src/command.rs
endif
balt src/lib.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 28 - ((22 * winheight(0) + 29) / 59)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 28
normal! 012|
wincmd w
2wincmd w
exe 'vert 1resize ' . ((&columns * 30 + 119) / 238)
exe 'vert 2resize ' . ((&columns * 96 + 119) / 238)
exe 'vert 3resize ' . ((&columns * 110 + 119) / 238)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
