# Escalares nativos: cadenas UTF-8 y flotantes IEEE-754. Sin libc ni intérprete.
# ABI interna: argumentos en rdi/rsi, resultado en rax; no modifica registros preservados.
.section .text

# Escribe el búfer rsi/rdx completo y añade un salto de línea.
# Reintenta EINTR y escrituras parciales. Un fallo devuelve un diagnóstico de ejecución.
pf_write_line:
    test rdx, rdx
    jz .L_write_newline
.L_write_more:
    mov eax, 1
    mov edi, 1
    syscall
    cmp rax, -4
    je .L_write_more
    test rax, rax
    jle .L_scalar_io_error
    add rsi, rax
    sub rdx, rax
    jnz .L_write_more
.L_write_newline:
    lea rsi, [rip+.L_scalar_newline]
    mov edx, 1
.L_newline_retry:
    mov eax, 1
    mov edi, 1
    syscall
    cmp rax, -4
    je .L_newline_retry
    cmp rax, 1
    jne .L_scalar_io_error
    xor eax, eax
    ret
.L_scalar_io_error:
    lea rsi, [rip+.L_scalar_io_message]
    mov edx, OFFSET FLAT:.L_scalar_io_size
    jmp pf_col_fail

# El descriptor contiene longitud en bytes seguida de contenido, sin terminador NUL.
pf_text_print:
    mov rdx, [rdi]
    lea rsi, [rdi+8]
    jmp pf_write_line

# Cuenta puntos de código Unicode: cada byte que no sea continuación inicia uno.
# No cuenta grafemas; las cadenas proceden de literales UTF-8 y concatenaciones válidas.
pf_text_length:
    mov rcx, [rdi]
    add rdi, 8
    xor eax, eax
.L_text_count:
    test rcx, rcx
    jz .L_text_count_done
    mov dl, [rdi]
    and dl, 0xc0
    cmp dl, 0x80
    je .L_text_continuation
    inc rax
.L_text_continuation:
    inc rdi
    dec rcx
    jmp .L_text_count
.L_text_count_done:
    ret

# Igualdad por longitud y bytes, no por dirección del descriptor.
pf_text_eq:
    mov rcx, [rdi]
    cmp rcx, [rsi]
    jne .L_text_not_equal
    add rdi, 8
    add rsi, 8
    test rcx, rcx
    jz .L_text_equal
    repe cmpsb
    jne .L_text_not_equal
.L_text_equal:
    mov eax, 1
    ret
.L_text_not_equal:
    xor eax, eax
    ret

# Concatena sin modificar los operandos. La reserva vive hasta que termina el proceso.
# Guarda las referencias en pila porque mmap modifica registros volátiles.
pf_text_concat:
    sub rsp, 40
    mov [rsp], rdi
    mov [rsp+8], rsi
    mov rax, [rdi]
    add rax, [rsi]
    jc pf_col_memory_error
    mov [rsp+16], rax
    add rax, 8
    jc pf_col_memory_error
    mov rsi, rax
    call pf_col_alloc
    mov [rsp+24], rax
    mov rcx, [rsp+16]
    mov [rax], rcx
    lea rdi, [rax+8]
    mov rsi, [rsp]
    mov rcx, [rsi]
    add rsi, 8
    rep movsb
    mov rsi, [rsp+8]
    mov rcx, [rsi]
    add rsi, 8
    rep movsb
    mov rax, [rsp+24]
    add rsp, 40
    ret

# Los flotantes viajan como bits en registros enteros y se calculan mediante SSE2.
pf_float_add:
    movq xmm0, rdi
    movq xmm1, rsi
    addsd xmm0, xmm1
    movq rax, xmm0
    ret
pf_float_sub:
    movq xmm0, rdi
    movq xmm1, rsi
    subsd xmm0, xmm1
    movq rax, xmm0
    ret
pf_float_mul:
    movq xmm0, rdi
    movq xmm1, rsi
    mulsd xmm0, xmm1
    movq rax, xmm0
    ret
