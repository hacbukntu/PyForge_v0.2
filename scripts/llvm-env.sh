#!/usr/bin/env bash
# Localiza LLVM 21 para los scripts de desarrollo sin guardar rutas personales.
# Debe cargarse con `source`; respeta primero una ruta definida por el usuario.

if [[ -z "${LLVM_SYS_211_PREFIX:-}" ]]; then
    pyforge_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    for llvm_prefix in \
        "$pyforge_root/target/tools/llvm-21/usr/lib/llvm-21" \
        /usr/lib/llvm-21
    do
        if [[ -x "$llvm_prefix/bin/llvm-config" ]]; then
            export LLVM_SYS_211_PREFIX="$llvm_prefix"
            break
        fi
    done
    unset pyforge_root llvm_prefix
fi

if [[ -z "${LLVM_SYS_211_PREFIX:-}" ]]; then
    echo "Falta LLVM 21. Instala llvm-21-dev o define LLVM_SYS_211_PREFIX." >&2
    return 1 2>/dev/null || exit 1
fi
