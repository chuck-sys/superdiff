" superdiff.vim - Navigate around the JSON reports of the superdiff script
" Maintainer: Cheuk Yin Ng <https://cheuksblog.ca/>
" Version: 1.0

if exists('g:loaded_superdiff')
    finish
endif

let g:loaded_superdiff = 1

command! -nargs=1 -complete=file SDLoad call superdiff#load(<f-args>)
command! -nargs=0 SDUnload call superdiff#unload()
command! -nargs=0 SDLocal call superdiff#query_local_matches()
command! -nargs=0 SDQuery call superdiff#query_matches()

if !exists('g:superdiff_loctext_maxwidth')
    let g:superdiff_loctext_maxwidth = 60
endif