pf_float_div:
    movq xmm0, rdi
    movq xmm1, rsi
    divsd xmm0, xmm1
    movq rax, xmm0
    ret
pf_float_neg:
    mov rax, rdi
    btc rax, 63
    ret

# Las comparaciones no ordenadas (NaN) son falsas, excepto !=.
pf_float_eq:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    sete al
    setnp dl
    and al, dl
    movzx eax, al
    ret
pf_float_ne:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    setne al
    setp dl
    or al, dl
    movzx eax, al
    ret
pf_float_lt:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    setb al
    setnp dl
    and al, dl
    movzx eax, al
    ret
pf_float_le:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    setbe al
    setnp dl
    and al, dl
    movzx eax, al
    ret
pf_float_gt:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    seta al
    movzx eax, al
    ret
pf_float_ge:
    movq xmm0, rdi
    movq xmm1, rsi
    ucomisd xmm0, xmm1
    setae al
    movzx eax, al
    ret

# Formatea con hasta seis decimales y al menos uno. Para magnitudes <1e-6 o >=1e12
# usa mantisa y exponente decimal. La precisión de presentación no cambia el valor f64.
# r8 conserva el signo, r9 el exponente; el búfer de 128 bytes queda en la pila.
pf_float_text:
    mov eax, 1
    jmp .L_float_entry
pf_float_print:
    xor eax, eax
.L_float_entry:
    sub rsp, 152
    mov [rsp+136], rax
    mov r8, rdi
    shr r8, 63
    btr rdi, 63
    movq xmm0, rdi
    mov rax, rdi
    shr rax, 52
    cmp eax, 2047
    je .L_float_special
    xor r9d, r9d
    test rdi, rdi
    jz .L_float_scale
    ucomisd xmm0, QWORD PTR [rip+.L_float_large]
    jae .L_float_normalize_large
    ucomisd xmm0, QWORD PTR [rip+.L_float_small]
    jae .L_float_scale
.L_float_normalize_small:
    ucomisd xmm0, QWORD PTR [rip+.L_float_one]
    jae .L_float_scale
    mulsd xmm0, QWORD PTR [rip+.L_float_ten]
    dec r9
    jmp .L_float_normalize_small
.L_float_normalize_large:
    ucomisd xmm0, QWORD PTR [rip+.L_float_ten]
    jb .L_float_scale
    divsd xmm0, QWORD PTR [rip+.L_float_ten]
    inc r9
    jmp .L_float_normalize_large
.L_float_scale:
    mulsd xmm0, QWORD PTR [rip+.L_float_million]
    cvtsd2si rax, xmm0
    # Una mantisa que redondea a diez se normaliza otra vez.
    test r9, r9
    jz .L_float_digits
    cmp rax, 10000000
    jne .L_float_digits
    mov eax, 1000000
    inc r9
.L_float_digits:
    lea rsi, [rsp+80]
    mov r10, rsi
    mov ecx, 6
    mov edi, 10
.L_float_fraction:
    xor edx, edx
    div rdi
    add dl, '0'
    dec rsi
    mov [rsi], dl
    dec ecx
    jnz .L_float_fraction
    # Conserva al menos un decimal, eliminando ceros finales del fragmento fraccionario.
    lea r11, [rsi+1]
.L_float_trim:
    cmp r10, r11
    jbe .L_float_dot
    cmp BYTE PTR [r10-1], '0'
    jne .L_float_dot
    dec r10
    jmp .L_float_trim
.L_float_dot:
    dec rsi
    mov BYTE PTR [rsi], '.'
.L_float_integer:
    xor edx, edx
    div rdi
    add dl, '0'
    dec rsi
    mov [rsi], dl
    test rax, rax
    jnz .L_float_integer
    test r8, r8
    jz .L_float_exponent
    dec rsi
    mov BYTE PTR [rsi], '-'
