let s:json_loaded = v:false
let s:json = {}

function! superdiff#load(filename) abort
    let s:json = json_decode(readfile(a:filename))
    let s:json_loaded = v:true
endfunction

function! superdiff#unload()
    let s:json = {}
    let s:json_loaded = v:false
endfunction

function! superdiff#query_local_matches() abort
    if !s:json_loaded
        echoerr 'superdiff: no file loaded; did you forget to :SDLoad?'
    endif

    let lineno = line('.')

    let loclist = []
    let bufnr = bufnr('%')
    let current_filename = expand('%')
    for m in s:json.matches
        if !has_key(m.files, current_filename)
            continue
        endif

        for block_info in m.blocks[current_filename]
            let text = s:truncate_line(block_info.starting_line) . ' (' . block_info.block_length . ' more lines'
            call add(loclist, {
                        \ 'bufnr': bufnr,
                        \ 'lnum': block_info.starting_line,
                        \ 'text': text,
                        \ })
        endfor
    endfor

    call sort(loclist, {i1, i2 -> i1.lnum - i2.lnum})
    call setloclist(bufnr, loclist)
    lopen
endfunction

function! superdiff#query_matches()
endfunction

function! s:truncate_line(lineno)
    let original = getline(a:lineno)
    if strwidth(original) > g:superdiff_loctext_maxwidth
        return original[:g:superdiff_loctext_maxwidth] . '...'
    else
        return original
    endif
endfunction
