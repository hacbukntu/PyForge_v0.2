# Teclado inmediato para juegos de terminal Linux x86-64, sin libc ni Python.
# Cada lectura conserva una copia de termios y la restaura antes de retornar.
# Los códigos 1001..1004 representan flechas y no chocan con bytes ASCII.
.section .text

# Devuelve 1 si stdin acepta TCGETS; una tubería o archivo devuelve 0.
pf_keyboard_is_terminal:
    sub rsp, 48
    xor edi, edi
    mov esi, 0x5401
    mov rdx, rsp
    mov eax, 16
    syscall
    test rax, rax
    setns al
    movzx eax, al
    add rsp, 48
    ret

# leer(espera_ms): -1 significa que no hubo tecla; -2.. son -errno.
# espera_ms se limita a 1000 para que termios nunca permanezca modificado mucho tiempo.
pf_keyboard_read:
    cmp rdi, -1
    jl .L_keyboard_argument
    cmp rdi, 1000
    jg .L_keyboard_argument
    sub rsp, 128
    mov [rsp+112], rdi

    # Guarda los 36 bytes de la configuración vigente en rsp.
    xor edi, edi
    mov esi, 0x5401
    mov rdx, rsp
    mov eax, 16
    syscall
    test rax, rax
    js .L_keyboard_early_error

    # Copia termios y desactiva ICANON, ECHO e ISIG solo durante esta lectura.
    mov rax, [rsp]
    mov [rsp+40], rax
    mov rax, [rsp+8]
    mov [rsp+48], rax
    mov rax, [rsp+16]
    mov [rsp+56], rax
    mov rax, [rsp+24]
    mov [rsp+64], rax
    mov eax, [rsp+32]
    mov [rsp+72], eax
    and DWORD PTR [rsp+52], -12
    mov BYTE PTR [rsp+62], 0
    mov BYTE PTR [rsp+63], 0
    xor edi, edi
    mov esi, 0x5402
    lea rdx, [rsp+40]
    mov eax, 16
    syscall
    test rax, rax
    js .L_keyboard_early_error

    # poll permite un bucle de juego no bloqueante con espera máxima configurable.
    mov DWORD PTR [rsp+80], 0
    mov WORD PTR [rsp+84], 1
    mov WORD PTR [rsp+86], 0
    lea rdi, [rsp+80]
    mov esi, 1
    mov rdx, [rsp+112]
    mov eax, 7
    syscall
    test rax, rax
    js .L_keyboard_save_result
    jz .L_keyboard_timeout

    # Lee hasta cuatro bytes para reconocer ESC [ A/B/C/D; el resto conserva el primer byte.
    xor edi, edi
    lea rsi, [rsp+88]
    mov edx, 4
    xor eax, eax
    syscall
    test rax, rax
    js .L_keyboard_save_result
    jz .L_keyboard_timeout
    movzx r10d, BYTE PTR [rsp+88]
    cmp r10d, 27
    jne .L_keyboard_have_result
    cmp rax, 3
    jb .L_keyboard_have_result
    cmp BYTE PTR [rsp+89], 91
    jne .L_keyboard_have_result
    movzx r11d, BYTE PTR [rsp+90]
    mov r10d, 1001
    cmp r11d, 65
    je .L_keyboard_have_result
    inc r10d
    cmp r11d, 66
    je .L_keyboard_have_result
    inc r10d
    cmp r11d, 67
    je .L_keyboard_have_result
    inc r10d
    cmp r11d, 68
    je .L_keyboard_have_result
    mov r10d, 27
.L_keyboard_have_result:
    mov rax, r10
    jmp .L_keyboard_save_result
.L_keyboard_timeout:
    mov rax, -1
.L_keyboard_save_result:
    mov [rsp+120], rax
    # La restauración se intenta en todas las salidas posteriores a TCSETS.
    xor edi, edi
    mov esi, 0x5402
    mov rdx, rsp
    mov eax, 16
    syscall
    mov rax, [rsp+120]
    add rsp, 128
    ret
.L_keyboard_early_error:
    add rsp, 128
    ret
.L_keyboard_argument:
    mov rax, -22
    ret