.L_float_exponent:
    test r9, r9
    jz .L_float_write
    mov BYTE PTR [r10], 'e'
    inc r10
    test r9, r9
    jns .L_float_positive_exp
    mov BYTE PTR [r10], '-'
    neg r9
    jmp .L_float_exp_digits
.L_float_positive_exp:
    mov BYTE PTR [r10], '+'
.L_float_exp_digits:
    inc r10
    mov rax, r9
    lea r11, [rsp+128]
    mov rcx, r11
.L_float_exp_reverse:
    xor edx, edx
    div rdi
    add dl, '0'
    dec rcx
    mov [rcx], dl
    test rax, rax
    jnz .L_float_exp_reverse
.L_float_exp_copy:
    mov dl, [rcx]
    mov [r10], dl
    inc rcx
    inc r10
    cmp rcx, r11
    jne .L_float_exp_copy
.L_float_write:
    mov rdx, r10
    sub rdx, rsi
    cmp QWORD PTR [rsp+136], 0
    je .L_float_output
    call pf_text_from_buffer
    add rsp, 152
    ret
.L_float_output:
    call pf_write_line
    add rsp, 152
    ret
.L_float_special:
    shl rdi, 12
    test rdi, rdi
    jnz .L_float_nan
    lea rsi, [rip+.L_float_inf]
    mov edx, 3
    test r8, r8
    jz .L_float_special_write
    lea rsi, [rip+.L_float_negative_inf]
    mov edx, 4
    jmp .L_float_special_write
.L_float_nan:
    lea rsi, [rip+.L_float_nan_text]
    mov edx, 3
.L_float_special_write:
    mov r10, rsi
    add r10, rdx
    jmp .L_float_write
.section .rodata
.L_scalar_newline: .byte 10
.L_scalar_io_message: .ascii "Error ST: no se pudo escribir la salida\n"
.L_scalar_io_size = . - .L_scalar_io_message
.L_float_inf: .ascii "inf"
.L_float_negative_inf: .ascii "-inf"
.L_float_nan_text: .ascii "nan"
.balign 8
.L_float_one: .double 1.0
.L_float_ten: .double 10.0
.L_float_million: .double 1000000.0
.L_float_large: .double 1000000000000.0
.L_float_small: .double 0.000001
.section .text

# Copia un búfer rsi/rdx a una cadena nueva; también sirve al formato de flotantes.
pf_text_from_buffer:
    sub rsp, 40
    mov [rsp], rsi
    mov [rsp+8], rdx
    lea rsi, [rdx+8]
    call pf_col_alloc
    mov [rsp+16], rax
    mov rcx, [rsp+8]
    mov [rax], rcx
    lea rdi, [rax+8]
    mov rsi, [rsp]
    rep movsb
    mov rax, [rsp+16]
    add rsp, 40
    ret

# Convierte i64, incluyendo el mínimo negativo, sin pasar por flotante.
pf_int_text:
    sub rsp, 72
    lea rsi, [rsp+64]
    mov rax, rdi
    mov r8, rdi
    test rax, rax
    jns .L_int_text_digits
    neg rax
.L_int_text_digits:
    mov ecx, 10
.L_int_text_loop:
    xor edx, edx
    div rcx
    add dl, '0'
    dec rsi
    mov [rsi], dl
    test rax, rax
    jnz .L_int_text_loop
    test r8, r8
    jns .L_int_text_copy
    dec rsi
    mov BYTE PTR [rsi], '-'
.L_int_text_copy:
    lea rdx, [rsp+64]
    sub rdx, rsi
    call pf_text_from_buffer
    add rsp, 72
    ret
pf_bool_text:
    lea rax, [rip+.L_text_false]
    test rdi, rdi
    jz .L_bool_text_done
    lea rax, [rip+.L_text_true]
.L_bool_text_done:
    ret
pf_null_text:
    lea rax, [rip+.L_text_null]
    ret

