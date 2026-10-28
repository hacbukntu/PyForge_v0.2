# Puente opcional Linux x86-64: un auxiliar Python por programa, sin shell.
# socketpair es local; no abre puertos TCP. Tramas JSON de hasta 4095 bytes + LF.
.section .bss
.balign 8
pf_bridge_env: .quad 0
pf_bridge_pid: .quad 0
pf_bridge_fd: .quad 0
pf_bridge_pair: .zero 8
pf_bridge_buffer: .zero 4096
.section .rodata
pf_bridge_isolated: .asciz "-I"
pf_bridge_command: .asciz "-c"
pf_bridge_lf: .byte 10
.balign 8
pf_bridge_error:
.quad pf_bridge_error_end-pf_bridge_error_text
pf_bridge_error_text: .ascii "INTEGRACION: auxiliar no disponible, canal cerrado o tiempo agotado; revisa Python >= 3.11 y el intérprete seleccionado al compilar"
pf_bridge_error_end:
.section .data
.balign 8
pf_bridge_argv: .quad pf_bridge_python, pf_bridge_isolated, pf_bridge_command, pf_bridge_source, 0
.section .text

# Guarda la petición en la pila, abre el canal bajo demanda y copia la respuesta.
pf_bridge_call:
    sub rsp, 40
    mov [rsp], rdi
    cmp QWORD PTR [rdi], 4095
    ja .L_bridge_fail
    cmp QWORD PTR [rip+pf_bridge_pid], 0
    jne .L_bridge_send
    mov edi, 1
    mov esi, 524289
    xor edx, edx
    lea r10, [rip+pf_bridge_pair]
    mov eax, 53
    syscall
    test rax, rax
    js .L_bridge_fail
    mov eax, 57
    syscall
    test rax, rax
    js .L_bridge_fork_fail
    jz .L_bridge_child
    mov [rip+pf_bridge_pid], rax
    mov edi, [rip+pf_bridge_pair]
    mov [rip+pf_bridge_fd], rdi
    mov edi, [rip+pf_bridge_pair+4]
    mov eax, 3
    syscall
.L_bridge_send:
    mov rax, [rsp]
    lea rsi, [rax+8]
    mov rdx, [rax]
    call .L_bridge_write
    lea rsi, [rip+pf_bridge_lf]
    mov edx, 1
    call .L_bridge_write
    mov QWORD PTR [rsp+8], 0
.L_bridge_read:
    # poll antes de cada lectura evita que un auxiliar bloqueado congele ST.
    mov rax, [rip+pf_bridge_fd]
    mov [rsp+16], eax
    mov DWORD PTR [rsp+20], 1
    lea rdi, [rsp+16]
    mov esi, 1
    mov edx, 35000
    mov eax, 7
    syscall
    cmp rax, -4
    je .L_bridge_read
    cmp rax, 1
    jne .L_bridge_fail
    mov rdi, [rip+pf_bridge_fd]
    lea rsi, [rip+pf_bridge_buffer]
    add rsi, [rsp+8]
    mov edx, 1
    xor eax, eax
    syscall
    cmp rax, -4
    je .L_bridge_read
    cmp rax, 1
    jne .L_bridge_fail
    cmp BYTE PTR [rsi], 10
    je .L_bridge_done
    inc QWORD PTR [rsp+8]
    cmp QWORD PTR [rsp+8], 4095
    jae .L_bridge_fail
    jmp .L_bridge_read
.L_bridge_done:
    mov rsi, [rsp+8]
    add rsi, 8
    call pf_col_alloc
    mov rcx, [rsp+8]
    mov [rax], rcx
    lea rdi, [rax+8]
    lea rsi, [rip+pf_bridge_buffer]
    rep movsb
    add rsp, 40
    ret

# sendto(MSG_NOSIGNAL) evita SIGPIPE si el auxiliar termina; reintenta EINTR.
.L_bridge_write:
    test rdx, rdx
    jz .L_bridge_written
    mov rdi, [rip+pf_bridge_fd]
    mov r10d, 16384
    xor r8d, r8d
    xor r9d, r9d
    mov eax, 44
    syscall
    cmp rax, -4
    je .L_bridge_write
    test rax, rax
    jle .L_bridge_fail
    add rsi, rax
    sub rdx, rax
    jmp .L_bridge_write
.L_bridge_written:
    ret

# El hijo reutiliza stdin/stdout exclusivamente para el protocolo y ejecuta Python aislado.
.L_bridge_child:
    mov edi, [rip+pf_bridge_pair]
    mov eax, 3
    syscall
    mov edi, [rip+pf_bridge_pair+4]
    xor esi, esi
    mov eax, 33
    syscall
    test rax, rax
    js .L_bridge_child_exit
    mov edi, [rip+pf_bridge_pair+4]
    mov esi, 1
    mov eax, 33
    syscall
    test rax, rax
    js .L_bridge_child_exit
    mov edi, [rip+pf_bridge_pair+4]
    cmp edi, 1
    jbe .L_bridge_exec
    mov eax, 3
    syscall
.L_bridge_exec:
    lea rdi, [rip+pf_bridge_python]
    lea rsi, [rip+pf_bridge_argv]
    mov rdx, [rip+pf_bridge_env]
    mov eax, 59
    syscall
.L_bridge_child_exit:
    mov edi, 127
    mov eax, 60
    syscall
.L_bridge_fork_fail:
    mov edi, [rip+pf_bridge_pair]
    mov eax, 3
    syscall
    mov edi, [rip+pf_bridge_pair+4]
    mov eax, 3
    syscall
.L_bridge_fail:
    # Descartar el canal evita reutilizar respuestas de una petición fallida.
    mov rdi, [rip+pf_bridge_pid]
    test rdi, rdi
    jz .L_bridge_throw
    mov esi, 9
    mov eax, 62
    syscall
    call pf_bridge_shutdown
.L_bridge_throw:
    lea rdi, [rip+pf_bridge_error]
    jmp pf_throw

# Cierra el canal y recoge el hijo. La llamada normal ocurre después de pf_main.
pf_bridge_shutdown:
    cmp QWORD PTR [rip+pf_bridge_pid], 0
    je .L_bridge_closed
    mov rdi, [rip+pf_bridge_fd]
    mov eax, 3
    syscall
.L_bridge_wait:
    mov rdi, [rip+pf_bridge_pid]
    xor esi, esi
    xor edx, edx
    xor r10d, r10d
    mov eax, 61
    syscall
    cmp rax, -4
    je .L_bridge_wait
    mov QWORD PTR [rip+pf_bridge_pid], 0
.L_bridge_closed:
    ret
