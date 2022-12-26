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
            let lnum = block_info.starting_line
            let size = block_info.block_length - 1
            if size > 0
                let text = s:truncate_line(lnum) . ' (' . l:size . ' more lines)'
            else
                let text = s:truncate_line(lnum)
            endif

            call add(loclist, {
                        \ 'bufnr': bufnr,
                        \ 'lnum': lnum,
                        \ 'text': text,
                        \ })
        endfor
    endfor

    call sort(loclist, {i1, i2 -> i1.lnum - i2.lnum})
    call setloclist(bufnr, loclist)
    lopen
endfunction

function! superdiff#query_matches()
    if !s:json_loaded
        echoerr 'superdiff: no file loaded; did you forget to :SDLoad?'
    endif

    let lineno = line('.')
    let bufnr = bufnr('%')
    let current_filename = expand('%')
    let loclist = []
    for m in s:json.matches
        if !has_key(m.files, current_filename)
            continue
        endif

        let is_line_within_match = v:false
        for block_info in m.blocks[current_filename]
            if s:line_in_block(lineno, block_info.starting_line, block_info.block_length)
                let is_line_within_match = v:true
                break
            endif
        endfor

        if is_line_within_match
            let loclist = loclist + s:blocks_to_loclist(current_filename, m.blocks)
        endif
    endfor

    call sort(loclist, {i1, i2 -> i1.lnum - i2.lnum})
    call sort(loclist, {i1, i2 -> i1.filename - i2.filename})
    call uniq(loclist)
    call setloclist(bufnr, loclist)
    lopen
endfunction

function! s:blocks_to_loclist(current_filename, blocks)
    let loclist = []
    for [filename, block_infos] in items(a:blocks)
        if filename == a:current_filename
            continue
        endif

        for i in block_infos
            call add(loclist, {
                        \ 'filename': filename,
                        \ 'lnum': i.starting_line,
                        \ 'text': i.block_length . ' line(s)',
                        \ })
        endfor
    endfor

    return loclist
endfunction

function! s:line_in_block(lineno, startline, size)
    return a:startline <= a:lineno && a:lineno <= a:startline + a:size
endfunction

function! s:truncate_line(lineno)
    let original = getline(a:lineno)
    if strwidth(original) > g:superdiff_loctext_maxwidth
        return original[:g:superdiff_loctext_maxwidth] . '...'
    else
        return original
    endif
endfunction