# Devuelve una cadena de un punto de código. Normaliza índices negativos por longitud.
pf_text_index:
    sub rsp, 40
    mov [rsp], rdi
    mov [rsp+8], rsi
    call pf_text_length
    mov rsi, [rsp+8]
    test rsi, rsi
    jns .L_text_index_positive
    add rsi, rax
.L_text_index_positive:
    cmp rsi, rax
    jae pf_col_index_error
    mov rdi, [rsp]
    lea rdx, [rdi+8]
    mov r8, [rdi]
    add r8, rdx
    xor ecx, ecx
.L_text_index_scan:
    cmp rcx, rsi
    je .L_text_index_start
    inc rdx
.L_text_index_cont:
    cmp rdx, r8
    jae .L_text_index_start
    mov al, [rdx]
    and al, 0xc0
    cmp al, 0x80
    jne .L_text_index_next
    inc rdx
    jmp .L_text_index_cont
.L_text_index_next:
    inc rcx
    jmp .L_text_index_scan
.L_text_index_start:
    mov rsi, rdx
    inc rdx
.L_text_index_end:
    cmp rdx, r8
    jae .L_text_index_copy
    mov al, [rdx]
    and al, 0xc0
    cmp al, 0x80
    jne .L_text_index_copy
    inc rdx
    jmp .L_text_index_end
.L_text_index_copy:
    sub rdx, rsi
    call pf_text_from_buffer
    add rsp, 40
    ret

# Búsqueda interna por bytes: fuente rdi, patrón rsi, inicio rdx. Devuelve offset o -1.
# No modifica rdi/rsi/rdx, lo que permite reutilizarla desde sustitución y búsqueda Unicode.
pf_text_find_bytes:
    mov r8, [rdi]
    mov r9, [rsi]
    mov rax, rdx
    cmp r9, r8
    ja .L_find_missing
    sub r8, r9
.L_find_candidate:
    cmp rax, r8
    ja .L_find_missing
    xor ecx, ecx
.L_find_compare:
    cmp rcx, r9
    je .L_find_done
    lea r10, [rax+rcx]
    mov r11b, [rdi+r10+8]
    cmp r11b, [rsi+rcx+8]
    jne .L_find_advance
    inc rcx
    jmp .L_find_compare
.L_find_advance:
    inc rax
    jmp .L_find_candidate
.L_find_missing:
    mov rax, -1
.L_find_done:
    ret

# Buscar devuelve un índice de punto de código; patrón vacío devuelve cero.
pf_text_find:
    xor edx, edx
    call pf_text_find_bytes
    cmp rax, -1
    je .L_find_public_done
    xor ecx, ecx
    xor edx, edx
.L_find_count:
    cmp rcx, rax
    jae .L_find_count_done
    mov r8b, [rdi+rcx+8]
    and r8b, 0xc0
    cmp r8b, 0x80
    je .L_find_count_next
    inc rdx
.L_find_count_next:
    inc rcx
    jmp .L_find_count
.L_find_count_done:
    mov rax, rdx
.L_find_public_done:
    ret

# Reemplaza coincidencias no superpuestas. Reserva una cota superior segura y copia
# una vez los segmentos. Patrón vacío se rechaza explícitamente para evitar ambigüedad.
pf_text_replace:
    cmp QWORD PTR [rsi], 0
    je .L_replace_empty
    sub rsp, 72
    mov [rsp], rdi
    mov [rsp+8], rsi
    mov [rsp+16], rdx
    mov rax, [rdx]
    add rax, 1
    jc pf_col_memory_error
    mul QWORD PTR [rdi]
    test rdx, rdx
    jnz pf_col_memory_error
    add rax, 8
    jc pf_col_memory_error
    mov rsi, rax
    call pf_col_alloc
    mov [rsp+24], rax
    add rax, 8
    mov [rsp+32], rax
    mov QWORD PTR [rsp+40], 0
