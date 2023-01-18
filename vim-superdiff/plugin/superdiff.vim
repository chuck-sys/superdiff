" superdiff.vim - Navigate around the JSON reports of the superdiff script
" Maintainer: Cheuk Yin Ng <https://cheuksblog.ca/>
" Version: 1.1

if exists('g:loaded_superdiff')
    finish
endif

let g:loaded_superdiff = 1

command! -nargs=1 -complete=file SDLoad call superdiff#load(<f-args>)
command! -nargs=0 SDUnload call superdiff#unload()
command! -nargs=0 SDLocal call superdiff#query_local_matches()
command! -nargs=0 SDQuery call superdiff#query_matches()
command! -nargs=0 SDHLLocal call superdiff#hl_matches()
command! -nargs=0 SDUnHL sign unplace * group=superdiff

if !exists('g:superdiff_loctext_maxwidth')
    let g:superdiff_loctext_maxwidth = 60
endif

if !exists('g:superdiff_hl_on_call_local')
    let g:superdiff_hl_on_call_local = 1
endif

sign define SuperdiffMatch numhl=Match numhl=ErrorMsg
