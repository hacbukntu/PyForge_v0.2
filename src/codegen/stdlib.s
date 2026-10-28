# Primitivas de biblioteca estándar, ABI interna: argumentos rdi/rsi, resultado rax.
# Linux x86-64, sin libc. Nunca modifican registros preservados por el llamador.
.section .text

# clock_gettime(reloj, timespec): convierte segundos/nanosegundos a milisegundos i64.
pf_std_clock:
    sub rsp, 24
    mov rsi, rsp
    mov eax, 228
    syscall
    test rax, rax
    js pf_io_error
    mov r8, [rsp]
    imul r8, r8, 1000
    jo pf_std_error
    mov rax, [rsp+8]
    xor edx, edx
    mov ecx, 1000000
    div rcx
    add rax, r8
    jo pf_std_error
    add rsp, 24
    ret

# nanosleep: la solicitud y el tiempo restante comparten timespec; EINTR reintenta.
pf_std_sleep:
    test rdi, rdi
    js pf_std_error
    sub rsp, 24
    mov rax, rdi
    xor edx, edx
    mov ecx, 1000
    div rcx
    mov [rsp], rax
    imul rdx, rdx, 1000000
    mov [rsp+8], rdx
.L_std_sleep_retry:
    mov rdi, rsp
    mov rsi, rsp
    mov eax, 35
    syscall
    cmp rax, -4
    je .L_std_sleep_retry
    add rsp, 24
    ret

# getcwd llena un búfer acotado; se elimina el terminador y se valida UTF-8.
pf_std_cwd:
    sub rsp, 24
    mov esi, 4104
    call pf_col_alloc
    mov [rsp], rax
    lea rdi, [rax+8]
    mov esi, 4096
    mov eax, 79
    syscall
    test rax, rax
    jle pf_io_error
    dec rax
    mov rdi, [rsp]
    mov [rdi], rax
    call pf_io_utf8
    mov rax, [rsp]
    add rsp, 24
    ret

# access(F_OK) retorna 0 o -errno; ST distingue ausencia y errores de permisos.
pf_std_exists:
    sub rsp, 8
    call pf_io_path
    mov rdi, rax
    xor esi, esi
    mov eax, 21
    syscall
    add rsp, 8
    ret

# mkdir usa 0777 antes de umask; no crea padres ni modifica permisos existentes.
pf_std_mkdir:
    sub rsp, 8
    call pf_io_path
    mov rdi, rax
    mov esi, 511
    mov eax, 83
    syscall
    add rsp, 8
    ret

# unlink borra únicamente el nombre indicado; no hay expansión de comodines.
pf_std_unlink:
    sub rsp, 8
    call pf_io_path
    mov rdi, rax
    mov eax, 87
    syscall
    add rsp, 8
    ret

# sqrtsd opera directamente sobre los bits f64 recibidos en el registro entero.
pf_std_sqrt:
    movq xmm0, rdi
    sqrtsd xmm0, xmm0
    movq rax, xmm0
    ret

# Codifica un escalar Unicode válido; rechaza sustitutos UTF-16 y valores fuera del rango.
# El búfer de cuatro bytes vive en la pila y pf_text_from_buffer copia antes de retornar.
pf_std_unicode:
    cmp rdi, 0x10ffff
    ja pf_std_error
    cmp rdi, 0xd800
    jb .L_std_unicode_valid
    cmp rdi, 0xdfff
    jbe pf_std_error
.L_std_unicode_valid:
    sub rsp, 24
    mov eax, edi
    mov edx, 1
    cmp eax, 0x80
    jb .L_std_unicode_last
    mov ecx, eax
    and ecx, 63
    or ecx, 128
    cmp eax, 0x800
    jae .L_std_unicode_three
    mov [rsp+1], cl
    shr eax, 6
    or eax, 192
    mov edx, 2
    jmp .L_std_unicode_last
.L_std_unicode_three:
    cmp eax, 0x10000
    jae .L_std_unicode_four
    mov [rsp+2], cl
    shr eax, 6
    mov ecx, eax
    and ecx, 63
    or ecx, 128
    mov [rsp+1], cl
    shr eax, 6
    or eax, 224
    mov edx, 3
    jmp .L_std_unicode_last
.L_std_unicode_four:
    mov [rsp+3], cl
    shr eax, 6
    mov ecx, eax
    and ecx, 63
    or ecx, 128
    mov [rsp+2], cl
    shr eax, 6
    mov ecx, eax
    and ecx, 63
    or ecx, 128
    mov [rsp+1], cl
    shr eax, 6
    or eax, 240
    mov edx, 4
.L_std_unicode_last:
    mov [rsp], al
    mov rsi, rsp
    call pf_text_from_buffer
    add rsp, 24
    ret

# Los fallos de representación del runtime siguen la política fatal del núcleo existente.
pf_std_error:
    lea rsi, [rip+.L_std_error_message]
    mov edx, OFFSET FLAT:.L_std_error_size
    jmp pf_col_fail
.section .rodata
.L_std_error_message: .ascii "Error ST: valor fuera de rango en biblioteca estandar\n"
.L_std_error_size = . - .L_std_error_message

.section .text
# Las cadenas ST ya son UTF-8 válido. Decodifica el primer escalar sin reservar memoria.
pf_std_codepoint:
    mov rax, -1
    cmp QWORD PTR [rdi], 0
    je .L_std_code_return
    movzx eax, BYTE PTR [rdi+8]
    cmp eax, 128
    jb .L_std_code_return
    mov ecx, 1
    cmp eax, 224
    jb .L_std_code_two
    mov ecx, 2
    cmp eax, 240
    jb .L_std_code_three
    mov ecx, 3
    and eax, 7
    jmp .L_std_code_loop_start
.L_std_code_three:
    and eax, 15
    jmp .L_std_code_loop_start
.L_std_code_two:
    and eax, 31
.L_std_code_loop_start:
    add rdi, 9
.L_std_code_loop:
    movzx edx, BYTE PTR [rdi]
    and edx, 63
    shl eax, 6
    or eax, edx
    inc rdi
    dec ecx
    jnz .L_std_code_loop
.L_std_code_return:
    ret