.L_replace_loop:
    mov rdi, [rsp]
    mov rsi, [rsp+8]
    mov rdx, [rsp+40]
    call pf_text_find_bytes
    mov [rsp+48], rax
    cmp rax, -1
    jne .L_replace_segment
    mov rdi, [rsp]
    mov rax, [rdi]
.L_replace_segment:
    sub rax, [rsp+40]
    mov rcx, rax
    mov rsi, [rsp]
    add rsi, 8
    add rsi, [rsp+40]
    mov rdi, [rsp+32]
    rep movsb
    cmp QWORD PTR [rsp+48], -1
    je .L_replace_done
    mov rsi, [rsp+16]
    mov rcx, [rsi]
    add rsi, 8
    rep movsb
    mov [rsp+32], rdi
    mov rax, [rsp+48]
    mov rsi, [rsp+8]
    add rax, [rsi]
    mov [rsp+40], rax
    jmp .L_replace_loop
.L_replace_done:
    mov rax, [rsp+24]
    sub rdi, rax
    sub rdi, 8
    mov [rax], rdi
    add rsp, 72
    ret
.L_replace_empty:
    lea rsi, [rip+.L_replace_error]
    mov edx, OFFSET FLAT:.L_replace_error_size
    jmp pf_col_fail
.section .rodata
.balign 8
.L_text_true: .quad 4
.ascii "True"
.balign 8
.L_text_false: .quad 5
.ascii "False"
.balign 8
.L_text_null: .quad 4
.ascii "nulo"
.L_replace_error: .ascii "Error ST: reemplazar requiere un patron no vacio\n"
.L_replace_error_size = . - .L_replace_error
.section .text

# Busca una clave textual por contenido; los conjuntos siguen usando claves enteras.
# Conserva rdi/rsi/rdx para que Get/Set compartan el resultado de la búsqueda.
pf_text_dict_find:
    sub rsp, 56
    mov [rsp], rdi
    mov [rsp+8], rsi
    mov [rsp+16], rdx
    mov rax, [rdi+16]
    mov [rsp+24], rax
    mov rax, [rdi]
    mov [rsp+32], rax
.L_text_dict_search:
    cmp QWORD PTR [rsp+32], 0
    je .L_text_dict_missing
    mov rax, [rsp+24]
    mov rdi, [rax]
    mov rsi, [rsp+8]
    call pf_text_eq
    test rax, rax
    jnz .L_text_dict_found
    add QWORD PTR [rsp+24], 16
    dec QWORD PTR [rsp+32]
    jmp .L_text_dict_search
.L_text_dict_missing:
    xor eax, eax
    jmp .L_text_dict_restore
.L_text_dict_found:
    mov rax, [rsp+24]
.L_text_dict_restore:
    mov rdi, [rsp]
    mov rsi, [rsp+8]
    mov rdx, [rsp+16]
    add rsp, 56
    ret
pf_text_dict_get:
    call pf_text_dict_find
    test rax, rax
    jz pf_col_key_error
    mov rax, [rax+8]
    ret
pf_text_dict_set:
    call pf_text_dict_find
    test rax, rax
    jz pf_col_append
    mov [rax+8], rdx
    mov rax, rdi
    ret

# La aritmética entera comprobada termina con diagnóstico en lugar de envolver o SIGFPE.
pf_integer_overflow:
    lea rsi, [rip+.L_integer_overflow]
    mov edx, OFFSET FLAT:.L_integer_overflow_size
    jmp pf_col_fail
pf_integer_zero:
    lea rsi, [rip+.L_integer_zero]
    mov edx, OFFSET FLAT:.L_integer_zero_size
    jmp pf_col_fail
.section .rodata
.L_integer_overflow: .ascii "Error ST: desbordamiento de entero i64\n"
.L_integer_overflow_size = . - .L_integer_overflow
.L_integer_zero: .ascii "Error ST: division o resto por cero\n"
.L_integer_zero_size = . - .L_integer_zero
.section .text
