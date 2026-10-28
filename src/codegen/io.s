# Entrada/salida nativa monohilo, sin libc. Cada lectura admite hasta 1 MiB.
# Todos los descriptores se cierran antes de retornar; los errores terminan el proceso.
.section .text

# Convierte una ruta cadena a bytes terminados en NUL y rechaza NUL dentro de la ruta.
pf_io_path:
    sub rsp, 24
    mov [rsp], rdi
    mov rcx, [rdi]
    test rcx, rcx
    jz pf_io_error
    cmp rcx, 4095
    ja pf_io_error
    lea rsi, [rdi+8]
.L_path_check:
    cmp BYTE PTR [rsi], 0
    je pf_io_error
    inc rsi
    dec rcx
    jnz .L_path_check
    mov rsi, [rdi]
    inc rsi
    call pf_col_alloc
    mov [rsp+8], rax
    mov rsi, [rsp]
    mov rcx, [rsi]
    add rsi, 8
    mov rdi, rax
    rep movsb
    mov BYTE PTR [rdi], 0
    mov rax, [rsp+8]
    add rsp, 24
    ret

# abrir(ruta) devuelve el contenido UTF-8 completo; no expone descriptores al usuario.
pf_read_file:
    call pf_io_path
    mov rdi, rax
    xor esi, esi
    xor edx, edx
    mov eax, 2
    syscall
    test rax, rax
    js pf_io_error
    mov rdi, rax
    xor esi, esi
    jmp pf_io_read

# entrada() lee una línea sin el salto LF; EOF devuelve cadena vacía.
pf_input:
    xor edi, edi
    mov esi, 1

# Modo rsi=1 lee stdin de byte en byte para no consumir la siguiente línea.
pf_io_read:
    sub rsp, 56
    mov [rsp], rdi
    mov [rsp+8], rsi
    mov esi, 1048585
    call pf_col_alloc
    mov [rsp+16], rax
    mov QWORD PTR [rsp+24], 0
.L_io_read_loop:
    mov rdi, [rsp]
    mov rsi, [rsp+16]
    add rsi, 8
    add rsi, [rsp+24]
    mov edx, 1048577
    sub rdx, [rsp+24]
    cmp QWORD PTR [rsp+8], 0
    je .L_io_read_call
    mov edx, 1
.L_io_read_call:
    xor eax, eax
    syscall
    cmp rax, -4
    je .L_io_read_loop
    test rax, rax
    js pf_io_error
    jz .L_io_read_done
    cmp QWORD PTR [rsp+8], 0
    je .L_io_read_count
    cmp BYTE PTR [rsi], 10
    je .L_io_read_done
.L_io_read_count:
    add [rsp+24], rax
    cmp QWORD PTR [rsp+24], 1048576
    ja pf_io_limit
    jmp .L_io_read_loop
.L_io_read_done:
    cmp QWORD PTR [rsp+8], 0
    jne .L_io_read_text
    mov rdi, [rsp]
    mov eax, 3
    syscall
    test rax, rax
    js pf_io_error
.L_io_read_text:
    mov rdi, [rsp+16]
    mov rax, [rsp+24]
    mov [rdi], rax
    call pf_io_utf8
    add rsp, 56
    ret

# escribir(ruta, texto) crea o trunca el archivo y escribe exactamente sus bytes.
pf_write_file:
    sub rsp, 24
    mov [rsp], rsi
    call pf_io_path
    mov rdi, rax
    mov esi, 577
    mov edx, 420
    mov eax, 2
    syscall
    test rax, rax
    js pf_io_error
    mov [rsp+8], rax
    mov rsi, [rsp]
    mov rdx, [rsi]
    add rsi, 8
.L_io_write_loop:
    test rdx, rdx
    jz .L_io_write_close
    mov rdi, [rsp+8]
    mov eax, 1
    syscall
    cmp rax, -4
    je .L_io_write_loop
    test rax, rax
    jle pf_io_error
    add rsi, rax
    sub rdx, rax
    jmp .L_io_write_loop
.L_io_write_close:
    mov rdi, [rsp+8]
    mov eax, 3
    syscall
    test rax, rax
    js pf_io_error
    xor eax, eax
    add rsp, 24
    ret

# Valida UTF-8: rechaza continuaciones aisladas, secuencias cortas, sobrelargas,
# sustitutos Unicode y puntos superiores a U+10FFFF. Retorna el descriptor original.
pf_io_utf8:
    mov rax, rdi
    lea rsi, [rdi+8]
    mov rdx, [rdi]
    add rdx, rsi
.L_utf8_next:
    cmp rsi, rdx
    jae .L_utf8_done
    movzx r8d, BYTE PTR [rsi]
    inc rsi
    cmp r8d, 128
    jb .L_utf8_next
    cmp r8d, 194
    jb pf_io_utf8_error
    cmp r8d, 224
    jb .L_utf8_two
    cmp r8d, 240
    jb .L_utf8_three
    cmp r8d, 245
    jae pf_io_utf8_error
    and r8d, 7
    mov r9d, 3
    mov r10d, 65536
    jmp .L_utf8_cont
.L_utf8_two:
    and r8d, 31
    mov r9d, 1
    mov r10d, 128
    jmp .L_utf8_cont
.L_utf8_three:
    and r8d, 15
    mov r9d, 2
    mov r10d, 2048
.L_utf8_cont:
    cmp rsi, rdx
    jae pf_io_utf8_error
    movzx r11d, BYTE PTR [rsi]
    inc rsi
    sub r11d, 128
    cmp r11d, 63
    ja pf_io_utf8_error
    shl r8d, 6
    or r8d, r11d
    dec r9d
    jnz .L_utf8_cont
    cmp r8d, r10d
    jb pf_io_utf8_error
    cmp r8d, 1114111
    ja pf_io_utf8_error
    cmp r8d, 55296
    jb .L_utf8_next
    cmp r8d, 57343
    jbe pf_io_utf8_error
    jmp .L_utf8_next
.L_utf8_done:
    ret

pf_io_error:
    lea rsi, [rip+.L_io_error]
    mov edx, OFFSET FLAT:.L_io_error_size
    jmp pf_col_fail
pf_io_limit:
    lea rsi, [rip+.L_io_limit]
    mov edx, OFFSET FLAT:.L_io_limit_size
    jmp pf_col_fail
pf_io_utf8_error:
    lea rsi, [rip+.L_io_utf8]
    mov edx, OFFSET FLAT:.L_io_utf8_size
    jmp pf_col_fail
.section .rodata
.L_io_error: .ascii "Error ST: fallo de entrada/salida (ruta, permisos o dispositivo)\n"
.L_io_error_size = . - .L_io_error
.L_io_limit: .ascii "Error ST: lectura superior a 1 MiB\n"
.L_io_limit_size = . - .L_io_limit
.L_io_utf8: .ascii "Error ST: entrada no valida como UTF-8\n"
.L_io_utf8_size = . - .L_io_utf8
.section .text

# entrada(mensaje): escribe exactamente los bytes UTF-8 sin añadir una nueva línea.
# Reintenta EINTR y escrituras parciales; el mensaje ya se evaluó una sola vez.
.section .text
pf_input_prompt:
    mov rdx, [rdi]
    lea rsi, [rdi+8]
.L_prompt_more:
    test rdx, rdx
    jz pf_input
    mov eax, 1
    mov edi, 1
    syscall
    cmp rax, -4
    je .L_prompt_more
    test rax, rax
    jle pf_io_error
    add rsi, rax
    sub rdx, rax
    jmp .L_prompt_more
